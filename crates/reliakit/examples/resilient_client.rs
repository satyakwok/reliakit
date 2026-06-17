//! One call to a flaky dependency, guarded by the whole reliability stack —
//! reached through a single crate name.
//!
//! Five building blocks cooperate, each clock-agnostic and driven by one
//! millisecond clock owned here:
//!
//! - [`reliakit::retry`] owns the retry loop: it counts attempts, decides
//!   whether an error is worth retrying, and asks for the backoff delay — it
//!   never sleeps on its own, so we hand it a sleeper.
//! - [`reliakit::backoff`] supplies the delay between attempts.
//! - [`reliakit::timeout`] caps the total effort with an overall [`Deadline`];
//!   when it expires, the operation reports it and the retry loop stops.
//! - [`reliakit::ratelimit`] paces calls so we never exceed the budget.
//! - [`reliakit::circuit`] stops hammering a dependency that is already failing.
//!
//! The rate limiter and circuit breaker act as *gates* inside one attempt: an
//! attempt waits for a token and for the breaker to allow the call before it
//! actually runs, so a paced or short-circuited wait is not itself a retry.
//!
//! Run it:
//!
//! ```sh
//! cargo run -p reliakit --example resilient_client \
//!   --features "retry timeout ratelimit circuit backoff"
//! ```
//!
//! [`Deadline`]: reliakit::timeout::Deadline

use std::time::{Duration, Instant};

use reliakit::backoff::Backoff;
use reliakit::circuit::CircuitBreaker;
use reliakit::ratelimit::RateLimiter;
use reliakit::retry::{RetryPolicy, retry_with_sleep};
use reliakit::timeout::{Deadline, Timeout};

/// Why a single guarded attempt did not succeed.
#[derive(Debug)]
enum CallError {
    /// The dependency was reachable but returned an error — worth retrying.
    Unavailable(&'static str),
    /// The overall deadline ran out — not worth retrying.
    DeadlineExpired,
}

/// A dependency that is down for the first few calls, then recovers.
fn call_dependency(attempt: u32) -> Result<(), &'static str> {
    if attempt < 4 {
        Err("upstream unavailable")
    } else {
        Ok(())
    }
}

fn main() {
    // Give the whole operation a 3-second budget.
    let deadline = Timeout::new(3_000).start(0);
    // At most 5 calls per second.
    let mut limiter = RateLimiter::new(5, 1, 200);
    // Trip after 3 consecutive failures; probe again after 500ms.
    let mut breaker = CircuitBreaker::new(3, 500);
    // Back off 100ms, doubling, capped at 1s — used by the retry policy below.
    let backoff =
        Backoff::exponential(Duration::from_millis(100), 2).with_max_delay(Duration::from_secs(1));
    // Up to 12 attempts (the deadline usually stops us first).
    let policy = RetryPolicy::new(12, backoff).expect("max_attempts is non-zero");

    let start = Instant::now();
    let mut attempt = 0u32;

    // The retry loop drives everything: each call to `op` is one guarded attempt,
    // `should_retry` keeps going on transient errors, and the backoff delay is
    // slept by us (the crate never sleeps internally).
    let outcome = retry_with_sleep(
        &policy,
        // One guarded attempt: gate on the rate limiter and the breaker, then call.
        || {
            // Pace: wait for a token (a paced wait is not a retry).
            loop {
                let now = elapsed(start);
                if expired(&deadline, now) {
                    return Err(CallError::DeadlineExpired);
                }
                if limiter.try_acquire_one(now) {
                    break;
                }
                let wait = limiter.retry_after(now, 1).unwrap_or(0).max(50);
                println!("[{now:>4}ms] rate limited; waiting {wait}ms");
                sleep_within(wait, &deadline, start);
            }

            // Short-circuit: wait out the cooldown while the breaker is open.
            loop {
                let now = elapsed(start);
                if expired(&deadline, now) {
                    return Err(CallError::DeadlineExpired);
                }
                if breaker.allow(now) {
                    break;
                }
                println!("[{now:>4}ms] circuit open; waiting for cooldown");
                sleep_within(120, &deadline, start);
            }

            let now = elapsed(start);
            attempt += 1;
            match call_dependency(attempt) {
                Ok(()) => {
                    breaker.on_success();
                    println!("[{now:>4}ms] call ok (circuit {:?})", breaker.state());
                    Ok(())
                }
                Err(e) => {
                    breaker.on_failure(now);
                    println!(
                        "[{now:>4}ms] call failed ({e}); circuit {:?}",
                        breaker.state()
                    );
                    Err(CallError::Unavailable(e))
                }
            }
        },
        // Retry transient failures; stop immediately once the deadline is gone.
        |error| !matches!(error, CallError::DeadlineExpired),
        // Sleep the backoff delay, but never past the deadline (keeps it snappy).
        |delay| sleep_within(delay.as_millis() as u64, &deadline, start),
    );

    let now = elapsed(start);
    match outcome {
        Ok(()) => println!("\nsucceeded after {attempt} attempt(s)"),
        Err(error) => {
            let reason = match error.last_error() {
                CallError::Unavailable(message) => message,
                CallError::DeadlineExpired => "deadline expired",
            };
            println!("\ngave up after {} attempt(s): {reason}", error.attempts());
        }
    }
    println!(
        "final circuit state: {:?}; {}ms of budget left",
        breaker.state(),
        deadline.remaining(now)
    );
}

/// Milliseconds elapsed since `start`, as the `u64` tick the policies use.
fn elapsed(start: Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

/// Whether the overall deadline has run out at `now`.
fn expired(deadline: &Deadline, now: u64) -> bool {
    match deadline.check(now) {
        Some(remaining) => remaining == 0,
        None => true,
    }
}

/// Sleep for `ms`, but never past the deadline (so the example stays snappy).
fn sleep_within(ms: u64, deadline: &Deadline, start: Instant) {
    let now = elapsed(start);
    let capped = ms.min(deadline.remaining(now)).min(300);
    std::thread::sleep(Duration::from_millis(capped));
}

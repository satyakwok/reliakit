//! One call to a flaky dependency, guarded by the whole reliability stack —
//! reached through a single crate name.
//!
//! Four building blocks cooperate, each clock-agnostic and driven by one
//! millisecond clock owned here:
//!
//! - [`reliakit::timeout`] caps the total effort with an overall [`Deadline`].
//! - [`reliakit::ratelimit`] paces calls so we never exceed the budget.
//! - [`reliakit::circuit`] stops hammering a dependency that is already failing.
//! - [`reliakit::backoff`] spaces out the retries.
//!
//! Run it:
//!
//! ```sh
//! cargo run -p reliakit --example resilient_client \
//!   --features "timeout ratelimit circuit backoff"
//! ```
//!
//! [`Deadline`]: reliakit::timeout::Deadline

use std::time::{Duration, Instant};

use reliakit::backoff::Backoff;
use reliakit::circuit::{CircuitBreaker, State};
use reliakit::ratelimit::RateLimiter;
use reliakit::timeout::Timeout;

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
    // Back off 100ms, doubling, capped at 1s.
    let backoff =
        Backoff::exponential(Duration::from_millis(100), 2).with_max_delay(Duration::from_secs(1));

    let start = Instant::now();
    let mut attempt = 0u32;
    let mut retries = 0u32;

    loop {
        let now = start.elapsed().as_millis() as u64;

        // Stop the moment the overall deadline is spent.
        match deadline.check(now) {
            Some(remaining) => {
                if remaining == 0 {
                    println!("[{now:>4}ms] deadline reached; giving up");
                    break;
                }
            }
            None => {
                println!("[{now:>4}ms] deadline expired; giving up");
                break;
            }
        }

        if !limiter.try_acquire_one(now) {
            let wait = limiter.retry_after(now, 1).unwrap_or(0);
            println!("[{now:>4}ms] rate limited; waiting {wait}ms");
            sleep_within(wait.max(50), &deadline, start);
            continue;
        }

        if !breaker.allow(now) {
            println!("[{now:>4}ms] circuit open; skipping call");
            sleep_within(120, &deadline, start);
            continue;
        }

        match call_dependency(attempt) {
            Ok(()) => {
                breaker.on_success();
                println!("[{now:>4}ms] call ok (circuit {:?})", breaker.state());
                if breaker.state() == State::Closed {
                    break;
                }
            }
            Err(e) => {
                breaker.on_failure(now);
                let wait = backoff.delay(retries).unwrap_or_default();
                retries += 1;
                attempt += 1;
                println!(
                    "[{now:>4}ms] call failed ({e}); circuit {:?}; backing off {}ms",
                    breaker.state(),
                    wait.as_millis()
                );
                sleep_within(wait.as_millis() as u64, &deadline, start);
            }
        }
    }

    let now = start.elapsed().as_millis() as u64;
    println!(
        "\nfinal circuit state: {:?}; {}ms of budget left",
        breaker.state(),
        deadline.remaining(now)
    );
}

/// Sleep for `ms`, but never past the deadline (so the example stays snappy).
fn sleep_within(ms: u64, deadline: &reliakit::timeout::Deadline, start: Instant) {
    let now = start.elapsed().as_millis() as u64;
    let capped = ms.min(deadline.remaining(now)).min(300);
    std::thread::sleep(Duration::from_millis(capped));
}

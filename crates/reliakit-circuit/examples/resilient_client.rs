//! The three reliability patterns working together to guard one dependency.
//!
//! - `reliakit-ratelimit` decides whether we are allowed to call right now.
//! - `reliakit-circuit` stops calling a dependency that is already failing.
//! - `reliakit-backoff` spaces out the retries.
//!
//! All three are clock-agnostic: this example owns a single millisecond clock
//! and passes it to each. Run with:
//!
//! ```sh
//! cargo run -p reliakit-circuit --example resilient_client
//! ```

use std::time::Instant;

use reliakit_backoff::Backoff;
use reliakit_circuit::{CircuitBreaker, State};
use reliakit_ratelimit::RateLimiter;

/// A dependency that is down for the first few calls, then recovers.
fn call_dependency(attempt: u32) -> Result<(), &'static str> {
    if attempt < 4 {
        Err("upstream unavailable")
    } else {
        Ok(())
    }
}

fn main() {
    // At most 5 calls per second.
    let mut limiter = RateLimiter::new(5, 1, 200);
    // Trip after 3 consecutive failures; probe again after 500ms.
    let mut breaker = CircuitBreaker::new(3, 500);
    // Back off 100ms, doubling, capped at 1s.
    let backoff = Backoff::exponential(std::time::Duration::from_millis(100), 2)
        .with_max_delay(std::time::Duration::from_secs(1));

    let start = Instant::now();
    let mut attempt = 0u32;
    let mut retries = 0u32;

    for _ in 0..20 {
        let now = start.elapsed().as_millis() as u64;

        if !limiter.try_acquire_one(now) {
            let wait = limiter.retry_after(now, 1).unwrap_or(0);
            println!("[{now:>4}ms] rate limited; retry in {wait}ms");
            std::thread::sleep(std::time::Duration::from_millis(wait.max(50)));
            continue;
        }

        if !breaker.allow(now) {
            println!("[{now:>4}ms] circuit open; skipping call");
            std::thread::sleep(std::time::Duration::from_millis(120));
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
                // Use the backoff policy to decide how long to wait before retrying.
                let wait = backoff.delay(retries).unwrap_or_default();
                retries += 1;
                attempt += 1;
                println!(
                    "[{now:>4}ms] call failed ({e}); circuit {:?}; backing off {}ms",
                    breaker.state(),
                    wait.as_millis()
                );
                std::thread::sleep(wait.min(std::time::Duration::from_millis(300)));
            }
        }
    }

    println!("\nfinal circuit state: {:?}", breaker.state());
}

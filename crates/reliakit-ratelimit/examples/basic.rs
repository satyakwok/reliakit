//! Throttle work with a token bucket.
//!
//! `reliakit-ratelimit` only decides whether a request fits within the rate; you
//! own the clock and the work. Run with:
//!
//! ```sh
//! cargo run -p reliakit-ratelimit --example basic
//! ```

use std::time::Instant;

use reliakit_ratelimit::RateLimiter;

fn main() {
    // Allow bursts of up to 5, refilling 1 token every 200ms.
    let mut limiter = RateLimiter::new(5, 1, 200);
    let start = Instant::now();

    for i in 0..15 {
        let now = start.elapsed().as_millis() as u64;

        if limiter.try_acquire_one(now) {
            println!(
                "[{now:>4}ms] request {i:>2}: allowed ({} left)",
                limiter.available(now)
            );
        } else {
            let wait = limiter.retry_after(now, 1).unwrap_or(0);
            println!("[{now:>4}ms] request {i:>2}: throttled, retry after {wait}ms");
        }

        std::thread::sleep(std::time::Duration::from_millis(80));
    }
}

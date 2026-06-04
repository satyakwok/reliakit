//! Run with: `cargo run -p reliakit-timeout --example timeout_basic`
//!
//! A clock-agnostic deadline driving a small retry loop. Time is a `u64` in
//! whatever monotonic unit you like; this example uses milliseconds and a fake
//! clock so the output is deterministic.

use reliakit_timeout::{Deadline, Timeout};

fn main() {
    // A 1-second budget for the whole operation.
    let policy = Timeout::new(1_000);

    // Pretend the operation starts at t = 0.
    let deadline: Deadline = policy.start(0);
    println!(
        "budget: {} ms, expires at t = {}",
        policy.budget(),
        deadline.expiry()
    );

    // A fake monotonic clock and a fixed retry delay.
    let mut now = 0;
    let attempt_cost = 250; // each attempt + wait advances the clock by this much

    for attempt in 1.. {
        match deadline.check(now) {
            None => {
                println!(
                    "t = {now}: deadline expired, giving up after {} attempts",
                    attempt - 1
                );
                break;
            }
            Some(remaining) => {
                // Never wait longer than the time left in the budget.
                let wait = deadline.clamp(now, attempt_cost);
                println!("t = {now}: attempt {attempt} ({remaining} ms left, waiting {wait} ms)");
                now += attempt_cost;
            }
        }
    }
}

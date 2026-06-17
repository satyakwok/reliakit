//! Drive a retry loop with an exponential backoff policy and full jitter.
//!
//! `reliakit-backoff` only computes delays; this example shows how you decide
//! when to wait and where randomness comes from. Run with:
//!
//! ```sh
//! cargo run -p reliakit-backoff --example basic
//! ```

use std::time::Duration;

use reliakit_backoff::{Backoff, full_jitter};

fn main() {
    // 100ms base, double each attempt, capped at 2s, up to 5 retries.
    let policy = Backoff::exponential(Duration::from_millis(100), 2)
        .with_max_delay(Duration::from_secs(2))
        .with_max_retries(5);

    // A tiny deterministic PRNG stands in for a real RNG so the example output
    // is reproducible. In real code use `rand`, `getrandom`, or a hardware RNG.
    let mut seed: u32 = 0x9e37_79b9;
    let mut next_rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };

    println!("attempt  base-delay  jittered-delay");
    for (attempt, base) in policy.delays().enumerate() {
        let jittered = full_jitter(base, next_rand());
        println!("{attempt:>7}  {base:>10?}  {jittered:>14?}");
        // In a real loop you would: sleep(jittered); if try_operation().is_ok() { break; }
    }

    println!("\nretry limit reached; giving up");
}

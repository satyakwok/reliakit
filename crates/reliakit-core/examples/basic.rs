//! Run with: `cargo run -p reliakit-core --example clock_basic`
//!
//! A `Clock` produces the `u64` tick that Reliakit's clock-agnostic policies
//! expect. `ManualClock` makes that deterministic for tests; `MonotonicClock`
//! reads real monotonic time.

use reliakit_core::{Clock, ManualClock, MonotonicClock};

fn main() {
    // Deterministic clock for tests / demos.
    let clock = ManualClock::new(0);
    println!("manual t = {}", clock.now());
    clock.advance(1_000);
    println!("after advance(1000): t = {}", clock.now());

    // Real monotonic clock (milliseconds since creation).
    let real = MonotonicClock::new();
    let a = real.now();
    let b = real.now();
    println!("monotonic: {a} -> {b} (non-decreasing: {})", b >= a);

    // Any &Clock works wherever a Clock is expected.
    println!("via trait object: {}", read(&clock));
}

fn read(clock: &dyn Clock) -> u64 {
    clock.now()
}

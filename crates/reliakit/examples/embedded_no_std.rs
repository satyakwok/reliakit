//! A bare-metal `no_std` demo: retrying a flaky transmit behind a circuit
//! breaker, driven entirely by a [`ManualClock`] so it needs no real time, no
//! allocator, and no operating system.
//!
//! It builds for a bare-metal target with zero dependencies:
//!
//! ```sh
//! cargo check -p reliakit --example embedded_no_std \
//!   --no-default-features --features "core backoff circuit" \
//!   --target thumbv7em-none-eabi
//! ```
//!
//! and runs on the host, where it prints the timeline:
//!
//! ```sh
//! cargo run -p reliakit --example embedded_no_std --features "core backoff circuit"
//! ```
//!
//! The decision logic in [`run`] is pure `core`: it takes a `log` callback so a
//! host can print and a device can stay silent, and it returns a small `Copy`
//! [`Outcome`]. On a real device, call `run` from your runtime's entry point
//! (this file deliberately ships no `_start`, since that needs a board-specific
//! runtime, which would pull in a dependency).
//!
//! - [`reliakit::core`] supplies [`ManualClock`], the clock every decision reads.
//! - [`reliakit::backoff`] computes the delay between attempts.
//! - [`reliakit::circuit`] stops hammering a link that is already failing.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

use core::time::Duration;

use reliakit::backoff::Backoff;
use reliakit::circuit::CircuitBreaker;
use reliakit::core::{Clock, ManualClock};

/// What one transmit run achieved.
// On a bare-metal target nothing consumes the returned value (there is no entry
// point here), so the fields read only on the host would warn.
#[cfg_attr(target_os = "none", allow(dead_code))]
#[derive(Clone, Copy)]
struct Outcome {
    /// How many real send attempts were made.
    attempts: u32,
    /// Whether the payload was finally accepted.
    sent: bool,
}

/// The flaky link: it drops the first two frames, then accepts.
fn transmit(attempt: u32) -> bool {
    attempt >= 3
}

/// Drives one transmit through backoff + circuit, reporting each step through
/// `log`. Pure `core`: no allocation, no real clock, no I/O.
#[cfg_attr(target_os = "none", allow(dead_code))]
fn run(mut log: impl FnMut(&str)) -> Outcome {
    let clock = ManualClock::new(0);
    let backoff = Backoff::exponential(Duration::from_millis(50), 2)
        .with_max_delay(Duration::from_millis(400));
    let mut breaker = CircuitBreaker::new(2, 200);

    let mut attempts = 0u32;
    loop {
        // Wait out the cooldown while the breaker is open.
        if !breaker.allow(clock.now()) {
            log("circuit open: waiting for cooldown");
            clock.advance(200);
            continue;
        }

        let delay = match backoff.delay(attempts) {
            Some(delay) => delay,
            None => {
                log("retry budget exhausted");
                return Outcome {
                    attempts,
                    sent: false,
                };
            }
        };

        attempts += 1;
        if transmit(attempts) {
            breaker.on_success();
            log("sent ok");
            return Outcome {
                attempts,
                sent: true,
            };
        }

        breaker.on_failure(clock.now());
        log("send failed");
        clock.advance(saturating_millis(delay));
    }
}

/// `Duration` to a `u64` millisecond tick, saturating.
fn saturating_millis(d: Duration) -> u64 {
    d.as_millis().min(u64::MAX as u128) as u64
}

#[cfg(not(target_os = "none"))]
fn main() {
    let outcome = run(|step| println!("  {step}"));
    println!(
        "outcome: {} after {} attempt(s)",
        if outcome.sent { "sent" } else { "gave up" },
        outcome.attempts
    );
}

// On a bare-metal target there is no std panic machinery, so the binary must
// supply its own. A spin loop is enough for a demo; a real device would reset or
// log. No unsafe and no third-party runtime are involved.
#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

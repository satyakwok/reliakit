//! Shared building blocks for the Reliakit workspace.
//!
//! Reliakit's time-driven resilience crates ([`reliakit-circuit`],
//! [`reliakit-ratelimit`], [`reliakit-timeout`]) are *clock-agnostic*: you pass
//! the current time in as a `u64` tick in any monotonic unit you choose. This
//! crate provides the small piece they have in common, a [`Clock`] trait and a
//! couple of ready-made clocks, so you do not have to hand-roll one. Each of
//! those crates has an optional `core` feature that adds `*_now(clock)` methods
//! backed by this trait. ([`reliakit-backoff`] computes delays from an attempt
//! number and does not read a clock.)
//!
//! - [`ManualClock`], a settable clock for deterministic tests; `no_std`.
//! - [`MonotonicClock`], wall-free monotonic milliseconds (requires `std`).
//!
//! The crate has no dependencies and forbids unsafe code. The `Clock` trait and
//! [`ManualClock`] are available on `no_std`; [`MonotonicClock`] needs the
//! default `std` feature.
//!
//! # Example
//!
//! ```
//! use reliakit_core::{Clock, ManualClock};
//!
//! let clock = ManualClock::new(0);
//! assert_eq!(clock.now(), 0);
//! clock.advance(250);
//! assert_eq!(clock.now(), 250);
//!
//! // Feed `clock.now()` into any clock-agnostic Reliakit policy.
//! fn elapsed_since<C: Clock>(clock: &C, start: u64) -> u64 {
//!     clock.now().saturating_sub(start)
//! }
//! assert_eq!(elapsed_since(&clock, 100), 150);
//! ```
//!
//! [`reliakit-backoff`]: https://docs.rs/reliakit-backoff
//! [`reliakit-circuit`]: https://docs.rs/reliakit-circuit
//! [`reliakit-ratelimit`]: https://docs.rs/reliakit-ratelimit
//! [`reliakit-timeout`]: https://docs.rs/reliakit-timeout

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::cell::Cell;

/// A source of monotonic time, expressed as a `u64` tick.
///
/// The unit is whatever the clock uses (milliseconds is typical) and must match
/// the unit you pass to the resilience policies. Implementations should be
/// monotonic non-decreasing, though Reliakit's policies all saturate and so do
/// not panic if a clock moves backwards.
pub trait Clock {
    /// Returns the current time as a `u64` tick.
    fn now(&self) -> u64;
}

impl<C: Clock + ?Sized> Clock for &C {
    fn now(&self) -> u64 {
        (**self).now()
    }
}

/// A clock whose time is set explicitly, for deterministic tests.
///
/// Uses interior mutability so [`now`](Clock::now), [`set`](Self::set), and
/// [`advance`](Self::advance) all take `&self`.
#[derive(Debug, Default)]
pub struct ManualClock {
    tick: Cell<u64>,
}

impl ManualClock {
    /// Creates a clock reading `start`.
    pub const fn new(start: u64) -> Self {
        Self {
            tick: Cell::new(start),
        }
    }

    /// Sets the clock to `tick`.
    pub fn set(&self, tick: u64) {
        self.tick.set(tick);
    }

    /// Advances the clock by `delta` (saturating).
    pub fn advance(&self, delta: u64) {
        self.tick.set(self.tick.get().saturating_add(delta));
    }
}

impl Clock for ManualClock {
    fn now(&self) -> u64 {
        self.tick.get()
    }
}

/// A monotonic clock measuring milliseconds since it was created.
///
/// Backed by [`std::time::Instant`], so it never goes backwards and is immune to
/// wall-clock adjustments. Requires the `std` feature.
#[cfg(feature = "std")]
#[derive(Debug, Clone)]
pub struct MonotonicClock {
    origin: std::time::Instant,
}

#[cfg(feature = "std")]
impl MonotonicClock {
    /// Creates a clock whose zero point is now.
    pub fn new() -> Self {
        Self {
            origin: std::time::Instant::now(),
        }
    }
}

#[cfg(feature = "std")]
impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl Clock for MonotonicClock {
    fn now(&self) -> u64 {
        // Milliseconds since creation, saturating rather than wrapping on the
        // (astronomically unlikely) overflow of a u64 millisecond count.
        let millis = self.origin.elapsed().as_millis();
        if millis > u64::MAX as u128 {
            u64::MAX
        } else {
            millis as u64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_reads_sets_and_advances() {
        let clock = ManualClock::new(10);
        assert_eq!(clock.now(), 10);
        clock.advance(5);
        assert_eq!(clock.now(), 15);
        clock.set(100);
        assert_eq!(clock.now(), 100);
    }

    #[test]
    fn manual_clock_advance_saturates() {
        let clock = ManualClock::new(u64::MAX - 1);
        clock.advance(10);
        assert_eq!(clock.now(), u64::MAX);
    }

    #[test]
    fn manual_clock_default_is_zero() {
        assert_eq!(ManualClock::default().now(), 0);
    }

    #[test]
    fn clock_is_object_safe_and_reference_forwards() {
        let clock = ManualClock::new(7);
        let by_ref: &dyn Clock = &clock;
        assert_eq!(by_ref.now(), 7);
        // The blanket impl on `&C` lets a `&ManualClock` be used as a `Clock`.
        fn read(c: impl Clock) -> u64 {
            c.now()
        }
        assert_eq!(read(&clock), 7);
    }

    #[cfg(feature = "std")]
    #[test]
    fn monotonic_clock_is_non_decreasing() {
        let clock = MonotonicClock::new();
        let a = clock.now();
        let b = clock.now();
        assert!(b >= a);
        assert!(MonotonicClock::default().now() < u64::MAX);
    }
}

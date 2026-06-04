//! Clock-agnostic retry backoff policies.
//!
//! `reliakit-backoff` computes *how long to wait* between retries. It does not
//! sleep, spawn tasks, or touch the clock — you decide when to call it and how
//! to wait. That makes it usable from sync code, any async runtime, and
//! `no_std` / embedded contexts, with deterministic, exact-byte-style tests.
//!
//! The core type is [`Backoff`]: a small, `Copy` policy describing a base delay,
//! a growth strategy (constant, linear, or exponential), an optional maximum
//! delay, and an optional retry limit. [`Backoff::delay`] maps a zero-based
//! attempt number to the delay to wait before that retry, or `None` once the
//! retry limit is reached.
//!
//! Randomized jitter is provided as explicit, pure functions
//! ([`full_jitter`], [`equal_jitter`]) that take a caller-supplied random value,
//! so the crate stays dependency-free and its output stays deterministic in
//! tests.
//!
//! # Example
//!
//! ```
//! use core::time::Duration;
//! use reliakit_backoff::Backoff;
//!
//! // 100ms base, double each attempt, capped at 2s, give up after 8 retries.
//! let policy = Backoff::exponential(Duration::from_millis(100), 2)
//!     .with_max_delay(Duration::from_secs(2))
//!     .with_max_retries(8);
//!
//! assert_eq!(policy.delay(0), Some(Duration::from_millis(100)));
//! assert_eq!(policy.delay(1), Some(Duration::from_millis(200)));
//! assert_eq!(policy.delay(4), Some(Duration::from_millis(1600)));
//! assert_eq!(policy.delay(5), Some(Duration::from_secs(2))); // 3200ms capped to 2s
//! assert_eq!(policy.delay(8), None); // retry limit reached
//!
//! // Drive your own retry loop (attempts 0..8 here):
//! for delay in policy.delays() {
//!     // sleep(delay); try_again()?; ...
//!     let _ = delay;
//!     # break;
//! }
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod jitter;

pub use jitter::{equal_jitter, full_jitter};

use core::time::Duration;

/// Growth strategy for the delay between retries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Kind {
    /// Same delay every attempt.
    Constant,
    /// `base + step * attempt`.
    Linear { step: Duration },
    /// `base * factor^attempt`.
    Exponential { factor: u32 },
}

/// A retry backoff policy.
///
/// `Backoff` is a small, `Copy` value. Construct one with [`Backoff::constant`],
/// [`Backoff::linear`], or [`Backoff::exponential`], then optionally cap it with
/// [`Backoff::with_max_delay`] and [`Backoff::with_max_retries`].
///
/// [`Backoff::delay`] takes a zero-based attempt number (attempt `0` is the wait
/// before the first retry) and returns the delay, or `None` when the retry limit
/// has been reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Backoff {
    base: Duration,
    kind: Kind,
    max_delay: Duration,
    max_retries: Option<u32>,
}

impl Backoff {
    /// A constant backoff that always waits `base`.
    pub const fn constant(base: Duration) -> Self {
        Self {
            base,
            kind: Kind::Constant,
            max_delay: Duration::MAX,
            max_retries: None,
        }
    }

    /// A linear backoff: attempt `n` waits `base + step * n`.
    pub const fn linear(base: Duration, step: Duration) -> Self {
        Self {
            base,
            kind: Kind::Linear { step },
            max_delay: Duration::MAX,
            max_retries: None,
        }
    }

    /// An exponential backoff: attempt `n` waits `base * factor^n`.
    ///
    /// `factor` is the integer multiplier (e.g. `2` doubles each attempt). A
    /// `factor` below `2` makes the policy behave like [`Backoff::constant`].
    pub const fn exponential(base: Duration, factor: u32) -> Self {
        Self {
            base,
            kind: Kind::Exponential { factor },
            max_delay: Duration::MAX,
            max_retries: None,
        }
    }

    /// Caps every computed delay at `max_delay`.
    pub const fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }

    /// Limits the policy to at most `max_retries` retries.
    ///
    /// After this, [`delay`](Self::delay) returns `None` for attempt numbers
    /// `>= max_retries`.
    pub const fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = Some(max_retries);
        self
    }

    /// Returns the base delay.
    pub const fn base(&self) -> Duration {
        self.base
    }

    /// Returns the maximum delay cap.
    pub const fn max_delay(&self) -> Duration {
        self.max_delay
    }

    /// Returns the retry limit, if any.
    pub const fn max_retries(&self) -> Option<u32> {
        self.max_retries
    }

    /// Returns the delay to wait before retry `attempt` (zero-based), or `None`
    /// if the retry limit has been reached.
    ///
    /// The result is always clamped to [`max_delay`](Self::max_delay). All
    /// arithmetic saturates and the computation runs in bounded time, so large
    /// attempt numbers never overflow, panic, or hang.
    pub fn delay(&self, attempt: u32) -> Option<Duration> {
        if let Some(max) = self.max_retries {
            if attempt >= max {
                return None;
            }
        }

        let raw = match self.kind {
            Kind::Constant => self.base,
            Kind::Linear { step } => self.base.saturating_add(step.saturating_mul(attempt)),
            Kind::Exponential { factor } => {
                if factor <= 1 {
                    self.base
                } else {
                    let mut delay = self.base;
                    let mut i = 0;
                    while i < attempt {
                        if delay >= self.max_delay {
                            delay = self.max_delay;
                            break;
                        }
                        let next = delay.saturating_mul(factor);
                        if next == delay {
                            // No further growth possible (zero base or the
                            // delay has saturated): stop instead of looping the
                            // full attempt count.
                            break;
                        }
                        delay = next;
                        i += 1;
                    }
                    delay
                }
            }
        };

        Some(if raw > self.max_delay {
            self.max_delay
        } else {
            raw
        })
    }

    /// Returns an iterator over the delays for attempts `0, 1, 2, ...`.
    ///
    /// The iterator ends when the retry limit is reached. With no retry limit it
    /// is infinite.
    pub const fn delays(&self) -> Delays {
        Delays {
            backoff: *self,
            attempt: 0,
        }
    }
}

/// Iterator over the successive delays of a [`Backoff`] policy.
///
/// Created by [`Backoff::delays`].
#[derive(Debug, Clone)]
pub struct Delays {
    backoff: Backoff,
    attempt: u32,
}

impl Iterator for Delays {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        let delay = self.backoff.delay(self.attempt)?;
        self.attempt = self.attempt.saturating_add(1);
        Some(delay)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn constant_is_flat() {
        let b = Backoff::constant(MS(50));
        assert_eq!(b.delay(0), Some(MS(50)));
        assert_eq!(b.delay(1000), Some(MS(50)));
    }

    #[test]
    fn linear_grows_by_step() {
        let b = Backoff::linear(MS(100), MS(25));
        assert_eq!(b.delay(0), Some(MS(100)));
        assert_eq!(b.delay(1), Some(MS(125)));
        assert_eq!(b.delay(4), Some(MS(200)));
    }

    #[test]
    fn exponential_doubles() {
        let b = Backoff::exponential(MS(10), 2);
        assert_eq!(b.delay(0), Some(MS(10)));
        assert_eq!(b.delay(1), Some(MS(20)));
        assert_eq!(b.delay(2), Some(MS(40)));
        assert_eq!(b.delay(3), Some(MS(80)));
    }

    #[test]
    fn exponential_factor_three() {
        let b = Backoff::exponential(MS(1), 3);
        assert_eq!(b.delay(0), Some(MS(1)));
        assert_eq!(b.delay(1), Some(MS(3)));
        assert_eq!(b.delay(2), Some(MS(9)));
    }

    #[test]
    fn max_delay_caps() {
        let b = Backoff::exponential(MS(100), 2).with_max_delay(MS(500));
        assert_eq!(b.delay(0), Some(MS(100)));
        assert_eq!(b.delay(2), Some(MS(400)));
        assert_eq!(b.delay(3), Some(MS(500))); // 800 capped to 500
        assert_eq!(b.delay(50), Some(MS(500)));
    }

    #[test]
    fn max_retries_stops() {
        let b = Backoff::constant(MS(10)).with_max_retries(3);
        assert_eq!(b.delay(0), Some(MS(10)));
        assert_eq!(b.delay(2), Some(MS(10)));
        assert_eq!(b.delay(3), None);
        assert_eq!(b.delay(100), None);
    }

    #[test]
    fn exponential_factor_below_two_is_constant() {
        let b = Backoff::exponential(MS(7), 1);
        assert_eq!(b.delay(0), Some(MS(7)));
        assert_eq!(b.delay(9), Some(MS(7)));
    }

    #[test]
    fn huge_attempt_saturates_without_hanging() {
        // No max_delay: must saturate quickly, not loop billions of times.
        let b = Backoff::exponential(Duration::from_secs(1), 2);
        assert_eq!(b.delay(u32::MAX), Some(Duration::MAX));
    }

    #[test]
    fn zero_base_exponential_does_not_hang() {
        // base == ZERO never grows; delay(huge) must return instantly, not loop
        // the full attempt count.
        let b = Backoff::exponential(Duration::ZERO, 2);
        assert_eq!(b.delay(u32::MAX), Some(Duration::ZERO));
        assert_eq!(b.delay(0), Some(Duration::ZERO));

        // Same guard with a max_delay set.
        let capped = Backoff::exponential(Duration::ZERO, 4).with_max_delay(MS(500));
        assert_eq!(capped.delay(u32::MAX), Some(Duration::ZERO));
    }

    #[test]
    fn linear_saturates() {
        let b = Backoff::linear(Duration::MAX, Duration::from_secs(1));
        assert_eq!(b.delay(10), Some(Duration::MAX));
    }

    #[test]
    fn delays_iterator_respects_limit() {
        let b = Backoff::exponential(MS(10), 2).with_max_retries(4);
        let collected: heapless_vec::Vec = b.delays().collect();
        assert_eq!(collected.as_slice(), &[MS(10), MS(20), MS(40), MS(80)]);
    }

    #[test]
    fn getters() {
        let b = Backoff::exponential(MS(5), 2)
            .with_max_delay(MS(100))
            .with_max_retries(7);
        assert_eq!(b.base(), MS(5));
        assert_eq!(b.max_delay(), MS(100));
        assert_eq!(b.max_retries(), Some(7));
    }

    // Tiny fixed-capacity collector so tests stay `no_std` without `alloc`.
    mod heapless_vec {
        use core::time::Duration;

        pub struct Vec {
            data: [Duration; 16],
            len: usize,
        }

        impl Vec {
            pub fn as_slice(&self) -> &[Duration] {
                &self.data[..self.len]
            }
        }

        impl FromIterator<Duration> for Vec {
            fn from_iter<I: IntoIterator<Item = Duration>>(iter: I) -> Self {
                let mut data = [Duration::ZERO; 16];
                let mut len = 0;
                for d in iter {
                    data[len] = d;
                    len += 1;
                }
                Self { data, len }
            }
        }
    }
}

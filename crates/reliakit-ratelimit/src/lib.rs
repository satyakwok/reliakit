//! Clock-agnostic token-bucket rate limiter.
//!
//! A token bucket holds up to `capacity` tokens and refills `refill_amount`
//! tokens every `refill_interval` time units. Each request takes one or more
//! tokens; when the bucket is empty, requests are denied until it refills. The
//! `capacity` sets the largest burst you will allow; the refill rate sets the
//! sustained throughput.
//!
//! [`RateLimiter`] is a small, `Copy` value. It does **not** read the clock,
//! sleep, or allocate; you pass the current time in on each call as a plain
//! `u64` in whatever monotonic unit you choose (milliseconds is typical), and
//! the intervals use that same unit. That keeps it usable from synchronous
//! code, any async runtime, and `no_std` / embedded targets, and makes every
//! decision deterministic and easy to test. All arithmetic is integer-only and
//! saturating, so no call ever overflows or panics.
//!
//! # Example
//!
//! ```
//! use reliakit_ratelimit::RateLimiter;
//!
//! // Allow bursts of up to 10, refilling 1 token every 100ms (~10/sec).
//! let mut limiter = RateLimiter::new(10, 1, 100);
//!
//! // The bucket starts full, so a burst of 10 is allowed immediately.
//! for _ in 0..10 {
//!     assert!(limiter.try_acquire_one(0));
//! }
//! // The 11th is denied until the bucket refills.
//! assert!(!limiter.try_acquire_one(0));
//! assert_eq!(limiter.retry_after(0, 1), Some(100)); // one token in 100ms
//!
//! // After 100ms exactly one token is back.
//! assert!(limiter.try_acquire_one(100));
//! assert!(!limiter.try_acquire_one(100));
//! ```
//!
//! # Feature flags
//!
//! - `core` (off by default) adds `*_now(clock)` convenience methods on
//!   [`RateLimiter`] that read the time from a `reliakit_core::Clock`. It pulls
//!   in `reliakit-core` (`no_std`, zero third-party dependencies); the
//!   `now: u64` methods remain the primitive API.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::cmp::min;

/// A token-bucket rate limiter.
///
/// Construct one with [`RateLimiter::new`], then gate work with
/// [`try_acquire`](Self::try_acquire) / [`try_acquire_one`](Self::try_acquire_one).
/// The bucket starts full (a burst of up to `capacity` is allowed immediately).
///
/// Time is a plain `u64` in any monotonic unit you choose (commonly
/// milliseconds); `refill_interval` uses that same unit. The limiter never reads
/// the clock itself; pass `now` to each method.
///
/// `RateLimiter` is not internally synchronized. Share one across threads by
/// wrapping it in your own `Mutex`/lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimiter {
    capacity: u64,
    refill_amount: u64,
    refill_interval: u64,
    tokens: u64,
    last_refill: u64,
}

impl RateLimiter {
    /// Creates a limiter that holds up to `capacity` tokens and adds
    /// `refill_amount` tokens every `refill_interval` time units.
    ///
    /// The bucket starts full. `capacity`, `refill_amount`, and
    /// `refill_interval` are each clamped to a minimum of `1` (a zero interval
    /// would never refill and would divide by zero).
    pub const fn new(capacity: u64, refill_amount: u64, refill_interval: u64) -> Self {
        let capacity = if capacity == 0 { 1 } else { capacity };
        Self {
            capacity,
            refill_amount: if refill_amount == 0 { 1 } else { refill_amount },
            refill_interval: if refill_interval == 0 {
                1
            } else {
                refill_interval
            },
            tokens: capacity,
            last_refill: 0,
        }
    }

    /// Returns the bucket capacity (the maximum burst).
    pub const fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns how many tokens are added on each refill.
    pub const fn refill_amount(&self) -> u64 {
        self.refill_amount
    }

    /// Returns the refill interval, in the caller's time unit.
    pub const fn refill_interval(&self) -> u64 {
        self.refill_interval
    }

    /// Refills the bucket for any whole intervals elapsed since the last refill.
    fn refill(&mut self, now: u64) {
        let elapsed = now.saturating_sub(self.last_refill);
        let batches = elapsed / self.refill_interval;
        if batches == 0 {
            return;
        }
        let added = batches.saturating_mul(self.refill_amount);
        self.tokens = min(self.capacity, self.tokens.saturating_add(added));
        self.last_refill = self
            .last_refill
            .saturating_add(batches.saturating_mul(self.refill_interval));
    }

    /// Returns the number of tokens available at `now`, after refilling.
    pub fn available(&mut self, now: u64) -> u64 {
        self.refill(now);
        self.tokens
    }

    /// Tries to take `tokens` tokens at `now`. Returns `true` and consumes them
    /// if enough are available, otherwise returns `false` and consumes nothing.
    ///
    /// A request for more than `capacity` tokens can never succeed.
    pub fn try_acquire(&mut self, now: u64, tokens: u64) -> bool {
        self.refill(now);
        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Tries to take a single token at `now`.
    pub fn try_acquire_one(&mut self, now: u64) -> bool {
        self.try_acquire(now, 1)
    }

    /// Returns how long to wait, from `now`, until `tokens` tokens are available.
    ///
    /// Returns `Some(0)` if they are available now, and `None` if `tokens`
    /// exceeds `capacity` (the bucket can never hold that many). Useful for a
    /// `Retry-After` hint. This refills the bucket but does not consume anything.
    pub fn retry_after(&mut self, now: u64, tokens: u64) -> Option<u64> {
        if tokens > self.capacity {
            return None;
        }
        self.refill(now);
        if self.tokens >= tokens {
            return Some(0);
        }
        let deficit = tokens - self.tokens;
        let batches = deficit.div_ceil(self.refill_amount);
        let into_interval = now.saturating_sub(self.last_refill);
        let time_to_first = self.refill_interval.saturating_sub(into_interval);
        Some(
            time_to_first.saturating_add(
                batches
                    .saturating_sub(1)
                    .saturating_mul(self.refill_interval),
            ),
        )
    }
}

/// Convenience methods that read the current time from a
/// [`Clock`](reliakit_core::Clock) instead of taking an explicit `now: u64`.
///
/// Available with the `core` feature. Each forwards to the matching `now`-taking
/// method, which remains the primitive API.
#[cfg(feature = "core")]
impl RateLimiter {
    /// Like [`available`](Self::available), reading the time from `clock`.
    ///
    /// ```
    /// use reliakit_ratelimit::RateLimiter;
    /// use reliakit_core::ManualClock;
    ///
    /// let clock = ManualClock::new(0);
    /// let mut rl = RateLimiter::new(10, 1, 100);
    /// assert!(rl.try_acquire_now(&clock, 10)); // drain the bucket
    /// assert!(!rl.try_acquire_one_now(&clock)); // empty now
    /// clock.set(100); // one refill interval later
    /// assert!(rl.try_acquire_one_now(&clock)); // one token refilled
    /// ```
    pub fn available_now<C: reliakit_core::Clock>(&mut self, clock: &C) -> u64 {
        self.available(clock.now())
    }

    /// Like [`try_acquire`](Self::try_acquire), reading the time from `clock`.
    pub fn try_acquire_now<C: reliakit_core::Clock>(&mut self, clock: &C, tokens: u64) -> bool {
        self.try_acquire(clock.now(), tokens)
    }

    /// Like [`try_acquire_one`](Self::try_acquire_one), reading the time from `clock`.
    pub fn try_acquire_one_now<C: reliakit_core::Clock>(&mut self, clock: &C) -> bool {
        self.try_acquire_one(clock.now())
    }

    /// Like [`retry_after`](Self::retry_after), reading the time from `clock`.
    pub fn retry_after_now<C: reliakit_core::Clock>(
        &mut self,
        clock: &C,
        tokens: u64,
    ) -> Option<u64> {
        self.retry_after(clock.now(), tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_full() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert_eq!(rl.available(0), 10);
        assert_eq!(rl.capacity(), 10);
    }

    #[test]
    fn acquire_consumes_tokens() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert!(rl.try_acquire(0, 4));
        assert_eq!(rl.available(0), 6);
    }

    #[test]
    fn burst_then_denied() {
        let mut rl = RateLimiter::new(10, 1, 100);
        for _ in 0..10 {
            assert!(rl.try_acquire_one(0));
        }
        assert!(!rl.try_acquire_one(0));
    }

    #[test]
    fn refills_over_time_capped_at_capacity() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert!(rl.try_acquire(0, 10));
        assert_eq!(rl.available(0), 0);
        assert_eq!(rl.available(250), 2); // two whole 100-unit intervals
        assert_eq!(rl.available(10_000), 10); // capped at capacity, not 100
    }

    #[test]
    fn partial_interval_does_not_refill_and_keeps_remainder() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert!(rl.try_acquire(0, 10));
        assert_eq!(rl.available(50), 0); // 50 < 100, no token yet
        assert_eq!(rl.available(100), 1); // the 50 remainder is preserved
    }

    #[test]
    fn refill_amount_greater_than_one() {
        let mut rl = RateLimiter::new(100, 5, 10);
        assert!(rl.try_acquire(0, 100));
        assert_eq!(rl.available(30), 15); // 3 intervals * 5
    }

    #[test]
    fn acquire_more_than_capacity_always_fails() {
        let mut rl = RateLimiter::new(5, 1, 100);
        assert!(!rl.try_acquire(0, 6));
        assert_eq!(rl.available(0), 5); // nothing consumed
    }

    #[test]
    fn retry_after_zero_when_available() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert_eq!(rl.retry_after(0, 5), Some(0));
    }

    #[test]
    fn retry_after_none_when_above_capacity() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert_eq!(rl.retry_after(0, 11), None);
    }

    #[test]
    fn retry_after_full_interval_when_empty() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert!(rl.try_acquire(0, 10));
        assert_eq!(rl.retry_after(0, 1), Some(100));
        assert_eq!(rl.retry_after(0, 3), Some(300));
    }

    #[test]
    fn retry_after_accounts_for_elapsed_partial_interval() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert!(rl.try_acquire(0, 10));
        // 60 units into the first interval: one token arrives in 40 more.
        assert_eq!(rl.retry_after(60, 1), Some(40));
        assert_eq!(rl.retry_after(60, 2), Some(140));
    }

    #[test]
    fn backwards_clock_does_not_panic_or_refill() {
        let mut rl = RateLimiter::new(10, 1, 100);
        assert!(rl.try_acquire(10_000, 10));
        assert!(!rl.try_acquire(5_000, 1)); // earlier time: no refill, still empty
        assert_eq!(rl.available(5_000), 0);
    }

    #[test]
    fn zero_config_is_clamped() {
        let rl = RateLimiter::new(0, 0, 0);
        assert_eq!(rl.capacity(), 1);
        assert_eq!(rl.refill_amount(), 1);
        assert_eq!(rl.refill_interval(), 1);
    }
}

#[cfg(all(test, feature = "core"))]
mod core_tests {
    use super::*;
    use reliakit_core::ManualClock;

    #[test]
    fn now_methods_match_explicit_now() {
        let clock = ManualClock::new(0);
        let mut viaclock = RateLimiter::new(10, 2, 100);
        let mut explicit = RateLimiter::new(10, 2, 100);

        assert_eq!(viaclock.available_now(&clock), explicit.available(0));
        assert_eq!(
            viaclock.try_acquire_now(&clock, 7),
            explicit.try_acquire(0, 7)
        );
        assert_eq!(
            viaclock.retry_after_now(&clock, 5),
            explicit.retry_after(0, 5)
        );

        clock.set(250);
        assert_eq!(
            viaclock.try_acquire_one_now(&clock),
            explicit.try_acquire_one(250)
        );
        assert_eq!(viaclock.available_now(&clock), explicit.available(250));
    }
}

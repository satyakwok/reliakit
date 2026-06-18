//! Clock-agnostic deadlines and timeouts.
//!
//! `reliakit-timeout` answers one question: *has my time budget run out, and how
//! much is left?* It does not read the clock, sleep, or spawn anything; you
//! capture a start instant and a budget, then pass `now` to the query methods.
//! That makes it usable from sync code, any async runtime, and `no_std` /
//! embedded contexts, with deterministic tests.
//!
//! Time is a plain `u64` in any monotonic unit you choose (milliseconds is
//! typical), matching [`reliakit-circuit`] and [`reliakit-ratelimit`]. All
//! arithmetic saturates, so no method panics: not on overflow, and not on a
//! clock that moves backwards.
//!
//! Two small types:
//!
//! - [`Timeout`] is a reusable budget that is not yet pinned to a timeline.
//!   Configure it once, then call [`Timeout::start`] per operation.
//! - [`Deadline`] is a budget pinned to a start instant. Query it with
//!   [`remaining`](Deadline::remaining), [`is_expired`](Deadline::is_expired),
//!   [`check`](Deadline::check), and friends.
//!
//! # Example
//!
//! ```
//! use reliakit_timeout::{Deadline, Timeout};
//!
//! // A 30s budget (here in milliseconds), pinned to the start of the operation.
//! let policy = Timeout::new(30_000);
//! let deadline = policy.start(1_000); // started at t = 1_000
//!
//! assert_eq!(deadline.remaining(1_000), 30_000);
//! assert_eq!(deadline.remaining(21_000), 10_000);
//! assert!(!deadline.is_expired(30_999));
//! assert!(deadline.is_expired(31_000)); // expiry is inclusive
//!
//! // Not yet expired -> Some(remaining); expired -> None.
//! assert_eq!(deadline.check(21_000), Some(10_000));
//! assert_eq!(deadline.check(40_000), None);
//! ```
//!
//! # Composing with backoff
//!
//! Use [`Deadline::clamp`] to keep a retry delay from running past the budget,
//! and [`Deadline::is_expired`] to stop retrying:
//!
//! ```
//! use reliakit_timeout::Deadline;
//!
//! let deadline = Deadline::new(0, 1_000);
//! let proposed_backoff = 800; // ms the backoff policy wants to wait
//!
//! let now = 500;
//! if deadline.is_expired(now) {
//!     // give up
//! } else {
//!     let wait = deadline.clamp(now, proposed_backoff); // min(800, 500 left) = 500
//!     assert_eq!(wait, 500);
//! }
//! ```
//!
//! [`reliakit-circuit`]: https://docs.rs/reliakit-circuit
//! [`reliakit-ratelimit`]: https://docs.rs/reliakit-ratelimit
//!
//! # Feature flags
//!
//! - `core` (off by default) adds `*_now(clock)` convenience methods on
//!   [`Timeout`] and [`Deadline`] that read the time from a
//!   `reliakit_core::Clock`. It pulls in `reliakit-core` (`no_std`, zero
//!   third-party dependencies); the `now: u64` methods remain the primitive API.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// A reusable timeout budget that is not yet pinned to a timeline.
///
/// A `Timeout` is just a length (in your chosen monotonic unit). Configure it
/// once and call [`start`](Self::start) per operation to get a [`Deadline`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Timeout {
    budget: u64,
}

impl Timeout {
    /// Creates a timeout with the given `budget` (its length).
    pub const fn new(budget: u64) -> Self {
        Self { budget }
    }

    /// The budget (length) of this timeout.
    pub const fn budget(&self) -> u64 {
        self.budget
    }

    /// Pins this timeout to the timeline, starting at `now`.
    pub const fn start(&self, now: u64) -> Deadline {
        Deadline::new(now, self.budget)
    }
}

/// A time budget pinned to a monotonic timeline.
///
/// A `Deadline` is a `start` instant plus a `budget`; it expires at
/// `start + budget`. It never reads the clock; pass `now` to the query
/// methods. All arithmetic saturates, so a backwards-moving clock or an
/// overflowing `start + budget` cannot panic.
///
/// A zero budget expires immediately at `start`. For the same reason,
/// [`Deadline::default`] (`start` and `budget` both `0`) is already expired;
/// it is not an "infinite" deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Deadline {
    start: u64,
    budget: u64,
}

impl Deadline {
    /// Creates a deadline that expires `budget` units after `start`.
    pub const fn new(start: u64, budget: u64) -> Self {
        Self { start, budget }
    }

    /// The start instant.
    pub const fn start(&self) -> u64 {
        self.start
    }

    /// The budget (the length of the deadline).
    pub const fn budget(&self) -> u64 {
        self.budget
    }

    /// The instant the deadline expires, i.e. `start + budget` (saturating).
    pub const fn expiry(&self) -> u64 {
        self.start.saturating_add(self.budget)
    }

    /// Time elapsed since `start` at `now`.
    ///
    /// Saturates to `0` when `now` is before `start`.
    pub const fn elapsed(&self, now: u64) -> u64 {
        now.saturating_sub(self.start)
    }

    /// Time left until expiry at `now`.
    ///
    /// Saturates to `0` once the deadline has expired.
    pub const fn remaining(&self, now: u64) -> u64 {
        self.expiry().saturating_sub(now)
    }

    /// Whether the deadline has expired at `now` (`now >= expiry`).
    pub const fn is_expired(&self, now: u64) -> bool {
        now >= self.expiry()
    }

    /// Returns the remaining time if the deadline is still live at `now`, or
    /// `None` once it has expired.
    pub const fn check(&self, now: u64) -> Option<u64> {
        if self.is_expired(now) {
            None
        } else {
            Some(self.remaining(now))
        }
    }

    /// Whether an operation that needs `duration` units can finish before the
    /// deadline at `now` (`remaining(now) >= duration`).
    ///
    /// A `duration` of `0` is always allowed, even once the deadline has
    /// expired.
    pub const fn allows(&self, now: u64, duration: u64) -> bool {
        self.remaining(now) >= duration
    }

    /// Caps `duration` so it does not run past the deadline: the smaller of
    /// `duration` and [`remaining`](Self::remaining) at `now`.
    ///
    /// Handy for bounding a backoff delay by the time left in the budget.
    pub const fn clamp(&self, now: u64, duration: u64) -> u64 {
        let left = self.remaining(now);
        if duration < left { duration } else { left }
    }
}

/// Convenience methods that read the current time from a
/// [`Clock`](reliakit_core::Clock) instead of taking an explicit `now: u64`.
///
/// Available with the `core` feature. Each forwards to the matching `now`-taking
/// method, which remains the primitive API.
#[cfg(feature = "core")]
impl Timeout {
    /// Like [`start`](Self::start), reading the start instant from `clock`.
    ///
    /// ```
    /// use reliakit_timeout::Timeout;
    /// use reliakit_core::ManualClock;
    ///
    /// let clock = ManualClock::new(1_000);
    /// let deadline = Timeout::new(30_000).start_now(&clock);
    /// clock.advance(10_000);
    /// assert_eq!(deadline.remaining_now(&clock), 20_000);
    /// assert!(!deadline.is_expired_now(&clock));
    /// ```
    pub fn start_now<C: reliakit_core::Clock>(&self, clock: &C) -> Deadline {
        self.start(clock.now())
    }
}

/// Convenience methods that read the current time from a
/// [`Clock`](reliakit_core::Clock) instead of taking an explicit `now: u64`.
///
/// Available with the `core` feature. Each forwards to the matching `now`-taking
/// method, which remains the primitive API.
#[cfg(feature = "core")]
impl Deadline {
    /// Like [`elapsed`](Self::elapsed), reading the time from `clock`.
    pub fn elapsed_now<C: reliakit_core::Clock>(&self, clock: &C) -> u64 {
        self.elapsed(clock.now())
    }

    /// Like [`remaining`](Self::remaining), reading the time from `clock`.
    pub fn remaining_now<C: reliakit_core::Clock>(&self, clock: &C) -> u64 {
        self.remaining(clock.now())
    }

    /// Like [`is_expired`](Self::is_expired), reading the time from `clock`.
    pub fn is_expired_now<C: reliakit_core::Clock>(&self, clock: &C) -> bool {
        self.is_expired(clock.now())
    }

    /// Like [`check`](Self::check), reading the time from `clock`.
    pub fn check_now<C: reliakit_core::Clock>(&self, clock: &C) -> Option<u64> {
        self.check(clock.now())
    }

    /// Like [`allows`](Self::allows), reading the time from `clock`.
    pub fn allows_now<C: reliakit_core::Clock>(&self, clock: &C, duration: u64) -> bool {
        self.allows(clock.now(), duration)
    }

    /// Like [`clamp`](Self::clamp), reading the time from `clock`.
    pub fn clamp_now<C: reliakit_core::Clock>(&self, clock: &C, duration: u64) -> u64 {
        self.clamp(clock.now(), duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_starts_a_deadline() {
        let t = Timeout::new(100);
        assert_eq!(t.budget(), 100);
        let d = t.start(50);
        assert_eq!(d, Deadline::new(50, 100));
        assert_eq!(d.start(), 50);
        assert_eq!(d.budget(), 100);
        assert_eq!(d.expiry(), 150);
    }

    #[test]
    fn remaining_and_elapsed_track_time() {
        let d = Deadline::new(1_000, 500);
        assert_eq!(d.elapsed(1_000), 0);
        assert_eq!(d.remaining(1_000), 500);
        assert_eq!(d.elapsed(1_200), 200);
        assert_eq!(d.remaining(1_200), 300);
        assert_eq!(d.elapsed(1_500), 500);
        assert_eq!(d.remaining(1_500), 0);
    }

    #[test]
    fn expiry_boundary_is_inclusive() {
        let d = Deadline::new(0, 10);
        assert!(!d.is_expired(9));
        assert!(d.is_expired(10)); // exactly at expiry counts as expired
        assert!(d.is_expired(11));
        assert_eq!(d.check(9), Some(1));
        assert_eq!(d.check(10), None);
    }

    #[test]
    fn zero_budget_expires_immediately() {
        let d = Deadline::new(42, 0);
        assert_eq!(d.expiry(), 42);
        assert!(d.is_expired(42));
        assert_eq!(d.remaining(42), 0);
        assert_eq!(d.check(42), None);
        // Before the start instant it is not yet expired.
        assert!(!d.is_expired(41));
        assert_eq!(d.check(41), Some(1));
    }

    #[test]
    fn backwards_clock_does_not_panic_or_underflow() {
        let d = Deadline::new(1_000, 200);
        // now < start: elapsed saturates to 0, remaining is the full budget.
        assert_eq!(d.elapsed(0), 0);
        assert_eq!(d.remaining(0), 1_200);
        assert!(!d.is_expired(0));
    }

    #[test]
    fn expiry_saturates_on_overflow() {
        let d = Deadline::new(u64::MAX - 5, 100);
        assert_eq!(d.expiry(), u64::MAX);
        assert!(!d.is_expired(u64::MAX - 1));
        assert!(d.is_expired(u64::MAX));
        assert_eq!(d.remaining(u64::MAX - 10), 10);
    }

    #[test]
    fn allows_checks_fit() {
        let d = Deadline::new(0, 100);
        assert!(d.allows(0, 100));
        assert!(d.allows(0, 99));
        assert!(!d.allows(0, 101));
        assert!(d.allows(60, 40));
        assert!(!d.allows(60, 41));
        assert!(!d.allows(100, 1)); // already expired
        assert!(d.allows(0, 0)); // zero duration always fits
        assert!(d.allows(100, 0)); // ...even once expired
    }

    #[test]
    fn clamp_caps_duration_by_remaining() {
        let d = Deadline::new(0, 1_000);
        assert_eq!(d.clamp(0, 800), 800); // 800 < 1000 remaining
        assert_eq!(d.clamp(500, 800), 500); // only 500 left
        assert_eq!(d.clamp(1_000, 800), 0); // expired -> no time
        assert_eq!(d.clamp(0, 1_000), 1_000); // exactly the budget
    }

    #[test]
    fn defaults_are_zero() {
        assert_eq!(Timeout::default(), Timeout::new(0));
        assert_eq!(Deadline::default(), Deadline::new(0, 0));
        // A default deadline is already expired, not infinite.
        assert!(Deadline::default().is_expired(0));
        assert_eq!(Deadline::default().check(0), None);
    }
}

#[cfg(all(test, feature = "core"))]
mod core_tests {
    use super::*;
    use reliakit_core::ManualClock;

    #[test]
    fn now_methods_match_explicit_now() {
        let clock = ManualClock::new(1_000);
        let deadline = Timeout::new(500).start_now(&clock);
        assert_eq!(deadline, Timeout::new(500).start(1_000));

        clock.set(1_200);
        assert_eq!(deadline.elapsed_now(&clock), deadline.elapsed(1_200));
        assert_eq!(deadline.remaining_now(&clock), deadline.remaining(1_200));
        assert_eq!(deadline.is_expired_now(&clock), deadline.is_expired(1_200));
        assert_eq!(deadline.check_now(&clock), deadline.check(1_200));
        assert_eq!(
            deadline.allows_now(&clock, 200),
            deadline.allows(1_200, 200)
        );
        assert_eq!(deadline.clamp_now(&clock, 400), deadline.clamp(1_200, 400));

        clock.set(2_000); // past expiry
        assert!(deadline.is_expired_now(&clock));
        assert_eq!(deadline.check_now(&clock), None);
    }
}

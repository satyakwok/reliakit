//! Clock-agnostic circuit breaker.
//!
//! A circuit breaker protects a caller from a failing dependency: once failures
//! pile up it "opens" and rejects calls immediately (failing fast) instead of
//! hammering a service that is already down, then periodically lets a trial call
//! through to test recovery.
//!
//! [`CircuitBreaker`] is a small, `Copy` state machine. It does **not** read the
//! clock, sleep, or allocate; you pass the current time in on each call as a
//! plain `u64` in whatever monotonic unit you choose (milliseconds is typical).
//! That keeps it usable from synchronous code, any async runtime, and `no_std`
//! / embedded targets, and makes its behavior fully deterministic in tests.
//!
//! # States
//!
//! ```text
//!            failures >= failure_threshold
//!   Closed ───────────────────────────────▶ Open
//!     ▲                                       │
//!     │ successes >= success_threshold        │ cooldown elapsed
//!     │                                       ▼
//!     └────────────── HalfOpen ◀──────────────┘
//!                        │
//!                        │ any failure
//!                        └──────────────▶ Open
//! ```
//!
//! - **Closed**, calls flow normally. Consecutive failures are counted; once
//!   they reach `failure_threshold` the breaker trips to **Open**.
//! - **Open**, calls are rejected immediately. After `cooldown` time units the
//!   next [`allow`](CircuitBreaker::allow) moves it to **HalfOpen**.
//! - **HalfOpen**, trial calls are allowed. `success_threshold` consecutive
//!   successes close the breaker; the first failure reopens it.
//!
//! # Example
//!
//! ```
//! use reliakit_circuit::{CircuitBreaker, State};
//!
//! // Trip after 3 consecutive failures; stay open for 30_000 ms.
//! let mut cb = CircuitBreaker::new(3, 30_000);
//!
//! // A run of failures opens the breaker.
//! for _ in 0..3 {
//!     assert!(cb.allow(0));      // still Closed, calls allowed
//!     cb.on_failure(0);
//! }
//! assert_eq!(cb.state(), State::Open);
//! assert!(!cb.allow(1_000));     // rejected while Open (cooldown not elapsed)
//!
//! // After the cooldown, one trial call is allowed (HalfOpen).
//! assert!(cb.allow(31_000));
//! assert_eq!(cb.state(), State::HalfOpen);
//!
//! // A success closes it again.
//! cb.on_success();
//! assert_eq!(cb.state(), State::Closed);
//! ```
//!
//! # Counting failures by rate
//!
//! [`CircuitBreaker`] counts *consecutive* failures. For a *failure rate* over a
//! rolling window, "trip if N of the last M calls failed", use
//! [`RollingBreaker`], a const-generic, inline (zero-allocation) variant.
//!
//! # Feature flags
//!
//! - `core` (off by default) adds `*_now(clock)` convenience methods on
//!   [`CircuitBreaker`] and [`RollingBreaker`] that read the time from a
//!   `reliakit_core::Clock`. It pulls in `reliakit-core` (`no_std`, zero
//!   third-party dependencies); the `now: u64` methods remain the primitive API.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod rolling;

pub use rolling::RollingBreaker;

/// The state of a [`CircuitBreaker`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Calls flow normally; failures are being counted.
    Closed,
    /// Calls are rejected immediately until the cooldown elapses.
    Open,
    /// A trial period: limited calls are allowed to test recovery.
    HalfOpen,
}

/// A circuit breaker: a small, `Copy` state machine that decides whether calls
/// to a dependency should be allowed, based on their recent success/failure
/// history and a caller-supplied clock.
///
/// Time is a plain `u64` in any monotonic unit you choose (commonly
/// milliseconds); `cooldown` uses the same unit. The breaker never reads the
/// clock itself; pass `now` to [`allow`](Self::allow) and
/// [`on_failure`](Self::on_failure).
///
/// `CircuitBreaker` is not internally synchronized. Share one across threads by
/// wrapping it in your own `Mutex`/lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    success_threshold: u32,
    cooldown: u64,
    state: State,
    failures: u32,
    successes: u32,
    opened_at: u64,
}

impl CircuitBreaker {
    /// Creates a breaker that trips to [`State::Open`] after `failure_threshold`
    /// consecutive failures and stays open for `cooldown` time units.
    ///
    /// The success threshold defaults to `1` (a single trial success closes the
    /// breaker); change it with [`with_success_threshold`](Self::with_success_threshold).
    /// A `failure_threshold` of `0` is treated as `1`.
    pub const fn new(failure_threshold: u32, cooldown: u64) -> Self {
        Self {
            failure_threshold: if failure_threshold == 0 {
                1
            } else {
                failure_threshold
            },
            success_threshold: 1,
            cooldown,
            state: State::Closed,
            failures: 0,
            successes: 0,
            opened_at: 0,
        }
    }

    /// Sets how many consecutive successes in [`State::HalfOpen`] are required to
    /// close the breaker. A value of `0` is treated as `1`.
    pub const fn with_success_threshold(mut self, success_threshold: u32) -> Self {
        self.success_threshold = if success_threshold == 0 {
            1
        } else {
            success_threshold
        };
        self
    }

    /// Returns the current state without advancing time.
    ///
    /// Note that a breaker which has been [`State::Open`] past its cooldown still
    /// reports `Open` here until the next [`allow`](Self::allow) call moves it to
    /// [`State::HalfOpen`].
    pub const fn state(&self) -> State {
        self.state
    }

    /// Returns the configured failure threshold.
    pub const fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }

    /// Returns the configured success threshold.
    pub const fn success_threshold(&self) -> u32 {
        self.success_threshold
    }

    /// Returns the configured cooldown, in the caller's time unit.
    pub const fn cooldown(&self) -> u64 {
        self.cooldown
    }

    /// Returns whether a call may proceed at `now`.
    ///
    /// If the breaker is [`State::Open`] and `cooldown` time units have elapsed
    /// since it opened, this transitions it to [`State::HalfOpen`] and returns
    /// `true` to permit a trial call. Otherwise it returns `true` for
    /// `Closed`/`HalfOpen` and `false` for `Open`.
    ///
    /// `now` is expected to be monotonic non-decreasing; a clock that moves
    /// backwards is handled with saturating arithmetic (it simply keeps the
    /// breaker open) and never panics.
    pub fn allow(&mut self, now: u64) -> bool {
        if matches!(self.state, State::Open) && now.saturating_sub(self.opened_at) >= self.cooldown
        {
            self.state = State::HalfOpen;
            self.successes = 0;
        }
        !matches!(self.state, State::Open)
    }

    /// Records that an allowed call succeeded.
    ///
    /// In [`State::Closed`] this resets the consecutive-failure count. In
    /// [`State::HalfOpen`] it counts toward `success_threshold`, closing the
    /// breaker once reached. Has no effect while [`State::Open`].
    pub fn on_success(&mut self) {
        match self.state {
            State::Closed => self.failures = 0,
            State::HalfOpen => {
                self.successes = self.successes.saturating_add(1);
                if self.successes >= self.success_threshold {
                    self.state = State::Closed;
                    self.failures = 0;
                    self.successes = 0;
                }
            }
            State::Open => {}
        }
    }

    /// Records that an allowed call failed, at time `now`.
    ///
    /// In [`State::Closed`] this counts toward `failure_threshold`, tripping the
    /// breaker to [`State::Open`] once reached. In [`State::HalfOpen`] any
    /// failure reopens the breaker. Has no effect while [`State::Open`].
    pub fn on_failure(&mut self, now: u64) {
        match self.state {
            State::Closed => {
                self.failures = self.failures.saturating_add(1);
                if self.failures >= self.failure_threshold {
                    self.trip(now);
                }
            }
            State::HalfOpen => self.trip(now),
            State::Open => {}
        }
    }

    /// Forces the breaker [`State::Open`] as of `now` (e.g. on a fatal signal).
    pub fn trip(&mut self, now: u64) {
        self.state = State::Open;
        self.opened_at = now;
        self.failures = 0;
        self.successes = 0;
    }

    /// Forces the breaker back to [`State::Closed`] and clears its counters.
    pub fn reset(&mut self) {
        self.state = State::Closed;
        self.failures = 0;
        self.successes = 0;
        self.opened_at = 0;
    }
}

/// Convenience methods that read the current time from a
/// [`Clock`](reliakit_core::Clock) instead of taking an explicit `now: u64`.
///
/// Available with the `core` feature. Each forwards to the matching `now`-taking
/// method, which remains the primitive API.
#[cfg(feature = "core")]
impl CircuitBreaker {
    /// Like [`allow`](Self::allow), reading the time from `clock`.
    ///
    /// ```
    /// use reliakit_circuit::CircuitBreaker;
    /// use reliakit_core::ManualClock;
    ///
    /// let clock = ManualClock::new(0);
    /// let mut breaker = CircuitBreaker::new(1, 1_000);
    /// breaker.on_failure_now(&clock); // one failure trips it
    /// assert!(!breaker.allow_now(&clock)); // still cooling down
    /// clock.set(1_000);
    /// assert!(breaker.allow_now(&clock)); // cooldown elapsed -> half-open trial
    /// ```
    pub fn allow_now<C: reliakit_core::Clock>(&mut self, clock: &C) -> bool {
        self.allow(clock.now())
    }

    /// Like [`on_failure`](Self::on_failure), reading the time from `clock`.
    pub fn on_failure_now<C: reliakit_core::Clock>(&mut self, clock: &C) {
        self.on_failure(clock.now())
    }

    /// Like [`trip`](Self::trip), reading the time from `clock`.
    pub fn trip_now<C: reliakit_core::Clock>(&mut self, clock: &C) {
        self.trip(clock.now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed_and_allows() {
        let mut cb = CircuitBreaker::new(3, 1000);
        assert_eq!(cb.state(), State::Closed);
        assert!(cb.allow(0));
    }

    #[test]
    fn failures_below_threshold_stay_closed() {
        let mut cb = CircuitBreaker::new(3, 1000);
        cb.on_failure(0);
        cb.on_failure(0);
        assert_eq!(cb.state(), State::Closed);
        assert!(cb.allow(0));
    }

    #[test]
    fn reaching_threshold_opens_and_rejects() {
        let mut cb = CircuitBreaker::new(3, 1000);
        for _ in 0..3 {
            cb.on_failure(0);
        }
        assert_eq!(cb.state(), State::Open);
        assert!(!cb.allow(500)); // cooldown not elapsed
    }

    #[test]
    fn success_resets_failure_run_in_closed() {
        let mut cb = CircuitBreaker::new(3, 1000);
        cb.on_failure(0);
        cb.on_failure(0);
        cb.on_success();
        cb.on_failure(0);
        cb.on_failure(0);
        assert_eq!(cb.state(), State::Closed); // run was interrupted
        cb.on_failure(0);
        assert_eq!(cb.state(), State::Open);
    }

    #[test]
    fn open_transitions_to_half_open_after_cooldown() {
        let mut cb = CircuitBreaker::new(1, 1000);
        cb.on_failure(0);
        assert_eq!(cb.state(), State::Open);
        assert!(!cb.allow(999)); // 1ms short
        assert_eq!(cb.state(), State::Open);
        assert!(cb.allow(1000)); // exactly cooldown -> HalfOpen
        assert_eq!(cb.state(), State::HalfOpen);
    }

    #[test]
    fn half_open_success_closes() {
        let mut cb = CircuitBreaker::new(1, 1000);
        cb.on_failure(0);
        assert!(cb.allow(1000));
        assert_eq!(cb.state(), State::HalfOpen);
        cb.on_success();
        assert_eq!(cb.state(), State::Closed);
    }

    #[test]
    fn half_open_failure_reopens_with_new_cooldown() {
        let mut cb = CircuitBreaker::new(1, 1000);
        cb.on_failure(0);
        assert!(cb.allow(1000));
        assert_eq!(cb.state(), State::HalfOpen);
        cb.on_failure(1000);
        assert_eq!(cb.state(), State::Open);
        assert!(!cb.allow(1999)); // cooldown counts from the reopen at t=1000
        assert!(cb.allow(2000));
        assert_eq!(cb.state(), State::HalfOpen);
    }

    #[test]
    fn success_threshold_requires_multiple_successes() {
        let mut cb = CircuitBreaker::new(1, 1000).with_success_threshold(2);
        cb.on_failure(0);
        assert!(cb.allow(1000));
        cb.on_success();
        assert_eq!(cb.state(), State::HalfOpen); // 1 of 2
        cb.on_success();
        assert_eq!(cb.state(), State::Closed); // 2 of 2
    }

    #[test]
    fn cooldown_zero_allows_immediately() {
        let mut cb = CircuitBreaker::new(1, 0);
        cb.on_failure(0);
        assert_eq!(cb.state(), State::Open);
        assert!(cb.allow(0)); // 0 elapsed >= 0 cooldown
        assert_eq!(cb.state(), State::HalfOpen);
    }

    #[test]
    fn zero_failure_threshold_is_treated_as_one() {
        let mut cb = CircuitBreaker::new(0, 1000);
        assert_eq!(cb.failure_threshold(), 1);
        cb.on_failure(0);
        assert_eq!(cb.state(), State::Open);
    }

    #[test]
    fn backwards_clock_does_not_panic_or_close_early() {
        let mut cb = CircuitBreaker::new(1, 1000);
        cb.on_failure(10_000);
        // now < opened_at: saturating_sub -> 0, which is < cooldown, stays Open.
        assert!(!cb.allow(5_000));
        assert_eq!(cb.state(), State::Open);
    }

    #[test]
    fn trip_and_reset_are_explicit() {
        let mut cb = CircuitBreaker::new(5, 1000);
        cb.trip(0);
        assert_eq!(cb.state(), State::Open);
        cb.reset();
        assert_eq!(cb.state(), State::Closed);
        assert!(cb.allow(0));
    }

    #[test]
    fn on_outcome_while_open_is_ignored() {
        let mut cb = CircuitBreaker::new(1, 1000);
        cb.on_failure(0);
        let before = cb;
        cb.on_success();
        cb.on_failure(0);
        assert_eq!(cb, before); // no state change while Open
    }
}

#[cfg(all(test, feature = "core"))]
mod core_tests {
    use super::*;
    use reliakit_core::ManualClock;

    #[test]
    fn now_methods_match_explicit_now() {
        let clock = ManualClock::new(0);
        let mut viaclock = CircuitBreaker::new(2, 1_000);
        let mut explicit = CircuitBreaker::new(2, 1_000);

        viaclock.on_failure_now(&clock);
        explicit.on_failure(0);
        assert_eq!(viaclock, explicit);

        viaclock.on_failure_now(&clock); // trips
        explicit.on_failure(0);
        assert_eq!(viaclock, explicit);
        assert_eq!(viaclock.state(), State::Open);

        assert_eq!(viaclock.allow_now(&clock), explicit.allow(0)); // both false
        clock.set(1_000);
        assert_eq!(viaclock.allow_now(&clock), explicit.allow(1_000)); // both true
        assert_eq!(viaclock, explicit);
    }

    #[test]
    fn trip_now_matches_trip() {
        let clock = ManualClock::new(500);
        let mut viaclock = CircuitBreaker::new(3, 100);
        let mut explicit = CircuitBreaker::new(3, 100);
        viaclock.trip_now(&clock);
        explicit.trip(500);
        assert_eq!(viaclock, explicit);
        assert_eq!(viaclock.state(), State::Open);
    }
}

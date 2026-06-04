//! A count-based sliding-window circuit breaker.

use crate::State;

/// A circuit breaker that trips on the number of failures within the last
/// `WINDOW` calls, rather than on *consecutive* failures like
/// [`CircuitBreaker`](crate::CircuitBreaker).
///
/// The window is a fixed-size ring of the most recent `WINDOW` outcomes, stored
/// inline (`[bool; WINDOW]`) — no allocation, `no_std`-friendly. The breaker
/// trips to [`State::Open`] once the window holds at least `failure_threshold`
/// failures, then behaves exactly like `CircuitBreaker` for cooldown and
/// half-open recovery.
///
/// Time is a plain `u64` in any monotonic unit you choose; the breaker never
/// reads the clock. All arithmetic saturates, so a backwards-moving clock cannot
/// panic. A `WINDOW` of `0` never trips on the failure rate (there is nothing to
/// count).
///
/// # Example
///
/// ```
/// use reliakit_circuit::{RollingBreaker, State};
///
/// // Trip if 3 of the last 5 calls fail.
/// let mut breaker = RollingBreaker::<5>::new(3, 1_000);
///
/// // Non-consecutive failures still count toward the window.
/// breaker.on_failure(0);
/// breaker.on_success();
/// breaker.on_failure(0);
/// breaker.on_success();
/// assert_eq!(breaker.state(), State::Closed);
/// breaker.on_failure(0); // 3 failures within the last 5 calls
/// assert_eq!(breaker.state(), State::Open);
/// assert!(!breaker.allow(500));    // still cooling down
/// assert!(breaker.allow(1_000));   // cooldown elapsed -> half-open trial
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollingBreaker<const WINDOW: usize> {
    failure_threshold: u32,
    success_threshold: u32,
    cooldown: u64,
    state: State,
    outcomes: [bool; WINDOW],
    head: usize,
    filled: usize,
    failures_in_window: u32,
    successes: u32,
    opened_at: u64,
}

impl<const WINDOW: usize> RollingBreaker<WINDOW> {
    /// Creates a breaker that trips once `failure_threshold` of the last
    /// `WINDOW` calls have failed, staying open for `cooldown` time units.
    ///
    /// A `failure_threshold` of `0` is treated as `1`. The success threshold
    /// defaults to `1`; change it with
    /// [`with_success_threshold`](Self::with_success_threshold).
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
            outcomes: [false; WINDOW],
            head: 0,
            filled: 0,
            failures_in_window: 0,
            successes: 0,
            opened_at: 0,
        }
    }

    /// Sets how many consecutive successes in [`State::HalfOpen`] close the
    /// breaker. A value of `0` is treated as `1`.
    pub const fn with_success_threshold(mut self, success_threshold: u32) -> Self {
        self.success_threshold = if success_threshold == 0 {
            1
        } else {
            success_threshold
        };
        self
    }

    /// Returns the current state without advancing time.
    pub const fn state(&self) -> State {
        self.state
    }

    /// The window size (`WINDOW`).
    pub const fn window_size(&self) -> usize {
        WINDOW
    }

    /// The number of failures currently recorded in the window.
    pub const fn failures_in_window(&self) -> u32 {
        self.failures_in_window
    }

    /// The configured failure threshold.
    pub const fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }

    /// The configured success threshold.
    pub const fn success_threshold(&self) -> u32 {
        self.success_threshold
    }

    /// The configured cooldown, in the caller's time unit.
    pub const fn cooldown(&self) -> u64 {
        self.cooldown
    }

    /// Returns whether a call may proceed at `now`, moving an expired
    /// [`State::Open`] breaker to [`State::HalfOpen`] just like
    /// [`CircuitBreaker::allow`](crate::CircuitBreaker::allow).
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
    /// In [`State::Closed`] the success enters the window (and can push an old
    /// failure out of it). In [`State::HalfOpen`] it counts toward
    /// `success_threshold`. Has no effect while [`State::Open`].
    pub fn on_success(&mut self) {
        match self.state {
            State::Closed => self.record(false),
            State::HalfOpen => {
                self.successes = self.successes.saturating_add(1);
                if self.successes >= self.success_threshold {
                    self.reset();
                }
            }
            State::Open => {}
        }
    }

    /// Records that an allowed call failed, at time `now`.
    ///
    /// In [`State::Closed`] the failure enters the window and trips the breaker
    /// once the window holds `failure_threshold` failures. In [`State::HalfOpen`]
    /// any failure reopens the breaker. Has no effect while [`State::Open`].
    pub fn on_failure(&mut self, now: u64) {
        match self.state {
            State::Closed => {
                self.record(true);
                if self.failures_in_window >= self.failure_threshold {
                    self.trip(now);
                }
            }
            State::HalfOpen => self.trip(now),
            State::Open => {}
        }
    }

    /// Forces the breaker [`State::Open`] as of `now`, clearing the window.
    pub fn trip(&mut self, now: u64) {
        self.state = State::Open;
        self.opened_at = now;
        self.clear_window();
        self.successes = 0;
    }

    /// Forces the breaker back to [`State::Closed`] and clears all counters.
    pub fn reset(&mut self) {
        self.state = State::Closed;
        self.clear_window();
        self.successes = 0;
        self.opened_at = 0;
    }

    /// Pushes one outcome into the ring, maintaining `failures_in_window`.
    fn record(&mut self, failure: bool) {
        if WINDOW == 0 {
            return; // nothing to count; the rate can never trip
        }
        if self.filled == WINDOW {
            // Overwriting the oldest slot: drop its contribution first.
            if self.outcomes[self.head] {
                self.failures_in_window -= 1;
            }
        } else {
            self.filled += 1;
        }
        self.outcomes[self.head] = failure;
        if failure {
            self.failures_in_window += 1;
        }
        self.head = (self.head + 1) % WINDOW;
    }

    fn clear_window(&mut self) {
        self.head = 0;
        self.filled = 0;
        self.failures_in_window = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trips_on_non_consecutive_failures_in_window() {
        let mut b = RollingBreaker::<5>::new(3, 100);
        b.on_failure(0);
        b.on_success();
        b.on_failure(0);
        b.on_success();
        assert_eq!(b.state(), State::Closed);
        assert_eq!(b.failures_in_window(), 2);
        b.on_failure(0); // third failure within the last 5 calls
        assert_eq!(b.state(), State::Open);
    }

    #[test]
    fn old_failures_age_out_of_window() {
        // Window 3, trip at 2 failures.
        let mut b = RollingBreaker::<3>::new(2, 100);
        b.on_failure(0); // window: [F]            failures=1
        b.on_success(); //  window: [F,S]          failures=1
        b.on_success(); //  window: [F,S,S]        failures=1
        b.on_success(); //  window: [S,S,S] (F out) failures=0
        assert_eq!(b.failures_in_window(), 0);
        assert_eq!(b.state(), State::Closed);
        b.on_failure(0); // window: [S,S,F]        failures=1, no trip
        assert_eq!(b.state(), State::Closed);
        b.on_failure(0); // window: [S,F,F]        failures=2 -> trip
        assert_eq!(b.state(), State::Open);
    }

    #[test]
    fn half_open_recovers_then_closes() {
        let mut b = RollingBreaker::<4>::new(2, 100).with_success_threshold(2);
        b.on_failure(0);
        b.on_failure(0); // trip
        assert_eq!(b.state(), State::Open);
        assert!(!b.allow(50)); // cooling down
        assert!(b.allow(100)); // -> half-open
        assert_eq!(b.state(), State::HalfOpen);
        b.on_success();
        assert_eq!(b.state(), State::HalfOpen); // needs 2
        b.on_success();
        assert_eq!(b.state(), State::Closed);
        assert_eq!(b.failures_in_window(), 0); // window cleared on close
    }

    #[test]
    fn half_open_failure_reopens() {
        let mut b = RollingBreaker::<4>::new(2, 100);
        b.on_failure(0);
        b.on_failure(0);
        assert!(b.allow(100)); // half-open
        b.on_failure(200); // reopens
        assert_eq!(b.state(), State::Open);
        assert!(!b.allow(250));
    }

    #[test]
    fn backwards_clock_keeps_open_without_panic() {
        let mut b = RollingBreaker::<2>::new(1, 100);
        b.on_failure(1_000); // trip at t=1000
        assert_eq!(b.state(), State::Open);
        assert!(!b.allow(0)); // now < opened_at: saturating -> stays open
        assert_eq!(b.state(), State::Open);
    }

    #[test]
    fn zero_window_never_trips_on_rate() {
        let mut b = RollingBreaker::<0>::new(1, 100);
        for _ in 0..1_000 {
            b.on_failure(0);
        }
        assert_eq!(b.state(), State::Closed);
        assert_eq!(b.failures_in_window(), 0);
        // An explicit trip still works.
        b.trip(0);
        assert_eq!(b.state(), State::Open);
    }

    #[test]
    fn accessors_and_threshold_flooring() {
        let b = RollingBreaker::<8>::new(0, 250).with_success_threshold(0);
        assert_eq!(b.window_size(), 8);
        assert_eq!(b.failure_threshold(), 1); // 0 floored to 1
        assert_eq!(b.success_threshold(), 1); // 0 floored to 1
        assert_eq!(b.cooldown(), 250);
        assert_eq!(b.state(), State::Closed);
    }

    #[test]
    fn reset_clears_everything() {
        let mut b = RollingBreaker::<3>::new(2, 100);
        b.on_failure(0);
        b.on_failure(0);
        assert_eq!(b.state(), State::Open);
        b.reset();
        assert_eq!(b.state(), State::Closed);
        assert_eq!(b.failures_in_window(), 0);
    }
}

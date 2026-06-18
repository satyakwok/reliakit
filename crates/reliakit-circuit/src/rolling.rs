//! A count-based sliding-window circuit breaker.

use crate::State;

/// Fixed-size ring buffer of pass/fail outcomes with a maintained failure count.
///
/// Stores the last `N` boolean outcomes inline (`[bool; N]`, no allocation) and
/// keeps a running tally of how many of them are failures, so reading the
/// current failure count is O(1) rather than a scan.
///
/// Each call to [`record`](Self::record) writes one slot. Once the ring is full,
/// subsequent writes overwrite the oldest slot; if that evicted slot was a
/// failure, the count is decremented before the new outcome is written. Reads
/// via [`failures`](Self::failures) return the count for the current window
/// contents.
///
/// `N == 0` is a legal but inert configuration: [`record`](Self::record) is a
/// no-op and [`failures`](Self::failures) always returns `0`. This lets callers
/// build a rolling-window-shaped type that opts out of windowed counting
/// without a separate code path.
///
/// All arithmetic on the failure counter saturates, so the count cannot
/// underflow or overflow even under pathological sequences.
///
/// This type is an internal building block for [`RollingBreaker`]; it does not
/// model any state machine of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RollingWindow<const N: usize> {
    outcomes: [bool; N],
    head: usize,
    filled: usize,
    failures: u32,
}

impl<const N: usize> RollingWindow<N> {
    /// Creates an empty window. No slots are filled and the failure count is `0`.
    const fn new() -> Self {
        Self {
            outcomes: [false; N],
            head: 0,
            filled: 0,
            failures: 0,
        }
    }

    /// Returns the number of failures currently held in the window.
    ///
    /// Always in the range `0..=min(filled, N)`. Returns `0` when `N == 0`.
    const fn failures(&self) -> u32 {
        self.failures
    }

    /// Records one outcome (`true` = failure, `false` = success) into the ring.
    ///
    /// If the ring is not yet full, the outcome fills the next slot. Once full,
    /// the oldest slot is overwritten and its contribution to the failure count
    /// is dropped before the new outcome is written.
    ///
    /// Has no effect when `N == 0`.
    fn record(&mut self, failure: bool) {
        if N == 0 {
            return; // nothing to count
        }
        if self.filled == N {
            // Overwriting the oldest slot: drop its contribution first.
            if self.outcomes[self.head] {
                self.failures = self.failures.saturating_sub(1);
            }
        } else {
            self.filled += 1;
        }
        self.outcomes[self.head] = failure;
        if failure {
            self.failures = self.failures.saturating_add(1);
        }
        self.head = (self.head + 1) % N;
    }

    /// Empties the window: all slots are forgotten and the failure count
    /// returns to `0`. The next [`record`](Self::record) writes to slot `0`.
    fn clear(&mut self) {
        self.head = 0;
        self.filled = 0;
        self.failures = 0;
    }
}

/// A circuit breaker that trips on the number of failures within the last
/// `WINDOW` calls, rather than on *consecutive* failures like
/// [`CircuitBreaker`](crate::CircuitBreaker).
///
/// The window is a fixed-size ring of the most recent `WINDOW` outcomes, stored
/// inline (`[bool; WINDOW]`), no allocation, `no_std`-friendly. The breaker
/// trips to [`State::Open`] once the window holds at least `failure_threshold`
/// failures, then behaves exactly like `CircuitBreaker` for cooldown and
/// half-open recovery.
///
/// Internally, the outcome ring and its failure count are delegated to a
/// private `RollingWindow` helper. The breaker itself owns only the state
/// machine: current [`State`], cooldown bookkeeping (`opened_at`), and the
/// half-open probe streak (`successes`). The streak counter is *not* part of
/// the rolling window; it tracks consecutive successful probes in
/// [`State::HalfOpen`] and is reset on every transition into half-open, on
/// [`trip`](Self::trip), and on [`reset`](Self::reset).
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
    window: RollingWindow<WINDOW>,
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
            window: RollingWindow::new(),
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
        self.window.failures()
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
            State::Closed => self.window.record(false),
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
                self.window.record(true);
                if self.window.failures() >= self.failure_threshold {
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
        self.window.clear();
        self.successes = 0;
    }

    /// Forces the breaker back to [`State::Closed`] and clears all counters.
    pub fn reset(&mut self) {
        self.state = State::Closed;
        self.window.clear();
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
impl<const WINDOW: usize> RollingBreaker<WINDOW> {
    /// Like [`allow`](Self::allow), reading the time from `clock`.
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

    mod rolling_window {
        use super::super::RollingWindow;

        #[test]
        fn empty_window_has_no_failures() {
            let w = RollingWindow::<4>::new();
            assert_eq!(w.failures(), 0);
        }

        #[test]
        fn partial_fill_counts_failures() {
            let mut w = RollingWindow::<5>::new();
            w.record(true); // window: [F]            failures=1
            w.record(false); // window: [F,S]          failures=1
            w.record(true); // window: [F,S,F]        failures=2
            assert_eq!(w.failures(), 2);
        }

        #[test]
        fn exact_full_counts_failures() {
            let mut w = RollingWindow::<3>::new();
            w.record(true); // window: [F]            failures=1
            w.record(false); // window: [F,S]          failures=1
            w.record(true); // window: [F,S,F]        failures=2
            assert_eq!(w.failures(), 2);
        }

        #[test]
        fn eviction_of_failure_decrements_count() {
            let mut w = RollingWindow::<3>::new();
            w.record(true); // window: [F]            failures=1
            w.record(true); // window: [F,F]          failures=2
            w.record(true); // window: [F,F,F]        failures=3
            assert_eq!(w.failures(), 3);
            w.record(false); // window: [S,F,F] (F out) failures=2
            assert_eq!(w.failures(), 2);
        }

        #[test]
        fn eviction_of_success_does_not_change_count() {
            let mut w = RollingWindow::<3>::new();
            w.record(false); // window: [S]            failures=0
            w.record(true); // window: [S,F]          failures=1
            w.record(false); // window: [S,F,S]        failures=1
            assert_eq!(w.failures(), 1);
            w.record(false); // window: [S,F,S] (S out) failures=1
            assert_eq!(w.failures(), 1);
            w.record(false); // window: [S,S,S] (F out) failures=0
            assert_eq!(w.failures(), 0);
        }

        #[test]
        fn full_wraparound_remains_correct() {
            // Write 3 * N alternating outcomes; last N decide the count.
            let mut w = RollingWindow::<4>::new();
            for i in 0..12 {
                w.record(i % 2 == 0); // F,S,F,S,F,S,F,S,F,S,F,S
            }
            // Last 4 outcomes were F,S,F,S -> 2 failures.
            assert_eq!(w.failures(), 2);
        }

        #[test]
        fn clear_resets_window() {
            let mut w = RollingWindow::<3>::new();
            w.record(true); // window: [F]             failures=1
            w.record(true); // window: [F,F]           failures=2
            w.record(true); // window: [F,F,F]         failures=3
            assert_eq!(w.failures(), 3);
            w.clear(); // window: []              failures=0
            assert_eq!(w.failures(), 0);
            w.record(true); // window: [F]             failures=1
            assert_eq!(w.failures(), 1);
        }

        #[test]
        fn zero_sized_window_is_inert() {
            let mut w = RollingWindow::<0>::new();
            for _ in 0..1_000 {
                w.record(true);
            }
            assert_eq!(w.failures(), 0);
        }

        #[test]
        fn failures_never_exceed_window_size() {
            let mut w = RollingWindow::<4>::new();
            for _ in 0..1_000 {
                w.record(true);
            }
            assert_eq!(w.failures(), 4); // capped at N
        }
    }
}

#[cfg(all(test, feature = "core"))]
mod core_tests {
    use super::*;
    use reliakit_core::ManualClock;

    #[test]
    fn now_methods_match_explicit_now() {
        let clock = ManualClock::new(0);
        let mut viaclock = RollingBreaker::<4>::new(2, 1_000);
        let mut explicit = RollingBreaker::<4>::new(2, 1_000);

        viaclock.on_failure_now(&clock);
        explicit.on_failure(0);
        viaclock.on_failure_now(&clock); // trips
        explicit.on_failure(0);
        assert_eq!(viaclock, explicit);
        assert_eq!(viaclock.state(), State::Open);

        assert_eq!(viaclock.allow_now(&clock), explicit.allow(0)); // both false
        clock.set(1_000);
        assert_eq!(viaclock.allow_now(&clock), explicit.allow(1_000)); // both true
        assert_eq!(viaclock, explicit);
    }
}

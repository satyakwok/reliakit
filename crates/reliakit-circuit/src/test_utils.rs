//! Test-only helpers for the circuit-breaker crate.
//!
//! These exist because the crate is `#![no_std]` and the workspace convention
//! keeps test code allocation-free as well. That rules out `std::time::Instant`
//! (no clock dependency, by design — the breaker takes `now: u64` directly)
//! and `Vec` (no allocator). The two helpers below cover the gap:
//!
//! - [`ManualClock`] gives tests a `u64` time source they fully control,
//!   so transitions that depend on `cooldown` elapsing are deterministic
//!   without sleeping or mocking a real clock.
//! - [`Log`] is a fixed-capacity transition recorder used to assert the
//!   exact sequence of state changes fired by the `_observed` hooks,
//!   without pulling in `Vec`.
//!
//! Gated behind `#[cfg(test)]` so they never appear in the public API or
//! a release build.

use crate::State;

/// A deterministic `u64` clock for tests.
///
/// The breaker treats `now` as an opaque monotonic counter, so tests can
/// advance time by any amount without involving the wall clock. Construct
/// with [`ManualClock::new`] (starts at `0`), read the current value with
/// [`now`](Self::now), and move it forward with [`advance`](Self::advance).
pub struct ManualClock {
    now: u64,
}

impl ManualClock {
    pub fn new() -> Self {
        Self { now: 0 }
    }
    pub fn now(&self) -> u64 {
        self.now
    }
    pub fn advance(&mut self, by: u64) {
        self.now += by;
    }
}

/// Fixed-capacity recorder for `(from, to)` state transitions.
///
/// Used as a stand-in for `Vec<(State, State)>` because the crate is
/// `no_std` and tests stay allocation-free. Tests pass a closure that
/// captures `&mut Log` and forwards each observed transition to
/// [`push`](Self::push); assertions compare the final
/// [`as_slice`](Self::as_slice) against an expected `&[(State, State)]`.
///
/// Overflowing the capacity panics with a clear message rather than
/// silently dropping transitions — bump [`CAP`] if you hit it.

const CAP: usize = 8;

pub struct Log {
    buf: [(State, State); CAP],
    len: usize,
}

impl Log {
    pub fn new() -> Self {
        Self {
            buf: [(State::Closed, State::Closed); CAP],
            len: 0,
        }
    }
    pub fn push(&mut self, t: (State, State)) {
        assert!(self.len < CAP, "Log overflow — bump CAP");
        self.buf[self.len] = t;
        self.len += 1;
    }
    pub fn as_slice(&self) -> &[(State, State)] {
        &self.buf[..self.len]
    }
}

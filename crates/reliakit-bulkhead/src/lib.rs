//! Clock-agnostic concurrency limiter.
//!
//! `reliakit-bulkhead` caps how many operations may be *in flight* at once. It
//! is a counting semaphore: you acquire a permit before starting work and
//! release it when the work finishes. When no permit is available the request is
//! rejected immediately so load is shed instead of piling up.
//!
//! It does not block, sleep, spawn tasks, or read the clock — acquiring a permit
//! either succeeds now or fails now. That keeps it usable from synchronous code,
//! any async runtime, and `no_std` / embedded targets, with deterministic tests.
//!
//! Where [`reliakit-ratelimit`](https://docs.rs/reliakit-ratelimit) caps the
//! *rate* of operations over time, a [`Bulkhead`] caps the *number running at
//! once*. The two compose: a rate limiter decides how often to start work, a
//! bulkhead bounds how much runs concurrently.
//!
//! # Example
//!
//! ```
//! use reliakit_bulkhead::Bulkhead;
//!
//! // Allow at most two concurrent operations.
//! let mut bulkhead = Bulkhead::new(2);
//!
//! assert!(bulkhead.try_acquire_one()); // 1 in flight
//! assert!(bulkhead.try_acquire_one()); // 2 in flight
//! assert!(!bulkhead.try_acquire_one()); // full: rejected, shed load
//!
//! bulkhead.release_one(); // one operation finished
//! assert!(bulkhead.try_acquire_one()); // room again
//! ```
//!
//! # Releasing permits
//!
//! Every successful acquire must be matched by a release, including on the error
//! path, or the bulkhead will slowly fill and reject everything. The crate keeps
//! the model explicit (no RAII guard) so it stays `Copy` and `no_std` with no
//! borrowing constraints; pair acquire/release yourself, e.g. with a `scopeguard`
//! or a manual `Drop` wrapper in your own code.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// A concurrency limiter: a counting semaphore that caps in-flight operations.
///
/// `Bulkhead` is a small, `Copy` value holding a fixed `capacity` and the number
/// of permits currently held (`in_flight`). [`try_acquire`](Self::try_acquire)
/// takes permits when room exists and reports whether it succeeded;
/// [`release`](Self::release) returns them.
///
/// The capacity is clamped to at least `1` at construction, so a bulkhead can
/// always admit one operation. The invariant `in_flight <= capacity` holds on
/// every public path, so [`available`](Self::available) never underflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bulkhead {
    capacity: usize,
    in_flight: usize,
}

impl Bulkhead {
    /// Creates a bulkhead allowing at most `capacity` concurrent permits.
    ///
    /// `capacity` is clamped to a minimum of `1`: a bulkhead always admits at
    /// least one operation, so a `0` would only ever reject and is treated as
    /// `1`.
    pub const fn new(capacity: usize) -> Self {
        let capacity = if capacity == 0 { 1 } else { capacity };
        Self {
            capacity,
            in_flight: 0,
        }
    }

    /// Returns the maximum number of concurrent permits.
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of permits currently held.
    pub const fn in_flight(&self) -> usize {
        self.in_flight
    }

    /// Returns how many more permits can be acquired right now.
    pub const fn available(&self) -> usize {
        self.capacity - self.in_flight
    }

    /// Returns `true` when no further permits are available.
    pub const fn is_full(&self) -> bool {
        self.in_flight >= self.capacity
    }

    /// Returns `true` when no permits are held.
    pub const fn is_empty(&self) -> bool {
        self.in_flight == 0
    }

    /// Tries to acquire `permits` permits at once.
    ///
    /// Returns `true` and reserves them if at least `permits` are available;
    /// otherwise returns `false` and changes nothing (no partial acquire). A
    /// request for more than [`capacity`](Self::capacity) always fails.
    /// Acquiring `0` permits always succeeds and reserves nothing.
    pub fn try_acquire(&mut self, permits: usize) -> bool {
        if permits > self.capacity {
            return false;
        }
        if self.available() >= permits {
            self.in_flight += permits;
            true
        } else {
            false
        }
    }

    /// Tries to acquire a single permit. See [`try_acquire`](Self::try_acquire).
    pub fn try_acquire_one(&mut self) -> bool {
        self.try_acquire(1)
    }

    /// Releases `permits` permits back to the bulkhead.
    ///
    /// Saturates at zero, so releasing more than are held simply empties the
    /// bulkhead rather than underflowing — a release without a matching acquire
    /// cannot drive `in_flight` negative or panic.
    pub fn release(&mut self, permits: usize) {
        self.in_flight = self.in_flight.saturating_sub(permits);
    }

    /// Releases a single permit. See [`release`](Self::release).
    pub fn release_one(&mut self) {
        self.release(1);
    }

    /// Releases every held permit, returning the bulkhead to empty.
    pub fn reset(&mut self) {
        self.in_flight = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_starts_empty() {
        let b = Bulkhead::new(3);
        assert_eq!(b.capacity(), 3);
        assert_eq!(b.in_flight(), 0);
        assert_eq!(b.available(), 3);
        assert!(b.is_empty());
        assert!(!b.is_full());
    }

    #[test]
    fn capacity_clamped_to_one() {
        let mut b = Bulkhead::new(0);
        assert_eq!(b.capacity(), 1);
        assert!(b.try_acquire_one());
        assert!(!b.try_acquire_one());
    }

    #[test]
    fn acquire_until_full_then_reject() {
        let mut b = Bulkhead::new(2);
        assert!(b.try_acquire_one());
        assert!(b.try_acquire_one());
        assert!(b.is_full());
        assert!(!b.try_acquire_one());
        assert_eq!(b.in_flight(), 2);
    }

    #[test]
    fn release_frees_room() {
        let mut b = Bulkhead::new(1);
        assert!(b.try_acquire_one());
        assert!(!b.try_acquire_one());
        b.release_one();
        assert!(b.is_empty());
        assert!(b.try_acquire_one());
    }

    #[test]
    fn batch_acquire_all_or_nothing() {
        let mut b = Bulkhead::new(5);
        assert!(b.try_acquire(3));
        assert_eq!(b.available(), 2);
        // Not enough room for 3 more: nothing is taken.
        assert!(!b.try_acquire(3));
        assert_eq!(b.available(), 2);
        assert!(b.try_acquire(2));
        assert!(b.is_full());
    }

    #[test]
    fn acquire_more_than_capacity_always_fails() {
        let mut b = Bulkhead::new(4);
        assert!(!b.try_acquire(5));
        assert_eq!(b.in_flight(), 0);
    }

    #[test]
    fn acquire_zero_succeeds_and_reserves_nothing() {
        let mut b = Bulkhead::new(2);
        assert!(b.try_acquire(0));
        assert_eq!(b.in_flight(), 0);
    }

    #[test]
    fn release_saturates_at_zero() {
        let mut b = Bulkhead::new(2);
        assert!(b.try_acquire_one());
        b.release(100);
        assert_eq!(b.in_flight(), 0);
        assert!(b.is_empty());
        // A spurious release on an empty bulkhead stays at zero.
        b.release_one();
        assert_eq!(b.in_flight(), 0);
    }

    #[test]
    fn reset_clears_all_permits() {
        let mut b = Bulkhead::new(4);
        assert!(b.try_acquire(3));
        b.reset();
        assert!(b.is_empty());
        assert_eq!(b.available(), 4);
    }

    #[test]
    fn available_never_underflows_at_capacity() {
        let mut b = Bulkhead::new(usize::MAX);
        assert!(b.try_acquire(usize::MAX));
        assert!(b.is_full());
        assert_eq!(b.available(), 0);
        assert!(!b.try_acquire_one());
    }
}

//! The retry policy: how many attempts, and the backoff schedule between them.

use core::time::Duration;

use reliakit_backoff::Backoff;

/// How many times to attempt an operation and how long to wait between tries.
///
/// `max_attempts` counts the *total* number of attempts, including the first
/// one, so `max_attempts = 1` means "try once, never retry" and
/// `max_attempts = 3` means "the first try plus up to two retries". A value of
/// `0` is rejected by [`new`](Self::new).
///
/// The [`Backoff`] supplies the delay *before each retry*. It is consulted only
/// for delay values; the attempt count is governed solely by `max_attempts`, so
/// the two limits never fight. If the backoff yields no delay for a given retry
/// index, [`Duration::ZERO`] is used.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    max_attempts: u32,
    backoff: Backoff,
}

impl RetryPolicy {
    /// Creates a policy that makes at most `max_attempts` attempts, waiting
    /// according to `backoff` between them.
    ///
    /// Returns `None` if `max_attempts` is `0`, which would never run the
    /// operation at all.
    pub const fn new(max_attempts: u32, backoff: Backoff) -> Option<Self> {
        if max_attempts == 0 {
            return None;
        }
        Some(Self {
            max_attempts,
            backoff,
        })
    }

    /// A policy that tries exactly once and never retries.
    ///
    /// Equivalent to `RetryPolicy::new(1, _).unwrap()`; the backoff is never
    /// consulted because there is no retry.
    pub const fn single(backoff: Backoff) -> Self {
        Self {
            max_attempts: 1,
            backoff,
        }
    }

    /// The maximum number of attempts (always `>= 1`).
    pub const fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// The backoff schedule used between attempts.
    pub const fn backoff(&self) -> &Backoff {
        &self.backoff
    }

    /// The delay to wait before the next retry, given how many attempts have
    /// already completed.
    ///
    /// `completed_attempts` is the 1-based number of attempts already made, so
    /// the delay before the first retry is `delay_before_retry(1)`. The backoff
    /// is indexed zero-based (retry `0` is the first retry); if it yields no
    /// delay, [`Duration::ZERO`] is returned.
    pub fn delay_before_retry(&self, completed_attempts: u32) -> Duration {
        let retry_index = completed_attempts.saturating_sub(1);
        self.backoff.delay(retry_index).unwrap_or(Duration::ZERO)
    }
}

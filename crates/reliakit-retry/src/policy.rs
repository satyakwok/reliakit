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
///
/// An optional backoff budget, set with [`with_budget`](Self::with_budget), caps
/// the cumulative delay spent waiting between attempts, stopping early once the
/// next wait would exceed it, independent of `max_attempts`. There is none by
/// default.
#[derive(Debug, Clone, Copy)]
pub struct RetryPolicy {
    max_attempts: u32,
    backoff: Backoff,
    budget: Option<Duration>,
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
            budget: None,
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
            budget: None,
        }
    }

    /// Sets a total backoff budget: a cap on the cumulative delay spent waiting
    /// between attempts. Returns a new policy; the original is unchanged.
    ///
    /// Once the next wait would push the cumulative backoff past `budget`, the
    /// drivers stop and report [`RetryError::Exhausted`](crate::RetryError) instead
    /// of waiting again, even if `max_attempts` is not yet reached. The budget
    /// bounds the **backoff time the policy computes**, not wall-clock time or how
    /// long each attempt runs: this crate reads no clock, so it can only account
    /// for the delays it produces. By default there is no budget.
    pub const fn with_budget(self, budget: Duration) -> Self {
        Self {
            max_attempts: self.max_attempts,
            backoff: self.backoff,
            budget: Some(budget),
        }
    }

    /// The total backoff budget, if one is set. See [`with_budget`](Self::with_budget).
    pub const fn budget(&self) -> Option<Duration> {
        self.budget
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

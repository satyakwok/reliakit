//! Small, runtime-agnostic retry helpers for fallible operations.
//!
//! `reliakit-retry` turns a [`Backoff`] schedule and an attempt limit into a
//! [`RetryPolicy`], then drives a fallible operation against it, synchronously
//! or asynchronously. It is deliberately minimal: it decides *whether* to retry
//! and *how long* the gap should be, but it never sleeps, spawns, or assumes an
//! async runtime. You inject the waiting.
//!
//! It has no third-party dependencies, forbids unsafe code, and is
//! `no_std`-friendly (it needs no allocation and no clock).
//!
//! # Why it does not sleep
//!
//! Blocking the current thread (`std::thread::sleep`) or awaiting a runtime
//! timer is hidden runtime behavior, and it ties a small helper to one
//! execution model. Instead:
//!
//! - [`retry`] runs attempts back-to-back and never waits.
//! - [`retry_with_sleep`] hands each backoff [`Duration`](core::time::Duration)
//!   to a `sleep` closure *you* provide (e.g. one that calls your timer).
//! - [`retry_async`] awaits a `sleep` future *you* provide, so it works under
//!   any executor without depending on Tokio, async-std, or `futures`.
//!
//! # Attempt counting
//!
//! [`RetryPolicy::max_attempts`] is the *total* number of attempts, including
//! the first:
//!
//! - `max_attempts = 1` → try once, never retry (the backoff is never used).
//! - `max_attempts = 3` → the first try plus up to two retries.
//! - `max_attempts = 0` is rejected by [`RetryPolicy::new`] (returns `None`).
//!
//! The attempt count is the single authority for how many times the operation
//! runs. The [`Backoff`] is consulted only for the delay *before each retry*
//! (retry `0` is the first retry, zero-based); if it yields no delay, the gap is
//! [`Duration::ZERO`](core::time::Duration::ZERO). The two limits therefore
//! never conflict.
//!
//! # Retry predicate
//!
//! Every helper takes a `should_retry: FnMut(&E) -> bool` classifier. Returning
//! `false` stops immediately; use it to retry only transient errors and fail
//! fast on permanent ones. It is consulted only when another attempt is actually
//! possible (so it is never called when `max_attempts` is already reached).
//!
//! # Example: sync, no sleeping
//!
//! ```
//! use core::time::Duration;
//! use reliakit_retry::{retry, Backoff, RetryError, RetryPolicy};
//!
//! let policy = RetryPolicy::new(3, Backoff::constant(Duration::from_millis(10))).unwrap();
//!
//! let mut calls = 0;
//! let result: Result<u32, RetryError<&str>> = retry(
//!     &policy,
//!     || {
//!         calls += 1;
//!         if calls < 2 { Err("temporary") } else { Ok(42) }
//!     },
//!     |_error| true, // retry every error
//! );
//!
//! assert_eq!(result.unwrap(), 42);
//! assert_eq!(calls, 2);
//! ```
//!
//! # Example: sync, with an injected sleeper
//!
//! ```
//! use core::time::Duration;
//! use reliakit_retry::{retry_with_sleep, Backoff, RetryError, RetryPolicy};
//!
//! let policy = RetryPolicy::new(4, Backoff::exponential(Duration::from_millis(1), 2)).unwrap();
//!
//! // Record the delays instead of really sleeping (a real caller would wait).
//! let mut waited: Vec<Duration> = Vec::new();
//! let mut attempts = 0;
//! let result: Result<(), RetryError<&str>> = retry_with_sleep(
//!     &policy,
//!     || { attempts += 1; Err("always fails") },
//!     |_error| true,
//!     |delay| waited.push(delay),
//! );
//!
//! assert!(matches!(result, Err(RetryError::Exhausted { attempts: 4, .. })));
//! // Three gaps before retries 2, 3, 4: 1ms, 2ms, 4ms.
//! assert_eq!(waited, [Duration::from_millis(1), Duration::from_millis(2), Duration::from_millis(4)]);
//! ```
//!
//! For the async helper, see [`retry_async`] and the `async_retry` example,
//! which drives it without any runtime.
//!
//! # Observing retries
//!
//! To log or count retries, use [`retry_with_sleep_observed`] (or
//! [`retry_async_observed`]). They take the same arguments plus an
//! `on_retry: FnMut(u32, Duration, &E)` hook called just before each wait, with
//! the failed attempt's number, the delay about to be waited, and the error that
//! triggered the retry. It fires only when another attempt will be made (not on
//! success, and not on the final failure that exhausts the policy), and it
//! allocates nothing. The crate still does no logging itself; the hook is yours.
//!
//! ```
//! use core::time::Duration;
//! use reliakit_retry::{retry_with_sleep_observed, Backoff, RetryError, RetryPolicy};
//!
//! let policy = RetryPolicy::new(3, Backoff::constant(Duration::from_millis(10))).unwrap();
//!
//! let mut seen: Vec<(u32, Duration)> = Vec::new();
//! let mut calls = 0;
//! let result: Result<u32, RetryError<&str>> = retry_with_sleep_observed(
//!     &policy,
//!     || {
//!         calls += 1;
//!         if calls < 2 { Err("temporary") } else { Ok(42) }
//!     },
//!     |_error| true,
//!     |_delay| {},                       // your sleeper
//!     |attempt, delay, _error| seen.push((attempt, delay)),
//! );
//!
//! assert_eq!(result.unwrap(), 42);
//! assert_eq!(seen, [(1, Duration::from_millis(10))]); // observed the one retry
//! ```
//!
//! # Feature flags
//!
//! - `std` (default) adds `impl std::error::Error for RetryError`. With
//!   `--no-default-features` the crate is pure `core`: no allocation, no clock,
//!   no runtime.
//!
//! # What this is not
//!
//! This is a small retry helper, not a framework, middleware stack, async
//! runtime, or a Tower replacement. It does not log, spawn, time, or schedule on
//! your behalf.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod policy;
mod retry;

pub use error::RetryError;
pub use policy::RetryPolicy;
pub use retry::{
    retry, retry_async, retry_async_observed, retry_with_sleep, retry_with_sleep_observed,
};

/// Re-exported from `reliakit-backoff` so the backoff schedule is reachable
/// without a separate dependency line.
pub use reliakit_backoff::Backoff;

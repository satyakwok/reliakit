//! The sync and async retry drivers.

use core::future::Future;
use core::time::Duration;

use crate::error::RetryError;
use crate::policy::RetryPolicy;

/// Retries `op` according to `policy`, **without ever sleeping**.
///
/// Each attempt is run back-to-back; the backoff delays are computed but not
/// waited on. Use this when retries are cheap and immediate, or when you only
/// want the attempt-bounding behavior. For spacing between attempts, use
/// [`retry_with_sleep`] (sync) or [`retry_async`] (async) and inject your own
/// delay.
///
/// On the first `Ok`, that value is returned immediately. If an attempt fails,
/// `should_retry` decides whether the error is worth retrying; returning `false`
/// stops at once.
///
/// # Errors
///
/// Returns [`RetryError::Exhausted`] when no attempt succeeds — either the
/// `max_attempts` limit was reached or `should_retry` returned `false`. The
/// error carries the number of attempts made and the final error.
pub fn retry<T, E, Op, ShouldRetry>(
    policy: &RetryPolicy,
    op: Op,
    should_retry: ShouldRetry,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Result<T, E>,
    ShouldRetry: FnMut(&E) -> bool,
{
    retry_with_sleep(policy, op, should_retry, |_delay| {})
}

/// Retries `op` according to `policy`, calling `sleep` with the backoff delay
/// before each retry.
///
/// The crate never sleeps on its own: `sleep` is your delay mechanism. It
/// receives the [`Duration`] to wait before the next attempt. The crate does not
/// call `std::thread::sleep` or any timer itself.
///
/// # Errors
///
/// Returns [`RetryError::Exhausted`] when no attempt succeeds — either the
/// `max_attempts` limit was reached or `should_retry` returned `false`.
pub fn retry_with_sleep<T, E, Op, ShouldRetry, Sleep>(
    policy: &RetryPolicy,
    mut op: Op,
    mut should_retry: ShouldRetry,
    mut sleep: Sleep,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Result<T, E>,
    ShouldRetry: FnMut(&E) -> bool,
    Sleep: FnMut(Duration),
{
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        match op() {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt >= policy.max_attempts() || !should_retry(&error) {
                    return Err(RetryError::Exhausted {
                        attempts: attempt,
                        last_error: error,
                    });
                }
                sleep(policy.delay_before_retry(attempt));
            }
        }
    }
}

/// Retries an async `op` according to `policy`, awaiting `sleep` before each
/// retry.
///
/// This is runtime-agnostic: it does not depend on Tokio, async-std, or the
/// `futures` crate, and it does not spawn anything. You supply `op` (which
/// returns a future) and `sleep` (which maps a [`Duration`] to a future that
/// completes after that long under *your* runtime). Both futures are awaited in
/// place on the caller's task.
///
/// # Errors
///
/// Returns [`RetryError::Exhausted`] when no attempt succeeds — either the
/// `max_attempts` limit was reached or `should_retry` returned `false`.
pub async fn retry_async<T, E, Op, Fut, ShouldRetry, Sleep, SleepFut>(
    policy: &RetryPolicy,
    mut op: Op,
    mut should_retry: ShouldRetry,
    mut sleep: Sleep,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: FnMut(&E) -> bool,
    Sleep: FnMut(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        match op().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                if attempt >= policy.max_attempts() || !should_retry(&error) {
                    return Err(RetryError::Exhausted {
                        attempts: attempt,
                        last_error: error,
                    });
                }
                sleep(policy.delay_before_retry(attempt)).await;
            }
        }
    }
}

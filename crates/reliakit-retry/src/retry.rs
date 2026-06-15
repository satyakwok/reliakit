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
    op: Op,
    should_retry: ShouldRetry,
    sleep: Sleep,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Result<T, E>,
    ShouldRetry: FnMut(&E) -> bool,
    Sleep: FnMut(Duration),
{
    retry_with_sleep_observed(policy, op, should_retry, sleep, |_, _, _| {})
}

/// Like [`retry_with_sleep`], but also calls `on_retry` just before each wait.
///
/// `on_retry` receives the number of the attempt that just failed, the
/// [`Duration`] that is about to be waited, and a reference to the error that
/// triggered the retry. It runs only when another attempt will actually be made:
/// not on success, and not on the final failure that exhausts the policy. Use it
/// for logging or metrics — the crate itself still logs nothing and allocates
/// nothing.
///
/// To observe the no-sleep driver, pass a no-op sleeper:
/// `retry_with_sleep_observed(policy, op, should_retry, |_| {}, on_retry)`.
///
/// # Errors
///
/// Returns [`RetryError::Exhausted`] when no attempt succeeds — either the
/// `max_attempts` limit was reached or `should_retry` returned `false`.
pub fn retry_with_sleep_observed<T, E, Op, ShouldRetry, Sleep, OnRetry>(
    policy: &RetryPolicy,
    mut op: Op,
    mut should_retry: ShouldRetry,
    mut sleep: Sleep,
    mut on_retry: OnRetry,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Result<T, E>,
    ShouldRetry: FnMut(&E) -> bool,
    Sleep: FnMut(Duration),
    OnRetry: FnMut(u32, Duration, &E),
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
                let delay = policy.delay_before_retry(attempt);
                on_retry(attempt, delay, &error);
                sleep(delay);
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
    op: Op,
    should_retry: ShouldRetry,
    sleep: Sleep,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: FnMut(&E) -> bool,
    Sleep: FnMut(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
{
    retry_async_observed(policy, op, should_retry, sleep, |_, _, _| {}).await
}

/// Like [`retry_async`], but also calls `on_retry` just before awaiting each
/// wait.
///
/// `on_retry` receives the number of the attempt that just failed, the
/// [`Duration`] about to be awaited, and a reference to the error that triggered
/// the retry. It runs only when another attempt will actually be made, and is a
/// plain (non-async) `FnMut`, so it cannot accidentally introduce hidden awaits.
/// Use it for logging or metrics; the crate still logs nothing and allocates
/// nothing.
///
/// # Errors
///
/// Returns [`RetryError::Exhausted`] when no attempt succeeds — either the
/// `max_attempts` limit was reached or `should_retry` returned `false`.
pub async fn retry_async_observed<T, E, Op, Fut, ShouldRetry, Sleep, SleepFut, OnRetry>(
    policy: &RetryPolicy,
    mut op: Op,
    mut should_retry: ShouldRetry,
    mut sleep: Sleep,
    mut on_retry: OnRetry,
) -> Result<T, RetryError<E>>
where
    Op: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    ShouldRetry: FnMut(&E) -> bool,
    Sleep: FnMut(Duration) -> SleepFut,
    SleepFut: Future<Output = ()>,
    OnRetry: FnMut(u32, Duration, &E),
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
                let delay = policy.delay_before_retry(attempt);
                on_retry(attempt, delay, &error);
                sleep(delay).await;
            }
        }
    }
}

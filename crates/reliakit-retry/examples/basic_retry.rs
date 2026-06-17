//! Sync retry: retry only temporary failures, with a user-provided sleeper.
//!
//! Run with: `cargo run -p reliakit-retry --example basic_retry`

use core::time::Duration;

use reliakit_retry::{Backoff, RetryError, RetryPolicy, retry_with_sleep};

#[derive(Debug)]
enum ApiError {
    /// A transient failure worth retrying.
    Temporary,
    /// A permanent failure; retrying will not help.
    Fatal,
}

fn main() {
    let policy = RetryPolicy::new(
        5,
        Backoff::exponential(Duration::from_millis(50), 2).with_max_delay(Duration::from_secs(1)),
    )
    .expect("max_attempts is non-zero");

    // An operation that fails twice with a temporary error, then succeeds.
    let mut attempt = 0;
    let result: Result<&str, RetryError<ApiError>> = retry_with_sleep(
        &policy,
        || {
            attempt += 1;
            println!("attempt {attempt}");
            if attempt < 3 {
                Err(ApiError::Temporary)
            } else {
                Ok("payload")
            }
        },
        |error| matches!(error, ApiError::Temporary), // retry only temporary errors
        |delay| {
            // You provide the waiting. In real code, call your platform or
            // runtime sleep here; this example only reports the delay so it
            // stays dependency-free and instant.
            println!("  would wait {delay:?} before the next attempt");
        },
    );
    match result {
        Ok(body) => println!("succeeded with: {body}"),
        Err(error) => println!("gave up: {error:?}"),
    }

    // A fatal error stops immediately, even though attempts remain.
    let mut attempt = 0;
    let result: Result<&str, RetryError<ApiError>> = retry_with_sleep(
        &policy,
        || {
            attempt += 1;
            Err(ApiError::Fatal)
        },
        |error| matches!(error, ApiError::Temporary),
        |_delay| {},
    );
    println!(
        "fatal path: stopped after {} attempt(s)",
        result.unwrap_err().attempts()
    );
}

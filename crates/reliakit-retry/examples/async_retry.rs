//! Async retry with no runtime: the sleep future is user-provided, and a tiny
//! in-file executor drives the result to completion.
//!
//! Run with: `cargo run -p reliakit-retry --example async_retry`
//!
//! `reliakit-retry` does not depend on Tokio, async-std, or `futures`. Under a
//! real runtime you would replace `block_on` with your executor and the sleep
//! future with your runtime's async timer.

use core::future::Future;
use core::task::{Context, Poll, Waker};
use core::time::Duration;

use reliakit_retry::{retry_async, Backoff, RetryError, RetryPolicy};

/// Polls a future to completion on the current thread. The futures here are
/// always immediately ready, so this never spins.
fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut future = core::pin::pin!(future);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
    }
}

fn main() {
    let policy = RetryPolicy::new(4, Backoff::constant(Duration::from_millis(20)))
        .expect("max_attempts is non-zero");

    let mut attempt = 0;
    let result: Result<u32, RetryError<&str>> = block_on(retry_async(
        &policy,
        || {
            attempt += 1;
            let outcome = if attempt < 3 {
                Err("temporary")
            } else {
                Ok(200)
            };
            async move { outcome }
        },
        |_error| true,
        |delay| async move {
            // Your runtime's async sleep goes here. This example resolves
            // immediately; the sleep future is entirely user-provided.
            let _ = delay;
        },
    ));

    println!("async result after {attempt} attempt(s): {result:?}");
}

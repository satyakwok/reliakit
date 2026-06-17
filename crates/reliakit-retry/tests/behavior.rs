//! Deterministic behavior tests. No real time is used: delays are recorded
//! through an injected sleeper and async is driven by a tiny in-test executor.

use core::future::Future;
use core::task::{Context, Poll, Waker};
use core::time::Duration;

use reliakit_retry::{
    Backoff, RetryError, RetryPolicy, retry, retry_async, retry_async_observed, retry_with_sleep,
    retry_with_sleep_observed,
};

/// Minimal executor: polls a future to completion on the current thread. The
/// futures under test are always immediately ready, so this never spins.
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

fn policy(max_attempts: u32) -> RetryPolicy {
    RetryPolicy::new(max_attempts, Backoff::constant(Duration::from_millis(5))).unwrap()
}

// 1. success on first attempt
#[test]
fn succeeds_on_first_attempt() {
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry(
        &policy(3),
        || {
            calls += 1;
            Ok(7)
        },
        |_| true,
    );
    assert_eq!(result.unwrap(), 7);
    assert_eq!(calls, 1, "must not retry after success");
}

// 2. success after retry
#[test]
fn succeeds_after_retries() {
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry(
        &policy(5),
        || {
            calls += 1;
            if calls < 3 { Err("temporary") } else { Ok(99) }
        },
        |_| true,
    );
    assert_eq!(result.unwrap(), 99);
    assert_eq!(calls, 3);
}

// 3. exhausted max attempts
#[test]
fn exhausts_max_attempts() {
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry(
        &policy(4),
        || {
            calls += 1;
            Err("always")
        },
        |_| true,
    );
    match result {
        Err(RetryError::Exhausted {
            attempts,
            last_error,
        }) => {
            assert_eq!(attempts, 4);
            assert_eq!(last_error, "always");
        }
        other => panic!("expected Exhausted, got {other:?}"),
    }
    assert_eq!(calls, 4);
}

// 4. should_retry false stops immediately
// 5. should_retry true keeps retrying
#[test]
fn should_retry_controls_continuation() {
    // Fatal error: stop at once even though attempts remain.
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry(
        &policy(5),
        || {
            calls += 1;
            Err("fatal")
        },
        |error| *error != "fatal",
    );
    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 1, .. })
    ));
    assert_eq!(calls, 1, "fatal error must not be retried");

    // Transient errors are retried until success.
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry(
        &policy(5),
        || {
            calls += 1;
            if calls < 4 { Err("transient") } else { Ok(1) }
        },
        |error| *error == "transient",
    );
    assert_eq!(result.unwrap(), 1);
    assert_eq!(calls, 4);
}

// 6. backoff delay values are used (via injected sleeper)
#[test]
fn backoff_delays_are_passed_to_sleeper() {
    let backoff = Backoff::exponential(Duration::from_millis(1), 2);
    let policy = RetryPolicy::new(4, backoff).unwrap();

    let mut waited: Vec<Duration> = Vec::new();
    let result: Result<(), RetryError<u8>> =
        retry_with_sleep(&policy, || Err(1u8), |_| true, |delay| waited.push(delay));

    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 4, .. })
    ));
    // Gaps before retries 2, 3, 4 are the backoff's delays for indices 0, 1, 2.
    assert_eq!(
        waited,
        [
            backoff.delay(0).unwrap(),
            backoff.delay(1).unwrap(),
            backoff.delay(2).unwrap(),
        ]
    );
    assert_eq!(
        waited,
        [
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(4),
        ]
    );
}

// 9. sync retry does not sleep unless a sleeper is injected
#[test]
fn plain_retry_needs_no_sleeper() {
    // `retry` runs to exhaustion with no sleep mechanism at all.
    let mut calls = 0;
    let result: Result<(), RetryError<()>> = retry(
        &policy(3),
        || {
            calls += 1;
            Err(())
        },
        |_| true,
    );
    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 3, .. })
    ));
    assert_eq!(calls, 3);
}

// 8. invalid max attempts rejected; single() helper
#[test]
fn zero_attempts_rejected_and_single_tries_once() {
    assert!(RetryPolicy::new(0, Backoff::constant(Duration::ZERO)).is_none());

    let policy = RetryPolicy::single(Backoff::constant(Duration::from_secs(99)));
    assert_eq!(policy.max_attempts(), 1);

    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry(
        &policy,
        || {
            calls += 1;
            Err("nope")
        },
        |_| true, // would retry, but max_attempts = 1 forbids it
    );
    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 1, .. })
    ));
    assert_eq!(calls, 1, "single() must try exactly once");
}

// 10. async retry works with a fake (immediately-ready) sleep future
#[test]
fn async_retry_drives_with_fake_sleep() {
    let mut waited: Vec<Duration> = Vec::new();
    let mut calls = 0;

    let result: Result<u32, RetryError<&str>> = block_on(retry_async(
        &policy(5),
        || {
            calls += 1;
            // Each call returns a ready future with the attempt's result.
            let outcome = if calls < 3 { Err("temporary") } else { Ok(55) };
            core::future::ready(outcome)
        },
        |_| true,
        |delay| {
            waited.push(delay);
            core::future::ready(())
        },
    ));

    assert_eq!(result.unwrap(), 55);
    assert_eq!(calls, 3);
    // Two gaps before attempts 2 and 3, each the constant 5ms.
    assert_eq!(waited, [Duration::from_millis(5), Duration::from_millis(5)]);
}

// 11. integration with the current reliakit-backoff API surface
#[test]
fn integrates_with_backoff_constructors() {
    for backoff in [
        Backoff::constant(Duration::from_millis(2)),
        Backoff::linear(Duration::from_millis(1), Duration::from_millis(3)),
        Backoff::exponential(Duration::from_millis(1), 3).with_max_delay(Duration::from_millis(10)),
    ] {
        let policy = RetryPolicy::new(3, backoff).unwrap();
        let result: Result<u32, RetryError<&str>> =
            retry_with_sleep(&policy, || Ok(0), |_| true, |_| {});
        assert_eq!(result.unwrap(), 0);
        // delay_before_retry mirrors the backoff's own schedule.
        assert_eq!(
            policy.delay_before_retry(1),
            backoff.delay(0).unwrap_or(Duration::ZERO)
        );
    }
}

// RetryError accessors.
#[test]
fn retry_error_accessors() {
    let err: RetryError<&str> = RetryError::Exhausted {
        attempts: 2,
        last_error: "boom",
    };
    assert_eq!(err.attempts(), 2);
    assert_eq!(*err.last_error(), "boom");
    assert_eq!(err.into_last_error(), "boom");
}

// RetryError Display and std::error::Error::source.
// `source` exists only when the `std` feature enables the `Error` impl.
#[cfg(feature = "std")]
#[test]
fn retry_error_display_and_source() {
    use std::error::Error;

    #[derive(Debug)]
    struct Inner;
    impl core::fmt::Display for Inner {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("inner failure")
        }
    }
    impl Error for Inner {}

    let err: RetryError<Inner> = RetryError::Exhausted {
        attempts: 3,
        last_error: Inner,
    };
    let text = format!("{err}");
    assert!(
        text.contains('3'),
        "message names the attempt count: {text}"
    );
    assert!(
        text.contains("inner failure"),
        "message includes the cause: {text}"
    );
    assert!(
        err.source().is_some(),
        "the cause is exposed as the error source"
    );
}

// on_retry hook records the (attempt, delay, error) sequence for a
// fails-then-succeeds operation.
#[test]
fn observed_hook_records_retry_sequence() {
    let policy = RetryPolicy::new(5, Backoff::exponential(Duration::from_millis(1), 2)).unwrap();
    let mut seen: Vec<(u32, Duration, &str)> = Vec::new();
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = retry_with_sleep_observed(
        &policy,
        || {
            calls += 1;
            if calls < 3 { Err("temporary") } else { Ok(7) }
        },
        |_| true,
        |_delay| {},
        |attempt, delay, error| seen.push((attempt, delay, *error)),
    );
    assert_eq!(result.unwrap(), 7);
    assert_eq!(calls, 3);
    // Fired before retries 2 and 3, with the backoff's delays for indices 0 and 1.
    assert_eq!(
        seen,
        [
            (1, Duration::from_millis(1), "temporary"),
            (2, Duration::from_millis(2), "temporary"),
        ]
    );
}

// The hook fires for each retry but NOT for the final failure that exhausts
// the policy.
#[test]
fn observed_hook_skips_exhausting_failure() {
    let mut seen_attempts: Vec<u32> = Vec::new();
    let result: Result<(), RetryError<u8>> = retry_with_sleep_observed(
        &policy(3),
        || Err(1u8),
        |_| true,
        |_| {},
        |attempt, _delay, _error| seen_attempts.push(attempt),
    );
    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 3, .. })
    ));
    // Three attempts, but the hook only precedes retries 2 and 3.
    assert_eq!(seen_attempts, [1, 2]);
}

// No retry means no observation: a fatal error fails fast without the hook.
#[test]
fn observed_hook_not_called_when_not_retried() {
    let mut count = 0;
    let result: Result<(), RetryError<&str>> = retry_with_sleep_observed(
        &policy(5),
        || Err("fatal"),
        |_| false,
        |_| {},
        |_, _, _| count += 1,
    );
    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 1, .. })
    ));
    assert_eq!(count, 0, "an un-retried error must not be observed");
}

// The async observed driver records the same sequence, driven by the fake sleep.
#[test]
fn async_observed_hook_records_sequence() {
    let mut seen: Vec<(u32, Duration)> = Vec::new();
    let mut calls = 0;
    let result: Result<u32, RetryError<&str>> = block_on(retry_async_observed(
        &policy(5),
        || {
            calls += 1;
            let outcome = if calls < 3 { Err("temp") } else { Ok(9) };
            core::future::ready(outcome)
        },
        |_| true,
        |_delay| core::future::ready(()),
        |attempt, delay, _error| seen.push((attempt, delay)),
    ));
    assert_eq!(result.unwrap(), 9);
    assert_eq!(calls, 3);
    assert_eq!(
        seen,
        [(1, Duration::from_millis(5)), (2, Duration::from_millis(5)),]
    );
}

// Policy accessors mirror the configured values.
#[test]
fn policy_accessors() {
    let backoff = Backoff::constant(Duration::from_millis(7));
    let policy = RetryPolicy::new(3, backoff).unwrap();
    assert_eq!(policy.max_attempts(), 3);
    assert_eq!(policy.backoff().delay(0), backoff.delay(0));
    assert_eq!(policy.delay_before_retry(1), Duration::from_millis(7));
}

// Async retry exercises the exhaustion path and awaits the sleep between tries.
#[test]
fn async_retry_exhausts() {
    let mut calls = 0;
    let mut sleeps = 0;
    let result: Result<u32, RetryError<&str>> = block_on(retry_async(
        &policy(3),
        || {
            calls += 1;
            core::future::ready(Err("nope"))
        },
        |_| true,
        |_delay| {
            sleeps += 1;
            core::future::ready(())
        },
    ));
    assert!(matches!(
        result,
        Err(RetryError::Exhausted { attempts: 3, .. })
    ));
    assert_eq!(calls, 3);
    assert_eq!(sleeps, 2, "slept before retries 2 and 3");
}

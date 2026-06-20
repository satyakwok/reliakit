# Retry with backoff

## Problem

A call to a dependency fails sometimes for transient reasons (a blip, a brief
overload). Blind immediate retries make it worse; an ad-hoc `for` loop with
`thread::sleep` bakes a runtime and a sleep strategy into code that should not
care. You want a retry policy that decides *whether* and *how long*, while you
keep control of the actual waiting.

## Use

- `reliakit-retry`: the retry loop: counts attempts, asks `should_retry`, and
  requests a delay. It never sleeps itself.
- `reliakit-backoff`: computes the delay schedule (exponential, capped, jitter).

## Example

```rust
use core::time::Duration;
use reliakit_retry::{Backoff, RetryError, RetryPolicy, retry_with_sleep};

#[derive(Debug)]
enum ApiError {
    Temporary, // worth retrying
    Fatal,     // retrying will not help
}

fn main() {
    // Up to 5 attempts; 50ms base, doubling, capped at 1s.
    let policy = RetryPolicy::new(
        5,
        Backoff::exponential(Duration::from_millis(50), 2).with_max_delay(Duration::from_secs(1)),
    )
    .expect("max_attempts is non-zero");

    let mut attempt = 0;
    let result: Result<&str, RetryError<ApiError>> = retry_with_sleep(
        &policy,
        || {
            attempt += 1;
            if attempt < 3 { Err(ApiError::Temporary) } else { Ok("payload") }
        },
        |error| matches!(error, ApiError::Temporary), // retry only transient errors
        |delay| {
            // You provide the waiting. In real code: std::thread::sleep(delay),
            // or under async, await your runtime's timer for `delay`.
            let _ = delay;
        },
    );

    assert_eq!(result.unwrap(), "payload");
}
```

A `Fatal` error stops immediately because `should_retry` returns `false`, even
though attempts remain.

## Run it

```sh
cargo run -p reliakit-retry --example basic_retry
cargo run -p reliakit-retry --example async_retry
cargo run -p reliakit-backoff --example basic
```

## Why this works

The crate computes the retry decision and the delay, then hands the delay to your
sleeper. Because it never sleeps or spawns, the same policy runs under sync code,
any async runtime, tests (inject an instant sleeper), and `no_std`. `should_retry`
keeps you from retrying errors that cannot succeed.

## Common mistakes

- **Retrying non-idempotent operations.** A retried "charge card" or "POST" can
  double-apply. Only retry idempotent calls, or carry an idempotency key.
- **Retrying fatal errors.** A 400 or "not found" will fail every time. Gate with
  `should_retry`.
- **No cap and no jitter.** Unbounded exponential growth and synchronized retries
  cause thundering herds. Use `with_max_delay`, and add `full_jitter` from
  `reliakit-backoff` when many clients retry together.
- **Sleeping inside a reusable library.** Take a sleeper (sync) or an async sleep
  future from the caller instead, the way this crate does.

## When not to use this

- Do not retry writes that are not idempotent without an idempotency key.
- Retry does not bound *total* time on its own. Pair it with `reliakit-timeout`
  (an overall deadline) when you need a wall-clock budget.
- If failures are sustained (not transient), add `reliakit-circuit` so you stop
  hammering a dependency that is already down.

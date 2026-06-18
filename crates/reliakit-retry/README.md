<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-retry

[![Crates.io](https://img.shields.io/crates/v/reliakit-retry.svg)](https://crates.io/crates/reliakit-retry)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-retry.svg)](https://crates.io/crates/reliakit-retry)
[![Docs.rs](https://docs.rs/reliakit-retry/badge.svg)](https://docs.rs/reliakit-retry)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-retry)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-retry)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

A small, runtime-agnostic retry helper for Rust operations that may fail
temporarily.

`reliakit-retry` turns a [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff)
schedule and an attempt limit into a `RetryPolicy`, then drives a fallible
operation against it, synchronously or asynchronously. It decides *whether* to
retry and *how long* the gap should be, but it never sleeps, spawns, or assumes
an async runtime: you inject the waiting.

It has no third-party dependencies, forbids unsafe code, and is `no_std`-friendly
(no allocation, no clock).

## What problem it solves

Transient failures (a flaky network call, a momentarily busy resource) are
worth retrying a few times with growing gaps. Writing that loop by hand each time
means re-deriving attempt counting, backoff, and "is this error even worth
retrying" every time. This crate is that loop, made explicit and reusable, with
no opinion about how you wait.

## When to use it

- You retry a fallible operation and want clear attempt limits and backoff.
- You want to retry only *some* errors (transient) and fail fast on others.
- You want one retry helper that works in sync code, async code, and `no_std`.

## When not to use it

- You want a framework, middleware stack, or a Tower-style layer system. This is
  a single function, not infrastructure.
- You want the crate to sleep, spawn, log, or schedule for you. It does none of
  that by design; you provide the sleeper.

## Installation

```toml
[dependencies]
reliakit-retry = "1.0"
```

For `no_std` (pure `core`, no `std::error::Error` impl):

```toml
[dependencies]
reliakit-retry = { version = "1.0", default-features = false }
```

## Basic sync example

```rust
use core::time::Duration;
use reliakit_retry::{retry_with_sleep, Backoff, RetryError, RetryPolicy};

let policy = RetryPolicy::new(
    5,
    Backoff::exponential(Duration::from_millis(50), 2).with_max_delay(Duration::from_secs(1)),
)
.unwrap();

let mut attempt = 0;
let result: Result<&str, RetryError<&str>> = retry_with_sleep(
    &policy,
    || {
        attempt += 1;
        if attempt < 3 { Err("temporary") } else { Ok("ok") }
    },
    |error| *error == "temporary", // retry only temporary errors
    |delay| {
        // You provide the waiting. In real code, call your platform sleep here.
        let _ = delay;
    },
);
assert_eq!(result.unwrap(), "ok");
```

Use `retry(&policy, op, should_retry)` for the same logic with no waiting at all.

## Async example (user-provided sleep, no runtime)

```rust
use core::time::Duration;
use reliakit_retry::{retry_async, Backoff, RetryError, RetryPolicy};

# async fn run() {
let policy = RetryPolicy::new(4, Backoff::constant(Duration::from_millis(20))).unwrap();

let mut attempt = 0;
let result: Result<u32, RetryError<&str>> = retry_async(
    &policy,
    || {
        attempt += 1;
        let outcome = if attempt < 3 { Err("temporary") } else { Ok(200) };
        async move { outcome }
    },
    |_error| true,
    // Your runtime's async sleep goes here (e.g. tokio::time::sleep(delay)).
    |delay| async move { let _ = delay; },
)
.await;
assert_eq!(result.unwrap(), 200);
# }
```

`retry_async` uses only `core::future::Future`; it does not depend on Tokio,
async-std, or `futures`. The `async_retry` example shows it running under a tiny
in-file executor with no runtime at all.

## Observing retries (logging, metrics)

To watch retries without changing the basic calls, use `retry_with_sleep_observed`
or `retry_async_observed`. They take the same arguments plus an
`on_retry: FnMut(u32, Duration, &E)` hook called just before each wait, with the
failed attempt's number, the delay about to be waited, and the error that
triggered the retry:

```rust
use core::time::Duration;
use reliakit_retry::{retry_with_sleep_observed, Backoff, RetryError, RetryPolicy};

let policy = RetryPolicy::new(3, Backoff::constant(Duration::from_millis(10))).unwrap();

let mut calls = 0;
let result: Result<u32, RetryError<&str>> = retry_with_sleep_observed(
    &policy,
    || { calls += 1; if calls < 2 { Err("temporary") } else { Ok(42) } },
    |_error| true,
    |_delay| {},                                   // your sleeper
    |attempt, delay, error| {
        eprintln!("retry #{attempt} after {delay:?}: {error}");
    },
);
assert_eq!(result.unwrap(), 42);
```

The hook fires only when another attempt will be made (not on success, and not
on the final failure that exhausts the policy), and it **allocates nothing**. The
crate still does no logging itself; the hook is yours. (To observe the no-sleep
driver, pass a no-op sleeper.)

## Attempt counting

`max_attempts` is the **total** number of attempts, including the first:

- `max_attempts = 1` → try once, never retry.
- `max_attempts = 3` → the first try plus up to two retries.
- `max_attempts = 0` → rejected by `RetryPolicy::new` (returns `None`).

The attempt count is the single authority for how many times the operation runs.
The `Backoff` is consulted only for the delay before each retry; if it yields no
delay, `Duration::ZERO` is used, so the two limits never conflict.

## Feature flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Adds `impl std::error::Error for RetryError`. Otherwise the crate is pure `core`. |

## `no_std`

`reliakit-retry` is `no_std`-friendly and needs no allocation and no clock. With
`--no-default-features` it builds for bare-metal targets; only the
`std::error::Error` impl is gated off.

## Design notes

- **It never sleeps.** Blocking a thread or awaiting a runtime timer is hidden
  behavior and ties the helper to one execution model. You pass the sleeper.
- **Clock-agnostic.** Delays come from the backoff schedule by attempt index, so
  no wall-clock is read; `reliakit-core::Clock` is not needed.
- **Runtime-agnostic async.** The async helper awaits a future you supply, so it
  runs under any executor.
- **Built on the family.** The backoff schedule is `reliakit-backoff::Backoff`,
  re-exported here as `reliakit_retry::Backoff`.

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Status

Pre-1.0. The API is small and stable; it may receive backward-compatible
refinements before a `1.0` release. Minimum supported Rust version: `1.85`.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-ratelimit

[![Crates.io](https://img.shields.io/crates/v/reliakit-ratelimit.svg)](https://crates.io/crates/reliakit-ratelimit)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-ratelimit.svg)](https://crates.io/crates/reliakit-ratelimit)
[![Docs.rs](https://docs.rs/reliakit-ratelimit/badge.svg)](https://docs.rs/reliakit-ratelimit)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-ratelimit)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-ratelimit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

A clock-agnostic **token-bucket rate limiter** for Rust.

A token bucket is a simple, well-understood way to cap how often something may
happen. The bucket holds up to `capacity` tokens and gains `refill_amount`
tokens every `refill_interval`; each request spends one or more tokens, and when
the bucket is empty requests are denied until it refills. Two numbers describe
the whole policy:

- **capacity** — the largest burst you will allow at once.
- **refill rate** (`refill_amount` per `refill_interval`) — the sustained rate
  once the burst is spent.

`reliakit-ratelimit` implements this with **no dependencies**, no `std`, and no
hidden behavior. It does not read the clock, sleep, spawn, or allocate — *you*
pass the current time in, and *you* decide what to do when a request is denied.
That makes it equally usable from synchronous code, any async runtime, and
`no_std` / embedded targets, and every decision is deterministic and trivial to
unit-test. All arithmetic is integer-only and saturating, so no call can
overflow or panic.

## Why "clock-agnostic"?

Most rate limiters reach for `std::time::Instant` or a runtime timer. This one
takes the current time as a `u64` argument in whatever monotonic unit you choose
(milliseconds is typical) and uses that same unit for `refill_interval`:

- **Runtime-neutral** — Tokio, async-std, blocking threads, or a bare-metal loop.
- **`no_std`-friendly** — CI builds it for `thumbv7em-none-eabi`.
- **Deterministic & testable** — behavior depends only on the timestamps you
  pass; tests assert exact token counts and wait times with no sleeping.

A clock that briefly moves backwards is handled with saturating arithmetic (no
refill happens, nothing panics).

## Installation

```toml
[dependencies]
reliakit-ratelimit = "0.1"
```

This crate is `#![no_std]` with no required dependencies. It has one optional
feature, `core` (off by default), which pulls in `reliakit-core` and adds
`*_now(clock)` convenience methods on `RateLimiter` backed by its `Clock` trait;
the existing `now: u64` methods are unchanged.

## Usage

The bucket starts full, so an initial burst up to `capacity` is allowed:

```rust
use reliakit_ratelimit::RateLimiter;

// Capacity 10, refill 1 token every 100ms — about 10 requests/second sustained,
// with bursts of up to 10.
let mut limiter = RateLimiter::new(10, 1, 100);

let now = now_millis(); // your own monotonic clock, in milliseconds
if limiter.try_acquire_one(now) {
    // proceed with the request
} else {
    // over the limit — drop it, queue it, or tell the caller to back off
}
```

Take several tokens at once (e.g. a request that costs more), and tell a caller
when to come back:

```rust
use reliakit_ratelimit::RateLimiter;

let mut limiter = RateLimiter::new(100, 10, 1_000);
let now = now_millis();

if !limiter.try_acquire(now, 5) {
    if let Some(wait_ms) = limiter.retry_after(now, 5) {
        // e.g. set a `Retry-After` header to `wait_ms`
    }
}
```

See [`examples/basic.rs`](./examples/basic.rs) for a complete loop that bursts,
gets throttled, and recovers.

## API

| Method | Purpose |
|---|---|
| `RateLimiter::new(capacity, refill_amount, refill_interval)` | Construct a limiter (const fn). Starts full. |
| `try_acquire(now, tokens) -> bool` | Take `tokens` if available; consumes nothing on failure. |
| `try_acquire_one(now) -> bool` | Take a single token. |
| `available(now) -> u64` | Tokens available now, after refilling. |
| `retry_after(now, tokens) -> Option<u64>` | Time until `tokens` are available (`Some(0)` if now; `None` if `tokens > capacity`). |

## Choosing the numbers

The refill rate is `refill_amount / refill_interval` in your time unit. A few
examples, using milliseconds:

| Policy | `new(capacity, refill_amount, refill_interval)` |
|---|---|
| 10 req/sec, burst 10 | `RateLimiter::new(10, 1, 100)` |
| 100 req/sec, burst 20 | `RateLimiter::new(20, 1, 10)` |
| 5 req/sec, burst 5 | `RateLimiter::new(5, 1, 200)` |
| 600 req/min, burst 60 | `RateLimiter::new(60, 10, 1_000)` |

`capacity` and the refill rate are independent: a large capacity with a slow
refill allows a big one-off burst but a low steady rate.

## Concurrency

`RateLimiter` is a plain value and is **not** internally synchronized (no atomics
— keeping it dependency-free and `no_std`). To share one limiter across threads
or tasks, wrap it in your own `Mutex`/lock. For per-key limiting, keep a separate
limiter per key.

## Pairs well with the rest of Reliakit

Use it alongside [`reliakit-circuit`](https://crates.io/crates/reliakit-circuit)
(stop calling a dependency that is down) and
[`reliakit-backoff`](https://crates.io/crates/reliakit-backoff) (space out
retries). All three are clock-agnostic and `no_std`.

## When to use it

- Capping calls to an API, a database, or any shared resource.
- Smoothing bursty work into a steady rate.
- Embedded or runtime-agnostic code that still needs throttling.

## When not to use it

- Distributed rate limiting across many processes — this is an in-process
  limiter; coordinate through a shared store for a global limit.
- Precise sub-token (fractional) rates — the bucket works in whole tokens; scale
  your unit (e.g. count in tenths) if you need finer granularity.

## Feature Flags

| Flag | Default | Effect |
|---|---:|---|
| `core` | no | Adds `*_now(clock)` methods that read the time from a `reliakit_core::Clock` instead of taking an explicit `now: u64`. Pulls in `reliakit-core` (`no_std`, zero third-party dependencies). |

The `now: u64` methods are the primitive API; the `core` feature is purely a
convenience.

## `no_std`

`reliakit-ratelimit` is `#![no_std]` and allocation-free — pure `core`, zero
third-party dependencies. The limiter is a small `Copy` value, so it runs on
embedded targets without changes.

## Safety

This crate is `#![forbid(unsafe_code)]` and `#![no_std]`. All arithmetic
saturates; no method panics on any input, including a non-monotonic clock.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Published to crates.io and pre-1.0. The token-bucket API is stable; it may
receive backward-compatible refinements before a `1.0` release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

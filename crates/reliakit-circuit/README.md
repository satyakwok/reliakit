<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-circuit

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-circuit)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-circuit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

A clock-agnostic **circuit breaker** for Rust.

When a dependency starts failing, retrying it immediately makes things worse: you
pile load onto a service that is already struggling and make your own callers
wait on calls that are almost certain to fail. A circuit breaker watches the
recent outcome history and, once failures cross a threshold, **"opens"** — it
rejects calls instantly (failing fast) for a cooldown period, then lets a single
trial call through to check whether the dependency has recovered before resuming
normal traffic.

`reliakit-circuit` implements that pattern as a small, `Copy` state machine with
**no dependencies**, no `std`, and no hidden behavior. It does not read the
clock, sleep, spawn tasks, or allocate — *you* pass the current time in and *you*
make the call. That makes it equally usable from synchronous code, any async
runtime, and `no_std` / embedded targets, and it makes every transition
deterministic and trivial to unit-test.

## How it works

A breaker moves between three states:

```text
           failures >= failure_threshold
  Closed ───────────────────────────────▶ Open
    ▲                                       │
    │ successes >= success_threshold        │ cooldown elapsed
    │                                       ▼
    └────────────── HalfOpen ◀──────────────┘
                       │
                       │ any failure
                       └──────────────▶ Open
```

| State | Calls | Behavior |
|---|---|---|
| **Closed** | allowed | Normal operation. Consecutive failures are counted; reaching `failure_threshold` trips the breaker to **Open**. A success resets the count. |
| **Open** | rejected | `allow()` returns `false` immediately. After `cooldown` time units the next `allow()` moves the breaker to **HalfOpen**. |
| **HalfOpen** | allowed (trial) | A probationary period. `success_threshold` consecutive successes close the breaker; the **first** failure reopens it (and restarts the cooldown). |

## Why "clock-agnostic"?

Most circuit breakers are tied to `std::time::Instant` or to a specific async
runtime's timer. This one takes the current time as a `u64` argument in whatever
monotonic unit you choose (milliseconds is typical) and `cooldown` in that same
unit. The benefits:

- **Runtime-neutral.** Works under Tokio, async-std, blocking threads, or a bare
  metal loop — you decide where time comes from.
- **`no_std`-friendly.** No clock dependency means it compiles for embedded
  targets (CI builds it for `thumbv7em-none-eabi`).
- **Deterministic & testable.** Transitions depend only on the timestamps you
  pass, so tests assert exact behavior with no sleeping or mocking.

A clock that briefly moves backwards is handled with saturating arithmetic — the
breaker simply stays open rather than panicking.

## Installation

```toml
[dependencies]
reliakit-circuit = "0.1"
```

This crate is `#![no_std]` and has no feature flags; it depends only on `core`.

## Usage

Wrap each call to a dependency in `allow()` / `on_success()` / `on_failure()`:

```rust
use reliakit_circuit::CircuitBreaker;

// Trip after 5 consecutive failures; stay open for 10 seconds.
let mut breaker = CircuitBreaker::new(5, 10_000);

let now = now_millis(); // your own monotonic clock, in milliseconds
if breaker.allow(now) {
    match call_remote() {
        Ok(_)  => breaker.on_success(),
        Err(_) => breaker.on_failure(now),
    }
} else {
    // Fail fast: skip the call, serve a cached value or return an error.
}
```

Require several good calls before fully trusting the dependency again:

```rust
use reliakit_circuit::CircuitBreaker;

// Need 3 consecutive successes during the trial period to close.
let breaker = CircuitBreaker::new(5, 10_000).with_success_threshold(3);
```

See [`examples/basic.rs`](./examples/basic.rs) for a complete request loop that
trips, rejects fast, recovers, and closes again.

## API

| Method | Purpose |
|---|---|
| `CircuitBreaker::new(failure_threshold, cooldown)` | Construct a breaker (const fn). |
| `.with_success_threshold(n)` | Successes needed in HalfOpen to close (default `1`). |
| `allow(now) -> bool` | Whether a call may proceed; advances Open → HalfOpen when the cooldown elapses. |
| `on_success()` | Record a successful call. |
| `on_failure(now)` | Record a failed call. |
| `state() -> State` | Current state (`Closed` / `Open` / `HalfOpen`). |
| `trip(now)` / `reset()` | Force the breaker open or closed (e.g. from health signals). |

## Pairs well with `reliakit-backoff`

A circuit breaker decides **whether** to attempt a call; [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff)
decides **how long to wait** before the next attempt. Used together, the breaker
sheds load while a dependency is down and backoff spaces out the retries — both
clock-agnostic, both `no_std`.

## Concurrency

`CircuitBreaker` is a plain value and is **not** internally synchronized (no
atomics — keeping it dependency-free and `no_std`). To share one across threads
or tasks, wrap it in your own `Mutex`/lock. For per-task breakers, just give each
its own copy.

## When to use it

- Calls to a remote service, database, or any dependency that can fail or stall.
- Guarding a shared resource so one failing downstream does not cascade.
- Embedded or runtime-agnostic code that still needs fail-fast behavior.

## When not to use it

- For a single retry with a delay, a backoff policy alone is enough — reach for
  [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff).
- It does not measure latency, error rates over sliding windows, or perform
  health checks on its own; it reacts to the success/failure outcomes you report.

## Safety

This crate is `#![forbid(unsafe_code)]` and `#![no_std]`. All arithmetic
saturates; no method panics on any input, including a non-monotonic clock.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

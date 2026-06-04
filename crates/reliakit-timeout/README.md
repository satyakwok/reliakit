<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-timeout

[![Crates.io](https://img.shields.io/crates/v/reliakit-timeout.svg)](https://crates.io/crates/reliakit-timeout)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-timeout.svg)](https://crates.io/crates/reliakit-timeout)
[![Docs.rs](https://docs.rs/reliakit-timeout/badge.svg)](https://docs.rs/reliakit-timeout)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-timeout)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-timeout)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

Clock-agnostic deadlines and timeouts for Rust.

`reliakit-timeout` answers one question: *has my time budget run out, and how
much is left?* It does not read the clock, sleep, or spawn anything — you
capture a start instant and a budget, then pass `now` to the query methods. That
keeps it usable from synchronous code, any async runtime, and `no_std` /
embedded targets, with deterministic tests.

Time is a plain `u64` in any monotonic unit you choose (milliseconds is
typical), matching `reliakit-circuit` and `reliakit-ratelimit`. All arithmetic
saturates, so no method panics — not on overflow, and not on a clock that moves
backwards.

The crate has no dependencies, is `#![no_std]`, and forbids unsafe code.

## What This Crate Does

- `Timeout` — a reusable budget that is not yet pinned to a timeline. Configure
  it once, then call `Timeout::start(now)` per operation to get a `Deadline`.
- `Deadline` — a budget pinned to a start instant; it expires at `start +
  budget`. Query it with:
  - `remaining(now)` / `elapsed(now)` — saturating time left / time used.
  - `is_expired(now)` — whether `now >= expiry`.
  - `check(now)` — `Some(remaining)` while live, `None` once expired.
  - `allows(now, duration)` — whether an operation of that length still fits.
  - `clamp(now, duration)` — `duration` capped to the time left in the budget.

## What This Crate Does Not Do

It does not sleep, cancel futures, or enforce the timeout for you. It tracks a
budget against a clock you own; you decide what to do when it expires. Pair it
with your runtime's timer or `select!` to actually abort work.

## Installation

```toml
[dependencies]
reliakit-timeout = "0.1"
```

This crate is `no_std` and has no feature flags; it depends only on `core`.

## Example

```rust
use reliakit_timeout::{Deadline, Timeout};

// A 30s budget (here in milliseconds), pinned to the start of the operation.
let policy = Timeout::new(30_000);
let deadline = policy.start(1_000); // started at t = 1_000

assert_eq!(deadline.remaining(21_000), 10_000);
assert_eq!(deadline.check(21_000), Some(10_000));
assert_eq!(deadline.check(40_000), None); // expired
```

Bound a retry delay by the time left in the budget:

```rust
use reliakit_timeout::Deadline;

let deadline = Deadline::new(0, 1_000);
let proposed_backoff = 800;

let now = 500;
if !deadline.is_expired(now) {
    let wait = deadline.clamp(now, proposed_backoff); // min(800, 500 left) = 500
    assert_eq!(wait, 500);
    // sleep(wait); try_again();
}
```

## Behavior

| Method | Result |
|---|---|
| `expiry()` | `start + budget` (saturating) |
| `remaining(now)` | `expiry - now`, saturating to `0` once expired |
| `elapsed(now)` | `now - start`, saturating to `0` before `start` |
| `is_expired(now)` | `now >= expiry` (a zero budget expires immediately) |
| `check(now)` | `Some(remaining)` while live, else `None` |
| `clamp(now, d)` | `min(d, remaining(now))` |

## Safety

This crate is `#![forbid(unsafe_code)]` and `#![no_std]`. Every method is a
`const fn` over saturating integer arithmetic, so there is no input — including a
backwards clock or an overflowing `start + budget` — that panics.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

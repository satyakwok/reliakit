<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-backoff

[![Crates.io](https://img.shields.io/crates/v/reliakit-backoff.svg)](https://crates.io/crates/reliakit-backoff)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-backoff.svg)](https://crates.io/crates/reliakit-backoff)
[![Docs.rs](https://docs.rs/reliakit-backoff/badge.svg)](https://docs.rs/reliakit-backoff)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-backoff)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-backoff)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

Clock-agnostic retry backoff policies for Rust.

`reliakit-backoff` computes *how long to wait* between retries. It does not
sleep, spawn tasks, or read the clock; you decide when to call it and how to
wait. That keeps it usable from synchronous code, any async runtime, and
`no_std` / embedded targets, with deterministic tests.

The crate has no dependencies, is `#![no_std]`, and forbids unsafe code.

## What This Crate Does

- `Backoff`: a small `Copy` policy: a base delay, a growth strategy (constant,
  linear, exponential, or Fibonacci), an optional maximum delay, and an optional retry
  limit.
- `Backoff::delay(attempt)`: maps a zero-based attempt number to the delay to
  wait, or `None` once the retry limit is reached. All arithmetic saturates, so
  large attempt numbers never overflow, panic, or hang.
- `Backoff::delays()`: an iterator over successive delays.
- `full_jitter` / `equal_jitter` / `decorrelated_jitter`: pure jitter helpers
  that take a caller-supplied random value, so the crate stays dependency-free
  and the math stays testable.

## What This Crate Does Not Do

It does not sleep, retry for you, manage tasks, or own a random number
generator. It computes delays; you drive the loop and supply randomness. For an
opinionated executor, combine it with your runtime's timer.

## Installation

```toml
[dependencies]
reliakit-backoff = "1.1"
```

This crate is `no_std` and has no feature flags; it depends only on `core`.

## Example

```rust
use core::time::Duration;
use reliakit_backoff::{full_jitter, Backoff};

let policy = Backoff::exponential(Duration::from_millis(100), 2)
    .with_max_delay(Duration::from_secs(2))
    .with_max_retries(5);

assert_eq!(policy.delay(0), Some(Duration::from_millis(100)));
assert_eq!(policy.delay(1), Some(Duration::from_millis(200)));
assert_eq!(policy.delay(5), None); // retry limit reached

// Drive your own loop; supply randomness for jitter from your RNG.
let rand = 0x1234_5678u32;
for base in policy.delays() {
    let wait = full_jitter(base, rand);
    // sleep(wait); if try_operation().is_ok() { break; }
    let _ = wait;
}
```

## Strategies

| Constructor | Delay for attempt `n` |
|---|---|
| `Backoff::constant(base)` | `base` |
| `Backoff::linear(base, step)` | `base + step * n` |
| `Backoff::exponential(base, factor)` | `base * factor^n` |
| `Backoff::fibonacci(base)` | `base * fib(n)` (`1, 1, 2, 3, 5, 8, ...`) |

All are clamped to `with_max_delay(..)` and stop at `with_max_retries(..)`.
`factor` is an integer multiplier (e.g. `2` doubles each attempt).

## Jitter

| Function | Range |
|---|---|
| `full_jitter(delay, rand)` | `0 ..= delay` |
| `equal_jitter(delay, rand)` | `delay/2 ..= delay` |
| `decorrelated_jitter(base, prev, cap, rand)` | `base ..= prev*3`, capped at `cap` |

`rand` is interpreted as the fraction `rand / u32::MAX`. Source it from `rand`,
`getrandom`, or a hardware RNG.

## Feature Flags

This crate has no feature flags.

## `no_std`

`reliakit-backoff` is `#![no_std]` and allocation-free: it depends only on
`core`, with no third-party crates. All delay arithmetic saturates, so large
attempt numbers never overflow, panic, or hang.

## Safety

This crate is `#![forbid(unsafe_code)]` and `#![no_std]`.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Published to crates.io at `1.1`. The API is small and stable; further changes
will be additive under semver.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

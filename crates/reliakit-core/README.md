<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-core

[![Crates.io](https://img.shields.io/crates/v/reliakit-core.svg)](https://crates.io/crates/reliakit-core)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-core.svg)](https://crates.io/crates/reliakit-core)
[![Docs.rs](https://docs.rs/reliakit-core/badge.svg)](https://docs.rs/reliakit-core)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-core)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-core)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

Shared building blocks for the Reliakit workspace.

Reliakit's resilience crates (`reliakit-backoff`, `reliakit-circuit`,
`reliakit-ratelimit`, `reliakit-timeout`) are *clock-agnostic*: you pass the
current time in as a `u64` tick in any monotonic unit. `reliakit-core` provides
the small piece they share — a `Clock` trait plus ready-made clocks — so you do
not have to hand-roll one.

The crate has no dependencies and forbids unsafe code.

## What This Crate Does

- `Clock` — a trait with a single `now(&self) -> u64`. There is a blanket impl
  for `&C`, and it is object-safe (`&dyn Clock`).
- `ManualClock` — a settable clock for deterministic tests (`new`, `set`,
  `advance`); uses interior mutability so every method takes `&self`. `no_std`.
- `MonotonicClock` — milliseconds since creation, backed by
  `std::time::Instant`, so it never goes backwards and ignores wall-clock
  adjustments. Requires the `std` feature.

## Installation

```toml
[dependencies]
reliakit-core = "1.0"
```

Disable default features for `no_std`; `ManualClock` and the `Clock` trait work
without `std`, while `MonotonicClock` needs the default `std` feature.

## Example

```rust
use reliakit_core::{Clock, ManualClock};

let clock = ManualClock::new(0);
clock.advance(250);
assert_eq!(clock.now(), 250);

// Feed `clock.now()` into any clock-agnostic Reliakit policy, e.g.
// `breaker.allow(clock.now())` or `limiter.try_acquire_one(clock.now())`.
```

## Feature flags

| Feature | Default | Effect |
|---|---|---|
| `std` | yes | Enables `MonotonicClock` (real monotonic time). |

## Safety

This crate is `#![forbid(unsafe_code)]` and `no_std` (with the default `std`
feature adding `MonotonicClock`). Clock arithmetic saturates.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Published to crates.io and pre-1.0. The crate is intentionally minimal — a
`Clock` trait plus two clocks — and its API may receive backward-compatible
refinements before a `1.0` release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

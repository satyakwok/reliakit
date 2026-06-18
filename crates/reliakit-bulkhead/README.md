<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-bulkhead

[![Crates.io](https://img.shields.io/crates/v/reliakit-bulkhead.svg)](https://crates.io/crates/reliakit-bulkhead)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-bulkhead.svg)](https://crates.io/crates/reliakit-bulkhead)
[![Docs.rs](https://docs.rs/reliakit-bulkhead/badge.svg)](https://docs.rs/reliakit-bulkhead)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-bulkhead)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-bulkhead)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

A concurrency limiter for Rust.

`reliakit-bulkhead` caps how many operations may be *in flight* at once. It is a
counting semaphore: acquire a permit before starting work, release it when the
work finishes. When no permit is available the request is rejected immediately,
so load is shed instead of piling up on a struggling dependency.

It does not block, sleep, spawn tasks, or read the clock; acquiring a permit
either succeeds now or fails now. That keeps it usable from synchronous code, any
async runtime, and `no_std` / embedded targets, with deterministic tests.

The crate has no dependencies, is `#![no_std]`, and forbids unsafe code.

## What This Crate Does

- `Bulkhead`: a small `Copy` value: a fixed `capacity` and the number of
  permits currently held.
- `try_acquire(n)` / `try_acquire_one()`: reserve permits when room exists and
  report whether it succeeded; an over-capacity request always fails and no
  partial acquire ever happens.
- `release(n)` / `release_one()`: return permits; saturates at zero, so a stray
  release cannot underflow or panic.
- `available()` / `in_flight()` / `is_full()` / `is_empty()` / `reset()`:
  inspect and clear the limiter.

## What This Crate Does Not Do

It does not block to wait for a permit, queue requests, run work, or read time.
There is no RAII guard: you pair acquire and release yourself (the type stays
`Copy` and `no_std` as a result). Where `reliakit-ratelimit` caps the *rate* of
operations over time, a bulkhead caps the *number running at once*; the two
compose.

## Installation

```toml
[dependencies]
reliakit-bulkhead = "1.0"
```

This crate is `no_std` and has no feature flags; it depends only on `core`.

## Example

```rust
use reliakit_bulkhead::Bulkhead;

// Allow at most two concurrent operations.
let mut bulkhead = Bulkhead::new(2);

assert!(bulkhead.try_acquire_one()); // 1 in flight
assert!(bulkhead.try_acquire_one()); // 2 in flight
assert!(!bulkhead.try_acquire_one()); // full: rejected, shed load

bulkhead.release_one(); // one operation finished
assert!(bulkhead.try_acquire_one()); // room again
```

## Releasing permits

Every successful acquire must be matched by a release, including on the error
path, or the bulkhead slowly fills and rejects everything. The crate keeps the
model explicit (no hidden guard) so it stays `Copy` and `no_std`; pair
acquire/release yourself, e.g. with a manual `Drop` wrapper in your own code.

## Feature Flags

This crate has no feature flags.

## `no_std`

`reliakit-bulkhead` is `#![no_std]` and allocation-free: it depends only on
`core`, with no third-party crates. Permit arithmetic saturates and the capacity
is clamped to at least `1`, so no input overflows, panics, or underflows.

## Safety

This crate is `#![forbid(unsafe_code)]` and `#![no_std]`.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Pre-1.0. The API is small and stable; it may receive backward-compatible
refinements before a `1.0` release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

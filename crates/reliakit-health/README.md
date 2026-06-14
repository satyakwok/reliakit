<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-health

[![Crates.io](https://img.shields.io/crates/v/reliakit-health.svg)](https://crates.io/crates/reliakit-health)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-health.svg)](https://crates.io/crates/reliakit-health)
[![Docs.rs](https://docs.rs/reliakit-health/badge.svg)](https://docs.rs/reliakit-health)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-health)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-health)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

Health status types and a criticality-aware aggregator for Rust.

`reliakit-health` answers one question: *given the state of my components, what
is the overall health of the service?* It is plain data — it does not run checks,
read the clock, or perform I/O. You report each component's status; it rolls them
up into one status, applying per-component **criticality** so a non-critical
dependency going down *degrades* the service instead of *downing* it.

Typical homes: a `/health` or `/readyz` endpoint, a Kubernetes readiness/liveness
probe, or a status page.

The crate has no dependencies, is `no_std`-friendly, and forbids unsafe code.

## What This Crate Does

- `Health` — `Healthy` / `Degraded` / `Unhealthy`, ordered by severity, with
  `worst`/`best`/`capped_at` combinators, `is_operational`, `from_ok`, `Display`.
- `Criticality` — `Critical` (default) or `Optional`; an `Optional` component's
  failure is capped at `Degraded`.
- `Check<'a>` + `aggregate(..)` — an allocation-free, `no_std` path: build a fixed
  array of borrowing checks and roll them up.
- `HealthReport` — an owned, dynamically built report with a
  `critical`/`optional`/`with` builder, `overall()`, `summary()` counts,
  `by_status()` filtering, and `reasons()` for non-healthy components.

## What This Crate Does Not Do

It does not perform health checks, ping anything, read time, or change behavior —
it only summarizes statuses you supply. To *act* on a failing dependency (stop
calling it) use a circuit breaker; `reliakit-health` reports, it does not decide.

## Installation

```toml
[dependencies]
reliakit-health = "0.1"
```

For `no_std` without allocation (core types + `aggregate` only):

```toml
[dependencies]
reliakit-health = { version = "0.1", default-features = false }
```

For `no_std` with the owned `HealthReport`:

```toml
[dependencies]
reliakit-health = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Example

```rust
use reliakit_health::{Health, HealthReport};

let report = HealthReport::new()
    .critical("database", Health::Healthy)
    .optional("cache", Health::Unhealthy) // non-critical: only degrades
    .critical("queue", Health::Degraded)
    .detail("redelivery backlog");

let overall = report.overall();
assert_eq!(overall, Health::Degraded);

// Map to an HTTP status for a /health endpoint.
let http = if overall.is_operational() { 200 } else { 503 };
assert_eq!(http, 200);

let s = report.summary();
assert_eq!((s.healthy, s.degraded, s.unhealthy), (1, 1, 1));
```

Allocation-free roll-up over a fixed array:

```rust
use reliakit_health::{aggregate, Check, Health};

let checks = [
    Check::new("database", Health::Healthy),
    Check::new("cache", Health::Unhealthy).optional().with_detail("primary down"),
];
assert_eq!(aggregate(checks), Health::Degraded);
```

## Roll-up rules

The overall status is the worst (most severe) effective status, where
`Healthy < Degraded < Unhealthy`. A `Critical` component contributes its status
unchanged; an `Optional` component's status is capped at `Degraded`. An empty set
is `Healthy`.

## Composing with the other resilience crates

The other `reliakit-*` resilience crates produce signals this one summarizes: a
tripped `reliakit-circuit` breaker maps to an `Unhealthy` (or `Degraded`)
component, a full `reliakit-bulkhead` or `reliakit-ratelimit` to `Degraded`, an
expired `reliakit-timeout` deadline to `Degraded`.

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables the standard library; implies `alloc` |
| `alloc` | no | Enables `HealthReport`, `Component`, `Summary` |

`Health`, `Criticality`, `Check`, and `aggregate` need neither feature.

## `no_std`

`reliakit-health` is `no_std`-friendly and depends only on `core` (plus `alloc`
for `HealthReport`). With `--no-default-features` you get the allocation-free core.

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Pre-1.0. The API is small and stable; it may receive backward-compatible
refinements before a `1.0` release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

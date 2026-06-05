<p align="center">
  <img src="./assets/reliakit-logo.png" alt="Reliakit" width="520">
</p>

# Reliakit

A toolkit of small, focused reliability building blocks for Rust — `no_std` and zero-dependency.

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/satyakwok/reliakit?style=flat)](https://github.com/satyakwok/reliakit/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/satyakwok/reliakit)](https://github.com/satyakwok/reliakit/issues)
[![Last commit](https://img.shields.io/github/last-commit/satyakwok/reliakit)](https://github.com/satyakwok/reliakit/commits/main)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/satyakwok/reliakit)

Reliakit is a Rust workspace of small, focused crates for building reliable
software: validated types that make invalid states unrepresentable, secret
redaction, bounded collections, canonical binary encoding, and runtime-agnostic
resilience patterns — retry backoff, a circuit breaker, and a rate limiter.

Each crate is small, dependency-free at runtime, `#![forbid(unsafe_code)]`,
`no_std`-friendly, and usable independently — take only what you need.

## Crates at a glance

| Crate | What it does |
|---|---|
| [`reliakit-primitives`](https://crates.io/crates/reliakit-primitives) | Type-safe constrained values: `Port`, `Email`, `BoundedStr`, `SemVer`, `Uuid`, … |
| [`reliakit-secret`](https://crates.io/crates/reliakit-secret) | Secret wrappers that redact in `Debug`/`Display` and logs |
| [`reliakit-validate`](https://crates.io/crates/reliakit-validate) | A `Validate` trait and error type that collects all field violations |
| [`reliakit-collections`](https://crates.io/crates/reliakit-collections) | Bounded collections (`BoundedVec`) whose size invariants can't be violated |
| [`reliakit-codec`](https://crates.io/crates/reliakit-codec) | Deterministic canonical binary encoding and strict decoding |
| [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff) | Retry backoff policies (constant/linear/exponential) with jitter |
| [`reliakit-circuit`](https://crates.io/crates/reliakit-circuit) | Circuit breaker that fails fast while a dependency is down |
| [`reliakit-ratelimit`](https://crates.io/crates/reliakit-ratelimit) | Token-bucket rate limiter with `retry_after` |
| [`reliakit-json`](https://crates.io/crates/reliakit-json) | Strict, bounded, deterministic JSON for untrusted input |
| [`reliakit-timeout`](https://crates.io/crates/reliakit-timeout) | Clock-agnostic deadlines and timeouts (time-limiter) |
| [`reliakit-core`](https://crates.io/crates/reliakit-core) | Shared `Clock` trait and ready-made clocks for the resilience crates |

`reliakit-backoff`, `reliakit-circuit`, `reliakit-ratelimit`, and
`reliakit-timeout` are **clock-agnostic resilience patterns** — you pass the
time in, so they work in
sync, async, and embedded code, and compose: a rate limiter decides whether to
call, a circuit breaker stops calling a failing dependency, and backoff spaces
out retries.

## Why Reliakit?

- **Validate once, at the boundary.** Construct a typed value where data enters
  your program (config, request, CLI, environment) and never re-check it again.
- **Carry invariants in types.** A `Port` is always `1..=65535`; a
  `BoundedStr<3, 32>` always has 3–32 characters. Function signatures document
  and enforce these rules for you.
- **Fewer unchecked strings and numbers.** Replace raw `String`/`u16`/`u8` with
  types that cannot hold invalid states.
- **No framework lock-in.** No runtime, no macros required, no global state.
  Reliakit is plain types and traits you can adopt one at a time.
- **Small independent crates.** Take only what you need. Each crate compiles
  fast and has zero runtime dependencies.

## When should I use this?

Reliakit is most useful at the edges of your program, where untrusted or
loosely-typed data becomes domain data:

- **Config parsing** — turn a TOML/JSON/env value into a `Port`, `Percent`, or
  `BoundedStr` once, and fail early with a clear error.
- **CLI input** — validate flags and arguments into typed values before they
  reach your logic.
- **API payload validation** — collect every field error at once with
  `reliakit-validate` and return a precise message per field.
- **Service settings** — model "must have between 1 and 10 endpoints" with
  `BoundedVec`, or "this must be non-empty" with `NonEmptyStr`.
- **Safe diagnostic / log output** — wrap secrets in `SecretString` so they
  show as `[REDACTED]` in `Debug`, `Display`, logs, and error reports.

## Before / After

Before — every field can hold an invalid state, and the API key can leak into
logs through `Debug`:

```rust
struct ServiceConfig {
    name: String,      // could be empty or 500 chars
    port: u16,         // could be 0
    error_budget: u8,  // could be 250
    api_key: String,   // leaks in {:?} / logs
}
```

After — invalid states are unrepresentable, and the secret never appears in
output:

```rust
use reliakit_primitives::{BoundedStr, Percent, Port};
use reliakit_secret::SecretString;

struct ServiceConfig {
    name: BoundedStr<3, 32>, // always 3–32 characters
    port: Port,              // always 1..=65535
    error_budget: Percent,   // always 0..=100
    api_key: SecretString,   // redacted in Debug/Display
}
```

See [`examples/service_config.rs`](./crates/reliakit-primitives/examples/service_config.rs)
for a complete, runnable version that combines primitives, secret redaction, and
struct-level validation.

## Crates

### `reliakit-primitives` — [crates.io](https://crates.io/crates/reliakit-primitives) · [docs.rs](https://docs.rs/reliakit-primitives)

Type-safe primitives for constrained values such as non-empty strings, bounded
strings, slugs, email addresses, HTTP URLs, percentages, ports, byte sizes,
semantic versions, UUIDs, and human-readable durations.

Implemented types:

- `NonEmptyStr`
- `BoundedStr`
- `Slug`
- `Email`
- `HttpUrl`
- `HexString`
- `Percent`
- `PercentageF64`
- `Port`
- `PositiveInt`
- `PositiveFloat`
- `ByteSize`
- `NonEmptyVec<T>`
- `SemVer`
- `Uuid`
- `HumanDuration`

### `reliakit-secret` — [crates.io](https://crates.io/crates/reliakit-secret) · [docs.rs](https://docs.rs/reliakit-secret)

Secret-safe wrappers for values that should not leak through `Debug`, `Display`,
logs, reports, or diagnostic output.

Implemented types:

- `Secret<T>`
- `SecretString`
- `ExposeSecret<T>`
- `ExposeSecretMut<T>`

### `reliakit-validate` — [crates.io](https://crates.io/crates/reliakit-validate) · [docs.rs](https://docs.rs/reliakit-validate)

Composable validation traits and error types for Rust structs and values.

Implemented types:

- `Validate` trait
- `Valid<T>`
- `ValidationError`
- `Violation`

### `reliakit-collections` — [crates.io](https://crates.io/crates/reliakit-collections) · [docs.rs](https://docs.rs/reliakit-collections)

Bounded and reliability-oriented collection types.

Implemented types:

- `BoundedVec<T, MIN, MAX>`

### `reliakit-codec` — [crates.io](https://crates.io/crates/reliakit-codec) · [docs.rs](https://docs.rs/reliakit-codec)

Deterministic canonical binary encoding and decoding. It defines one canonical
binary representation per supported type and keeps encoding explicit through
handwritten trait implementations. It supports `no_std` with `alloc` and offers
optional `reliakit-primitives` integration.

Implemented types:

- `CanonicalEncode`
- `CanonicalDecode`
- `EncodeSink`
- `DecodeSource`
- `SliceReader`
- `CodecError`

### `reliakit-backoff` — [crates.io](https://crates.io/crates/reliakit-backoff) · [docs.rs](https://docs.rs/reliakit-backoff)

Clock-agnostic retry backoff policies. It computes the delay to wait before each
retry (constant, linear, or exponential, with optional cap and retry limit) and
provides pure jitter helpers. It does not sleep or read the clock, so it works
in sync, async, and `no_std` contexts. Depends only on `core`.

Implemented types:

- `Backoff`
- `Delays`
- `full_jitter` / `equal_jitter`

### `reliakit-circuit` — [crates.io](https://crates.io/crates/reliakit-circuit) · [docs.rs](https://docs.rs/reliakit-circuit)

Clock-agnostic circuit breaker. A small `Copy` state machine
(`Closed`/`Open`/`HalfOpen`) that fails fast while a dependency is down and lets
trial calls through to test recovery. It does not read the clock, sleep, or
allocate — you pass the time in. Depends only on `core`. Pairs with
`reliakit-backoff`.

Implemented types:

- `CircuitBreaker`
- `State`

### `reliakit-ratelimit` — [crates.io](https://crates.io/crates/reliakit-ratelimit) · [docs.rs](https://docs.rs/reliakit-ratelimit)

Clock-agnostic token-bucket rate limiter. Caps how often something may happen,
with a configurable burst capacity and refill rate, and reports how long to wait
when a request is denied. It does not read the clock or allocate — you pass the
time in. Depends only on `core`. Pairs with `reliakit-circuit` and
`reliakit-backoff`.

Implemented types:

- `RateLimiter`

### `reliakit-json` — [crates.io](https://crates.io/crates/reliakit-json) · [docs.rs](https://docs.rs/reliakit-json)

Strict, bounded, and deterministic JSON for untrusted input and predictable
output. Rejects duplicate keys, enforces explicit resource limits, preserves
number precision, reports errors with location and path, and serializes
deterministically. Depends only on `core` + `alloc`. Deliberately narrow in
scope: no derive macros, schema validation, JSON5, comments, or lenient parsing.

Implemented types:

- `JsonValue`, `JsonNumber`, `JsonObject`
- `JsonLimits`
- `JsonError` / `JsonErrorKind`

An opt-in `canonical` feature adds RFC 8785 (JCS) canonical output
(`to_canonical_string` / `to_canonical_vec`) for hashing and signing.

### `reliakit-timeout` — [crates.io](https://crates.io/crates/reliakit-timeout) · [docs.rs](https://docs.rs/reliakit-timeout)

Clock-agnostic deadlines and timeouts — the time-limiter that rounds out the
resilience set. You capture a start instant and a budget (`u64`, any monotonic
unit) and ask whether the budget has run out and how much is left; it never
reads the clock, sleeps, or cancels work. Pure `core`, zero dependencies,
saturating arithmetic, no panics.

Implemented types:

- `Timeout` — a reusable budget; `start(now)` pins it to a `Deadline`.
- `Deadline` — `remaining`, `elapsed`, `is_expired`, `check`, `allows`, `clamp`.

### `reliakit-core` — [crates.io](https://crates.io/crates/reliakit-core) · [docs.rs](https://docs.rs/reliakit-core)

Shared building blocks for the workspace. Provides a `Clock` trait (one method,
`now(&self) -> u64`) plus ready-made clocks so you do not have to hand-roll the
`u64` tick the resilience crates expect: `ManualClock` (settable, `no_std`, for
deterministic tests) and `MonotonicClock` (milliseconds since creation, backed
by `std::time::Instant`). Zero dependencies.

### `reliakit-derive`

Planned. Derive macros for validation and constrained types.

## Installation

From crates.io:

```toml
[dependencies]
reliakit-primitives = "0.4"
reliakit-secret = "0.1"
reliakit-validate = "0.3"
reliakit-collections = "0.3"
reliakit-codec = "0.2"
reliakit-backoff = "0.1"
reliakit-circuit = "0.2"
reliakit-ratelimit = "0.1"
reliakit-json = "0.2"
reliakit-timeout = "0.1"
reliakit-core = "0.1"
```

Add only the crates you need — each is usable independently.

Or depend on the Git repository directly:

```toml
[dependencies]
reliakit-primitives = { git = "https://github.com/satyakwok/reliakit", package = "reliakit-primitives" }
```

## Workspace vs Crates

This repository is a Cargo workspace. The workspace lets Reliakit develop
multiple related crates in one repository with shared CI, tests, examples, and
metadata.

Users do not depend on the workspace directly. Add only the crate you need:

- Use `reliakit-primitives` for constrained primitive types.
- Use `reliakit-secret` for redacted secret wrappers.
- Use `reliakit-validate` for validation traits and errors.
- Use `reliakit-collections` for bounded collection types.
- Use `reliakit-codec` for deterministic canonical binary encoding.

## MSRV

Reliakit currently supports Rust `1.85` and newer.

## Example

```rust
use reliakit_primitives::{BoundedStr, Percent, Port};
use reliakit_secret::{ExposeSecret, SecretString};

type ServiceName = BoundedStr<3, 32>;

let name = ServiceName::new("api-service")?;
let success_rate = Percent::new(99)?;
let port = Port::new(8080)?;
let api_key = SecretString::from_string("rk_live_example");

assert_eq!(api_key.to_string(), "[REDACTED]");
assert_eq!(api_key.expose_secret(), "rk_live_example");
```

## Design Goals

- Reusable library primitives.
- Clear type semantics.
- Minimal dependencies.
- No hidden runtime.
- No framework lock-in.
- Optional feature flags.
- `no_std` support where practical.
- Safe diagnostic output.
- Stable, documented APIs.
- Composable crates.

## Scope

Reliakit focuses on small, explicit building blocks for reliability-oriented
Rust code.

It is intentionally not a framework. It does not provide a runtime, web stack,
ORM, logging system, or broad application platform.

Each crate is designed to be adopted independently, with minimal API surface,
clear invariants, and no hidden runtime behavior.

## Workspace Layout

```text
reliakit/
├── crates/
│   ├── reliakit-primitives/
│   │   └── examples/
│   ├── reliakit-secret/
│   │   └── examples/
│   ├── reliakit-validate/
│   ├── reliakit-collections/
│   ├── reliakit-codec/
│   │   └── examples/
│   ├── reliakit-backoff/
│   │   └── examples/
│   ├── reliakit-circuit/
│   │   └── examples/
│   ├── reliakit-ratelimit/
│   │   └── examples/
│   ├── reliakit-json/
│   │   └── examples/
│   ├── reliakit-timeout/
│   │   └── examples/
│   ├── reliakit-core/
│   │   └── examples/
│   └── reliakit-derive/
├── Cargo.toml
├── README.md
└── LICENSE
```

## Status

Active. Reliakit is published as a real Rust library workspace and follows
normal Rust crate versioning.

`reliakit-primitives`, `reliakit-secret`, `reliakit-validate`,
`reliakit-collections`, `reliakit-codec`, `reliakit-backoff`,
`reliakit-circuit`, `reliakit-ratelimit`, `reliakit-json`,
`reliakit-timeout`, and `reliakit-core` are published to crates.io. APIs may
receive compatible refinements before a `1.0` release.

Logo assets are stored under [`assets/`](./assets/).

## Roadmap

Published:

- `reliakit-primitives`
- `reliakit-secret`
- `reliakit-validate`
- `reliakit-collections`
- `reliakit-codec`
- `reliakit-backoff`
- `reliakit-circuit`
- `reliakit-ratelimit`
- `reliakit-json`
- `reliakit-timeout`
- `reliakit-core`

Planned:

- `reliakit-derive`

## Contributing

Contributions are welcome. Please open an issue before submitting a pull request
for non-trivial changes so the direction can be discussed first.

- Keep each crate minimal and focused.
- Add tests for any new public API surface.
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting.

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for contribution guidelines,
[`CHANGELOG.md`](./CHANGELOG.md) for release notes, [`RELEASING.md`](./RELEASING.md)
for the release process, and [`SECURITY.md`](./SECURITY.md) for vulnerability
reporting.

Every crate depends only on the standard library and on other `reliakit-*`
crates; a CI check fails the build if any third-party dependency is introduced.

## Star History

<a href="https://github.com/satyakwok/reliakit/stargazers">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=satyakwok/reliakit&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=satyakwok/reliakit&type=date&legend=top-left" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=satyakwok/reliakit&type=date&legend=top-left" />
 </picture>
</a>

## License

Licensed under the MIT License. See [`LICENSE`](./LICENSE).

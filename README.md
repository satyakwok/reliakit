<p align="center">
  <img src="./assets/reliakit-logo.png" alt="Reliakit" width="520">
</p>

# Reliakit

Small Rust reliability crates for explicit invariants, redacted secrets, bounded
inputs, deterministic data, and runtime-agnostic resilience.

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/satyakwok/reliakit?style=flat)](https://github.com/satyakwok/reliakit/stargazers)
[![Last commit](https://img.shields.io/github/last-commit/satyakwok/reliakit)](https://github.com/satyakwok/reliakit/commits/main)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/satyakwok/reliakit)

Reliakit is a workspace of small, focused crates for building reliable Rust
software — CLIs, services, bots, libraries, and infrastructure tools. The core
idea is simple: **validate and constrain data at the boundary, then carry the
trusted invariant deeper into your program** so the rest of the code cannot hold
an invalid state.

It is a general-purpose reliability toolkit. Validated primitives, secret
redaction, bounded collections, deterministic encoding, and runtime-agnostic
resilience utilities (retry backoff, circuit breaker, rate limiter, timeouts)
are useful in web backends, command-line tools, embedded code, data pipelines,
and protocol or blockchain work alike — none of those is the primary target.

Every crate is small, dependency-free at runtime (only the standard library and
other `reliakit-*` crates — a CI check fails the build if any third-party
dependency appears), `#![forbid(unsafe_code)]`, and usable on its own. You adopt
**one crate at a time**, not a framework.

## Why Reliakit?

- **Validate once, at the boundary.** Construct a typed value where data enters
  your program (config, request, CLI, environment) and never re-check it again.
- **Make invalid states hard to represent.** A `Port` is always `1..=65535`; a
  `BoundedStr<3, 32>` always has 3–32 characters. The type signature documents
  and enforces the rule for you.
- **Stop leaking secrets.** Wrap sensitive values in `Secret<T>` / `SecretString`
  so they render as `[REDACTED]` in `Debug`, `Display`, logs, and error reports.
- **Bound your inputs and collections.** `BoundedVec<T, MIN, MAX>` cannot be
  built outside its size limits.
- **Encode data deterministically.** `reliakit-codec` (binary) and
  `reliakit-json` (text) produce the same bytes for the same value — handy for
  cache keys, fixtures, hashing, and signing.
- **Handle resilience explicitly.** Backoff, circuit breaking, rate limiting,
  and timeouts are plain values you pass the current time into — no runtime, no
  hidden threads, no global state.
- **Keep adoption cost low.** Small independent crates compile fast and pull in
  nothing extra.
- **One cohesive family — take one or all.** Use a single crate for one job, or
  the `reliakit` umbrella for several; every block follows the same conventions
  and the same zero-dependency, `no_std`, no-`unsafe` rules. Reliability patterns
  usually mean stitching together unrelated crates with different designs and
  dependency trees — here they are built to fit.

## Core features

| Area | Crate(s) | What you get |
|---|---|---|
| Validated primitives | `reliakit-primitives` | `Port`, `Email`, `HttpUrl`, `Hostname`, `BoundedStr`, `Percent`, `SemVer`, `Uuid`, `HumanDuration`, … |
| Secret redaction | `reliakit-secret` | `Secret<T>`, `SecretString`, opt-in `expose_secret` |
| Validation traits | `reliakit-validate` | `Validate` trait, `ValidationError` that collects every field violation |
| Bounded collections | `reliakit-collections` | `BoundedVec<T, MIN, MAX>` with enforced size invariants |
| Canonical binary codec | `reliakit-codec` | `CanonicalEncode` / `CanonicalDecode`, strict decoding |
| Strict JSON | `reliakit-json` | Strict parser + limits, deterministic output, typed `JsonEncode` / `JsonDecode` |
| Strict CSV | `reliakit-csv` | Strict, bounded reader + deterministic writer, typed `CsvEncode` / `CsvDecode` |
| Resilience | `reliakit-backoff`, `reliakit-bulkhead`, `reliakit-circuit`, `reliakit-ratelimit`, `reliakit-timeout` | Retry backoff, concurrency limiter, circuit breaker, token-bucket rate limiter, deadlines — all clock-agnostic |
| Retry helper | `reliakit-retry` | `RetryPolicy` + `retry` / `retry_with_sleep` / `retry_async`; runtime-agnostic, never sleeps internally |
| Health reporting | `reliakit-health` | `Health` status + criticality-aware aggregator for `/health`, probes, and status pages |
| Shared clock | `reliakit-core` | `Clock` trait + `ManualClock` / `MonotonicClock` |
| Derive helpers | `reliakit-derive` | `#[derive(CanonicalEncode, CanonicalDecode, JsonEncode, JsonDecode)]` |
| Decision logic | `reliakit-decide` | Deterministic utility-based decisions (`Reasoner` with `decide`/`explain`/`gate`/`Policy`) |

## Real-world use cases

### 1. Backend / API input validation

Validate request fields into typed values once, near the edge:

```rust
use reliakit_primitives::{Email, Port};

let contact = Email::new("ops@example.com")?;
let port = Port::new(8080)?;
assert_eq!(contact.domain(), "example.com");
assert_eq!(port.get(), 8080);
```

### 2. CLI tools / config parsing + secret-safe logging

Turn loosely-typed config into trusted types, and keep credentials out of logs:

```rust
use reliakit_primitives::{BoundedStr, Percent, Port};
use reliakit_secret::{ExposeSecret, SecretString};

type ServiceName = BoundedStr<3, 32>;

let name = ServiceName::new("api-service")?;
let success_rate = Percent::new(99)?;
let port = Port::new(8080)?;
let api_key = SecretString::from_string("rk_live_example");

assert_eq!(api_key.to_string(), "[REDACTED]"); // never leaks in Display/Debug/logs
assert_eq!(api_key.expose_secret(), "rk_live_example"); // explicit opt-in to read it
```

### 3. Microservices / external calls — rate limiting and circuit breaking

Clock-agnostic resilience values you drive with your own time source:

```rust
use reliakit_ratelimit::RateLimiter;
use reliakit_circuit::{CircuitBreaker, State};

// Allow bursts of up to 10, refilling 1 token every 100 ms (~10/sec).
let mut limiter = RateLimiter::new(10, 1, 100);
assert!(limiter.try_acquire_one(0));

// Trip after 3 consecutive failures; stay open for 30_000 ms.
let mut breaker = CircuitBreaker::new(3, 30_000);
for _ in 0..3 {
    let _ = breaker.allow(0);
    breaker.on_failure(0);
}
assert_eq!(breaker.state(), State::Open); // fail fast instead of hammering a down service
```

### 4. Data pipelines / bounded input handling

```rust
use reliakit_codec::{decode_from_slice_exact, encode_to_vec};
use reliakit_derive::{CanonicalDecode, CanonicalEncode};

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Record { id: u64, ok: bool }

let bytes = encode_to_vec(&Record { id: 7, ok: true })?;
assert_eq!(decode_from_slice_exact::<Record>(&bytes)?, Record { id: 7, ok: true });
```

### 5. Typed JSON for APIs and storage

```rust
use reliakit_derive::{JsonDecode, JsonEncode};
use reliakit_json::{from_json_str, to_json_string};

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Event { id: u64, name: String }

let json = to_json_string(&Event { id: 1, name: "deploy".into() });
assert_eq!(json, r#"{"id":1,"name":"deploy"}"#);
assert_eq!(from_json_str::<Event>(&json).unwrap(), Event { id: 1, name: "deploy".into() });
```

### 6. Embedded / `no_std`-friendly constraints

The resilience crates and the allocation-free primitives work without `std` or
even `alloc`. A `CircuitBreaker` or `RateLimiter` is a small `Copy` value with
saturating, panic-free integer math — you pass a `u64` tick in, so it runs on
embedded targets just as well as on a server.

### 7. Protocols and deterministic encoding (incl. blockchain)

Because `reliakit-codec` defines one canonical byte representation per type and
`reliakit-json` can emit RFC 8785 (JCS) canonical JSON (opt-in `canonical`
feature), the same value always produces the same bytes — useful for cache keys,
content addressing, and hashing or signing in protocol and blockchain work. This
is one use case among many, not the focus.

## Quick start / installation

The quickest way in is the umbrella crate `reliakit`, which re-exports every
building block behind a feature flag. Add one dependency and enable only the
pieces you want:

```toml
[dependencies]
reliakit = { version = "0.1", features = ["ratelimit", "secret"] }
```

```rust
use reliakit::ratelimit::RateLimiter;
use reliakit::secret::Secret;
```

Nothing is pulled in beyond the features you enable, so the zero-dependency,
`no_std`-friendly nature of each block is preserved. Use `features = ["full"]`
for everything.

Prefer the tightest possible dependency graph? The crates are fully independent —
depend on just the ones you need:

```toml
[dependencies]
reliakit-primitives  = "0.4"
reliakit-secret      = "0.1"
reliakit-validate    = "0.3"
reliakit-collections = "0.3"
reliakit-codec       = "0.2"
reliakit-json        = "0.2"
reliakit-csv         = "0.1"
reliakit-backoff     = "0.1"
reliakit-retry       = "0.1"
reliakit-bulkhead    = "0.1"
reliakit-health      = "0.1"
reliakit-circuit     = "0.2"
reliakit-ratelimit   = "0.1"
reliakit-timeout     = "0.1"
reliakit-core        = "0.1"
reliakit-derive      = "0.1"
reliakit-decide      = "0.1"
```

Each crate is independent — most projects use two or three. The minimum
supported Rust version is **1.85**.

## Crate overview

| Crate | Purpose | Use when | Status |
|---|---|---|---|
| [`reliakit-primitives`](https://crates.io/crates/reliakit-primitives) | Validated primitive types | You want `Email`, `Port`, `Percent`, `BoundedStr`, … instead of unchecked strings/numbers. | Published (pre-1.0) |
| [`reliakit-secret`](https://crates.io/crates/reliakit-secret) | Secret redaction wrappers | A value must not leak through `Debug`/`Display`/logs. | Published (pre-1.0) |
| [`reliakit-validate`](https://crates.io/crates/reliakit-validate) | Validation trait + error aggregation | You want to collect every field error at once. | Published (pre-1.0) |
| [`reliakit-collections`](https://crates.io/crates/reliakit-collections) | Bounded collection types | A collection must stay within a fixed size range. | Published (pre-1.0) |
| [`reliakit-codec`](https://crates.io/crates/reliakit-codec) | Canonical binary encoding/decoding | You need deterministic bytes (cache keys, fixtures, framing). | Published (pre-1.0) |
| [`reliakit-json`](https://crates.io/crates/reliakit-json) | Strict, deterministic JSON + typed encode/decode | You parse untrusted JSON or need predictable output. | Published (pre-1.0) |
| [`reliakit-csv`](https://crates.io/crates/reliakit-csv) | Strict, deterministic CSV + typed encode/decode | You parse untrusted CSV or need reproducible output. | Published (pre-1.0) |
| [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff) | Retry backoff delays + jitter | You retry an operation and want explicit spacing. | Published (pre-1.0) |
| [`reliakit-retry`](https://crates.io/crates/reliakit-retry) | Runtime-agnostic retry helper (sync + async) | You retry fallible operations and want attempt limits, backoff, and an error classifier without forcing a runtime. | Published (pre-1.0) |
| [`reliakit-bulkhead`](https://crates.io/crates/reliakit-bulkhead) | Concurrency limiter (counting semaphore) | You cap how many operations run at once and shed the rest. | Published (pre-1.0) |
| `reliakit-health` | Health status + criticality-aware aggregator | You expose a `/health`/`readyz` endpoint or status page. | Unreleased (pre-1.0) |
| [`reliakit-circuit`](https://crates.io/crates/reliakit-circuit) | Circuit breaker state machine | You want to stop calling a failing dependency. | Published (pre-1.0) |
| [`reliakit-ratelimit`](https://crates.io/crates/reliakit-ratelimit) | Token-bucket rate limiter | You cap how often something may happen. | Published (pre-1.0) |
| [`reliakit-timeout`](https://crates.io/crates/reliakit-timeout) | Deadlines / time budgets | You track whether a budget has run out. | Published (pre-1.0) |
| [`reliakit-core`](https://crates.io/crates/reliakit-core) | Shared `Clock` trait + clocks | You want a ready-made `u64` time source for the resilience crates. | Published (pre-1.0) |
| [`reliakit-derive`](https://crates.io/crates/reliakit-derive) | Derive macros for codec + JSON traits | You want `#[derive(...)]` instead of hand-writing encode/decode. | Published (pre-1.0) |
| [`reliakit-decide`](https://crates.io/crates/reliakit-decide) | Deterministic utility decision engine | You want graded, explainable, testable decisions (routing, selection, when to call an LLM). | Published (pre-1.0) |

The resilience crates (`backoff`, `bulkhead`, `circuit`, `ratelimit`, `timeout`)
are **clock-agnostic** — you pass the time in (where they need it), so they
compose and work in sync, async, and embedded code: a rate limiter decides
whether to call, a bulkhead bounds how many calls run at once, a circuit breaker
stops calling a failing dependency, backoff spaces out retries, and a timeout
bounds how long you wait.

## Design philosophy

- **Small, independent crates** you adopt one at a time — no framework lock-in.
- **Explicit invariants** validated at construction; invalid states are hard to
  represent.
- **Boring, predictable APIs** — plain types and traits, no hidden runtime,
  threads, or global state.
- **Zero runtime dependencies** (standard library + other `reliakit-*` crates
  only) and `#![forbid(unsafe_code)]` throughout.
- **Deterministic behavior** — same input, same output; saturating arithmetic in
  the resilience crates.
- **Feature-gated integrations** — cross-crate links (e.g. codec ↔ primitives,
  JSON ↔ validate) are opt-in features, never default.

## When to use Reliakit

- Validating config, CLI flags, environment, or request payloads at the boundary.
- Backend services, bots, and libraries that need small typed constraints.
- Keeping secrets out of logs and diagnostics.
- Deterministic encoding for cache keys, fixtures, protocols, or signing.
- Adding explicit retry/backoff/rate-limit/circuit-breaker/timeout logic without
  pulling in an async runtime.
- Embedded or `no_std` code that needs constrained values or resilience math.

## When not to use Reliakit

Reliakit is a set of small building blocks, not a platform. Reach for something
else when you need:

- a full web framework, HTTP stack, or async runtime integration;
- a complete serialization ecosystem with format plugins and zero-copy
  deserialization;
- schema validation, query/database tooling, or an ORM;
- domain-specific validators beyond Reliakit's intentionally narrow checks
  (its `Email`/`HttpUrl` validation is pragmatic, not a full RFC implementation).

## Feature flags & `no_std`

Reliakit is `no_std`-friendly where it makes sense, but the details differ per
crate — check each crate's README for the exact flags.

- **Default features** enable `std`, which implies `alloc`. Building with
  `--no-default-features` gives the `no_std` subset.
- **Allocation-backed APIs need `alloc`.** Owned types (`String`/`Vec`-backed,
  e.g. `Email`, `BoundedStr`, `SecretString`, `BoundedVec`, all of
  `reliakit-json` and `reliakit-csv`) require the `alloc` feature; the
  allocation-free primitives
  (`Port`, `Percent`, `Uuid`, `MacAddress`, `HumanDuration`, numeric types) work
  with neither.
- **The resilience crates are pure `core`.** `reliakit-backoff`,
  `reliakit-retry`, `reliakit-circuit`, `reliakit-ratelimit`, `reliakit-timeout`,
  and `reliakit-core` need no allocation at all. `circuit`, `ratelimit`, and
  `timeout` offer an optional `core` feature that adds `*_now(clock)` convenience
  methods. `reliakit-retry` never sleeps or spawns; the caller injects any
  waiting, so it forces no async runtime.
- **`reliakit-derive` is a proc-macro crate.** It runs at compile time on the
  host, so the usual `no_std`/`alloc` discussion does not apply to it; the code
  it generates inherits the `no_std` support of the trait crate.

## Contributing

Contributions are welcome. New here? The pinned
[good first issues](https://github.com/satyakwok/reliakit/labels/good%20first%20issue)
are a friendly place to start, and
[help wanted](https://github.com/satyakwok/reliakit/labels/help%20wanted) issues
need a bit more design judgment. Please open an issue before submitting a pull
request for non-trivial changes so the direction can be discussed first.

- Keep each crate minimal and focused.
- Add tests for any new public API surface.
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting.

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for guidelines, [`CHANGELOG.md`](./CHANGELOG.md)
for release notes, [`RELEASING.md`](./RELEASING.md) for the release process, and
[`SECURITY.md`](./SECURITY.md) for vulnerability reporting.

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

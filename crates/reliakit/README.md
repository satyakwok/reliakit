<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit

[![Crates.io](https://img.shields.io/crates/v/reliakit.svg)](https://crates.io/crates/reliakit)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit.svg)](https://crates.io/crates/reliakit)
[![Docs.rs](https://docs.rs/reliakit/badge.svg)](https://docs.rs/reliakit)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

The umbrella crate for the **Reliakit** reliability toolkit: one name that
re-exports the individual `reliakit-*` building blocks behind feature flags.

This crate contains no logic of its own. It exists so you can depend on a single
name and turn on only the pieces you need. Nothing is pulled in by default beyond
the `std` flag — each module appears only when its feature is enabled, so the
zero-dependency, `no_std`-friendly nature of each building block is preserved.

## What This Crate Does

- Gives the toolkit **one import name** instead of a dozen.
- Re-exports each building block as a module: `reliakit::ratelimit`,
  `reliakit::secret`, `reliakit::circuit`, and so on.
- Forwards `std`/`alloc` to whichever sub-crates you enable, so `no_std` works
  through the umbrella exactly as it does for the individual crates.

## When To Use It

- You want several Reliakit building blocks and prefer one dependency line and
  one version to track.
- You are exploring the toolkit and want everything reachable under one name
  (`features = ["full"]`).

## When Not To Use It

- You need exactly one building block and want the tightest possible dependency
  graph — depend on that crate directly (e.g. `reliakit-ratelimit`). The umbrella
  adds no capability, only convenience.

## Installation

Enable only the building blocks you need:

```toml
[dependencies]
reliakit = { version = "0.1", features = ["ratelimit", "secret"] }
```

```rust
use reliakit::ratelimit::RateLimiter;
use reliakit::secret::Secret;
```

`no_std` with `alloc`:

```toml
[dependencies]
reliakit = { version = "0.1", default-features = false, features = ["alloc", "primitives"] }
```

Everything at once:

```toml
[dependencies]
reliakit = { version = "0.1", features = ["full"] }
```

## Examples

The building blocks are designed to compose, all reached through the one
`reliakit` name.

**A resilient client** — guard one call to a flaky dependency with an overall
deadline, rate limiting, a circuit breaker, and backoff, driven by a single clock:

```sh
cargo run -p reliakit --example resilient_client \
  --features "timeout ratelimit circuit backoff"
```

See [`examples/resilient_client.rs`](./examples/resilient_client.rs).

**A config check** — validate a service config from raw, untrusted strings,
reporting every problem at once with the credential kept out of logs:

```sh
cargo run -p reliakit --example config_check \
  --features "primitives validate secret"
```

See [`examples/config_check.rs`](./examples/config_check.rs).

**Typed JSON** — parse untrusted JSON strictly, then lift the raw fields into
validated primitive types:

```sh
cargo run -p reliakit --example typed_json --features "json primitives"
```

See [`examples/typed_json.rs`](./examples/typed_json.rs).

## Building Blocks

| Feature | Module | Crate |
|---|---|---|
| `core` | `reliakit::core` | [`reliakit-core`](https://crates.io/crates/reliakit-core) — `Clock` trait + clocks |
| `primitives` | `reliakit::primitives` | [`reliakit-primitives`](https://crates.io/crates/reliakit-primitives) — validated primitive types |
| `secret` | `reliakit::secret` | [`reliakit-secret`](https://crates.io/crates/reliakit-secret) — secret redaction wrappers |
| `validate` | `reliakit::validate` | [`reliakit-validate`](https://crates.io/crates/reliakit-validate) — validation traits + error aggregation |
| `collections` | `reliakit::collections` | [`reliakit-collections`](https://crates.io/crates/reliakit-collections) — bounded collections |
| `codec` | `reliakit::codec` | [`reliakit-codec`](https://crates.io/crates/reliakit-codec) — canonical binary encoding |
| `csv` | `reliakit::csv` | [`reliakit-csv`](https://crates.io/crates/reliakit-csv) — strict, bounded CSV |
| `backoff` | `reliakit::backoff` | [`reliakit-backoff`](https://crates.io/crates/reliakit-backoff) — retry backoff policies |
| `retry` | `reliakit::retry` | [`reliakit-retry`](https://crates.io/crates/reliakit-retry) — runtime-agnostic retry helpers |
| `circuit` | `reliakit::circuit` | [`reliakit-circuit`](https://crates.io/crates/reliakit-circuit) — circuit breaker |
| `ratelimit` | `reliakit::ratelimit` | [`reliakit-ratelimit`](https://crates.io/crates/reliakit-ratelimit) — token-bucket rate limiter |
| `timeout` | `reliakit::timeout` | [`reliakit-timeout`](https://crates.io/crates/reliakit-timeout) — deadlines and timeouts |
| `json` | `reliakit::json` | [`reliakit-json`](https://crates.io/crates/reliakit-json) — strict, bounded JSON |
| `derive` | `reliakit::derive` | [`reliakit-derive`](https://crates.io/crates/reliakit-derive) — derive macros |
| `decide` | `reliakit::decide` | [`reliakit-decide`](https://crates.io/crates/reliakit-decide) — utility decision engine |

## Feature Flags

| Feature | Default | Effect |
|---|---|---|
| `std` | yes | Implies `alloc`; forwards `std` to enabled crates. |
| `alloc` | via `std` | Forwards `alloc` to enabled crates that need owned storage. |
| `core` | no | Adds `reliakit::core` and enables the clock-aware `*_now` methods of any enabled resilience crate. |
| `<crate>` | no | Adds that crate's module (see table above). |
| `full` | no | Enables every building block. |
| `json-canonical` | no | Enables `reliakit-json`'s RFC 8785 canonical serialization. |
| `json-primitives` | no | Typed JSON extraction into `reliakit-primitives`. |
| `json-validate` | no | Accumulating JSON field validation into `reliakit-validate`. |
| `codec-primitives` | no | Canonical codec impls for `reliakit-primitives` types. |

## `no_std`

`no_std`-compatible (`default-features = false`). Enable the umbrella's `alloc`
feature for the modules whose owned storage is gated behind it — `primitives`,
`secret`, `validate`, `collections`, and `codec`. `json` and `decide` always
include `alloc` on their own, so they need no extra feature. The pure-`core`
blocks (`backoff`, `circuit`, `ratelimit`, `timeout`) need neither `std` nor
`alloc`.

## Safety

`#![forbid(unsafe_code)]`. The umbrella adds no code beyond re-exports; each
building block forbids unsafe code in its own right.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Pre-1.0. A thin re-export layer over the `reliakit-*` building blocks; its public
surface is the set of feature flags, which may gain backward-compatible additions
before a 1.0 release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

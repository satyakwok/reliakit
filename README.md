<p align="center">
  <img src="./assets/reliakit-logo.png" alt="Reliakit" width="520">
</p>

# Reliakit

Reusable Rust primitives and utility crates for building correct, safe, and reliable libraries and applications.

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/satyakwok/reliakit?style=flat)](https://github.com/satyakwok/reliakit/stargazers)
[![GitHub issues](https://img.shields.io/github/issues/satyakwok/reliakit)](https://github.com/satyakwok/reliakit/issues)
[![Last commit](https://img.shields.io/github/last-commit/satyakwok/reliakit)](https://github.com/satyakwok/reliakit/commits/main)

Reliakit is a Rust workspace for reusable reliability primitives and utility crates.

It focuses on small, composable building blocks for writing correct, safe, and reliable Rust libraries and applications.

Each crate is designed to be usable independently.

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

### `reliakit-core`

Planned.

Shared core types, traits, and errors used across Reliakit crates.

### `reliakit-derive`

Planned.

Derive macros for validation and constrained types.

## Installation

From crates.io:

```toml
[dependencies]
reliakit-primitives = "0.2"
reliakit-secret = "0.1"
```

Or depend on the Git repository directly:

```toml
[dependencies]
reliakit-primitives = { git = "https://github.com/satyakwok/reliakit", package = "reliakit-primitives" }
reliakit-secret = { git = "https://github.com/satyakwok/reliakit", package = "reliakit-secret" }
```

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

## Non-Goals

Reliakit is not:

- an async runtime,
- a web framework,
- an ORM,
- a logging framework,
- a replacement for `serde`,
- a replacement for `tokio`,
- a replacement for `clap`,
- a replacement for `anyhow`,
- a replacement for `thiserror`,
- a replacement for `hashbrown`,
- a replacement for `syn`.

Reliakit is intended to provide focused primitives and utility crates, not replace mature ecosystem foundations.

## Workspace Layout

```text
reliakit/
|-- crates/
|   `-- reliakit-primitives/
|       `-- examples/
|-- Cargo.toml
|-- README.md
`-- LICENSE
```

## Status

Active. Reliakit is published as a real Rust library workspace and follows
normal Rust crate versioning.

The current focus is a small, well-tested `reliakit-primitives` crate before
adding more workspace crates.

Logo assets are stored under [`assets/`](./assets/).

## Roadmap

Current:

- `reliakit-primitives`
- `reliakit-secret`

Planned:

- `reliakit-core`
- `reliakit-collections`
- `reliakit-validate`
- `reliakit-derive`

## Contributing

Contributions are welcome. Please open an issue before submitting a pull request for non-trivial changes so the direction can be discussed first.

- Keep each crate minimal and focused.
- Add tests for any new public API surface.
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting.

See [`CONTRIBUTING.md`](./CONTRIBUTING.md) for contribution guidelines,
[`CHANGELOG.md`](./CHANGELOG.md) for release notes, and
[`SECURITY.md`](./SECURITY.md) for vulnerability reporting.

## Star History

<a href="https://www.star-history.com/?repos=satyakwok%2Freliakit&type=Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=satyakwok/reliakit&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=satyakwok/reliakit&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=satyakwok/reliakit&type=Date" />
 </picture>
</a>

## License

Licensed under the MIT License. See `LICENSE`.

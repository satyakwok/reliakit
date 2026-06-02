<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-primitives

Type-safe primitives for constrained and reliability-oriented Rust values.

[![Crates.io](https://img.shields.io/crates/v/reliakit-primitives.svg)](https://crates.io/crates/reliakit-primitives)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-primitives.svg)](https://crates.io/crates/reliakit-primitives)
[![Docs.rs](https://docs.rs/reliakit-primitives/badge.svg)](https://docs.rs/reliakit-primitives)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

`reliakit-primitives` provides small owned wrapper types for values that should satisfy common constraints before they move through an application or library boundary.

The crate has no dependencies and forbids unsafe code.

## When To Use It

Use this crate when a value has simple validity rules that should be checked once and then carried as a typed value:

- names and identifiers that must not be empty,
- strings with minimum or maximum character lengths,
- percentages constrained to `0..=100`,
- ports constrained to `1..=65535`,
- byte sizes that should display consistently.

## When Not To Use It

Do not use this crate as a replacement for domain-specific validation, parsing, serialization, or schema libraries. The types here are intentionally small and general.

## Installation

```toml
[dependencies]
reliakit-primitives = "0.1"
```

For `no_std` environments:

```toml
[dependencies]
reliakit-primitives = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Examples

### Non-empty strings

```rust
use reliakit_primitives::NonEmptyStr;

let name = NonEmptyStr::new("service-api")?;
```

### Bounded strings

```rust
use reliakit_primitives::BoundedStr;

type Username = BoundedStr<3, 32>;

let username = Username::new("satyakwok")?;
```

### Numeric primitives

```rust
use reliakit_primitives::{ByteSize, Percent, Port};

let limit = ByteSize::from_mb(10);
let threshold = Percent::new(80)?;
let port = Port::new(3000)?;
```

## Available Types

| Type | Description |
|---|---|
| `NonEmptyStr` | Owned string that is not empty and not whitespace-only |
| `BoundedStr<MIN, MAX>` | Owned string constrained by character length |
| `Percent` | Percentage value from `0` to `100` inclusive |
| `Port` | TCP/UDP port from `1` to `65535` inclusive |
| `ByteSize` | Byte size value with human-readable display output |

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables `std::error::Error` for `PrimitiveError` |
| `alloc` | no | Enables allocation-backed types without `std` |

## `no_std`

The crate supports `no_std` environments when `std` feature is disabled and `alloc` is available.

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust stable. No nightly features are used.

## Status

Active. The `0.1.x` API is considered stable for the current set of types.

## Contributing

See [CONTRIBUTING.md](https://github.com/satyakwok/reliakit/blob/main/CONTRIBUTING.md).

## License

Licensed under the [MIT License](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

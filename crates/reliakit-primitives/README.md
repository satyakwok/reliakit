<p align="center">
  <img src="../../assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-primitives

Type-safe primitives for constrained and reliability-oriented Rust values.

[![Crates.io](https://img.shields.io/crates/v/reliakit-primitives.svg)](https://crates.io/crates/reliakit-primitives)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-primitives.svg)](https://crates.io/crates/reliakit-primitives)
[![Docs.rs](https://docs.rs/reliakit-primitives/badge.svg)](https://docs.rs/reliakit-primitives)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE)

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

After publishing:

```toml
[dependencies]
reliakit-primitives = "0.1"
```

Until then:

```toml
[dependencies]
reliakit-primitives = { git = "https://github.com/satyakwok/reliakit", package = "reliakit-primitives" }
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

- `NonEmptyStr`: owned string that is not empty and not whitespace-only.
- `BoundedStr<MIN, MAX>`: owned string constrained by character length.
- `Percent`: percentage value from `0` to `100` inclusive.
- `Port`: TCP/UDP port from `1` to `65535` inclusive.
- `ByteSize`: byte size value with human-readable display output.

## Feature Flags

- `default = ["std"]`
- `std`: enables `std::error::Error` for primitive errors.
- `alloc`: marker feature for allocation-backed usage without `std`.

## `no_std`

The crate is designed for `no_std` usage when default features are disabled and `alloc` is available for string-backed types.

```toml
[dependencies]
reliakit-primitives = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Safety

This crate forbids unsafe code.

## Status

Active. The `0.1.x` API is considered stable for the current set of types.

## License

Licensed under the MIT License. See `../../LICENSE`.

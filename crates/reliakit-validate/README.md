<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-validate

Composable validation traits and error types for Rust structs and values.

[![Crates.io](https://img.shields.io/crates/v/reliakit-validate.svg)](https://crates.io/crates/reliakit-validate)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-validate.svg)](https://crates.io/crates/reliakit-validate)
[![Docs.rs](https://docs.rs/reliakit-validate/badge.svg)](https://docs.rs/reliakit-validate)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

`reliakit-validate` provides a small set of types for expressing validation rules as part of a type's contract, collecting multiple failures at once, and carrying proof of validation in the type system.

The crate has no dependencies and forbids unsafe code.

## When To Use It

Use this crate when:

- a struct or value has validity rules that go beyond what a type alone can express,
- you want to collect all validation failures at once rather than short-circuiting on the first one,
- you want function signatures to carry proof that a value was validated (`Valid<T>`),
- you are building a form, API handler, config loader, or CLI where user-facing error messages should name the failing field.

## When Not To Use It

This crate covers struct- and value-level validation rules. The following are
out of scope:

- type-level constraints at construction time, which belong in `reliakit-primitives`,
- schema validation and deserialization, which belong in a dedicated parsing library,
- domain-specific business rules, which belong in your own code.

## Installation

```toml
[dependencies]
reliakit-validate = "0.3"
```

For `no_std` environments:

```toml
[dependencies]
reliakit-validate = { version = "0.3", default-features = false, features = ["alloc"] }
```

## Examples

### Single-field validation

```rust
use reliakit_validate::{Validate, Valid, ValidationError};

struct Score(u8);

impl Validate for Score {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        if self.0 > 100 {
            return Err(ValidationError::new("score must not exceed 100"));
        }
        Ok(())
    }
}

let score = Valid::new(Score(95)).unwrap();
assert_eq!(score.0, 95);

assert!(Valid::new(Score(101)).is_err());
```

### Multi-field struct validation

```rust
use reliakit_validate::{Validate, Valid, ValidationError, Violation};

struct CreateUser {
    name: String,
    age: u8,
}

impl Validate for CreateUser {
    type Error = ValidationError;

    fn validate(&self) -> Result<(), Self::Error> {
        let mut errors = ValidationError::empty();

        if self.name.is_empty() {
            errors.push(Violation::with_field("name", "must not be empty"));
        }
        if self.age < 18 {
            errors.push(Violation::with_field("age", "must be at least 18"));
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

let result = CreateUser { name: String::new(), age: 15 }.validate();
let err = result.unwrap_err();
assert_eq!(err.len(), 2);
```

### Building errors with chaining

```rust
use reliakit_validate::{ValidationError, Violation};

let error = ValidationError::empty()
    .with(Violation::with_field("email", "invalid format"))
    .with(Violation::with_field("password", "too short"));

assert_eq!(error.len(), 2);
println!("{error}"); // "email: invalid format; password: too short"
```

### Conditional rules with `require` / `finish`

`require` records a violation only when a check fails, and `finish` turns the
accumulated violations into a `Result` — so you cannot forget the final
emptiness check:

```rust
use reliakit_validate::{ValidationError, Violation};

let name = "";
let age = 15;

let result = ValidationError::empty()
    .require(!name.is_empty(), Violation::with_field("name", "must not be empty"))
    .require_field(age >= 18, "age", "must be at least 18")
    .finish();

assert_eq!(result.unwrap_err().len(), 2);
```

## API Overview

| Item | Description |
|---|---|
| `Validate` | Trait for types that can validate themselves |
| `Valid<T>` | Zero-cost wrapper carrying proof of successful validation |
| `ValidationError` | Error collecting one or more `Violation`s |
| `Violation` | A single failed constraint with optional field name |
| `ValidateResult<T>` | `Result<T, ValidationError>` alias |

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables `std::error::Error` for `ValidationError`; implies `alloc` |
| `alloc` | no | Enables `ValidationError` and `ValidateResult` (backed by `Vec`) |

## `no_std`

The crate supports `no_std`. The `Validate` trait, `Valid<T>`, and `Violation`
are available without `alloc` — implement `Validate` with your own error type in
allocation-free contexts. `ValidationError` and `ValidateResult` require the
`alloc` feature (enabled by default via `std`).

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust 1.85 stable. No nightly features are used.

## Status

Active. The `0.1.x` API is considered stable.

## Contributing

See [CONTRIBUTING.md](https://github.com/satyakwok/reliakit/blob/main/CONTRIBUTING.md).

## License

Licensed under the [MIT License](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

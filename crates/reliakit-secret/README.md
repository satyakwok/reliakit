<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-secret

Secret-safe wrappers for Rust values that should not leak through formatting or
diagnostics.

[![Crates.io](https://img.shields.io/crates/v/reliakit-secret.svg)](https://crates.io/crates/reliakit-secret)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-secret.svg)](https://crates.io/crates/reliakit-secret)
[![Docs.rs](https://docs.rs/reliakit-secret/badge.svg)](https://docs.rs/reliakit-secret)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-secret)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-secret)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

`reliakit-secret` provides `Secret<T>`, a small wrapper that redacts its inner
value in `Debug` and `Display` output. Access to the wrapped value is explicit
through `ExposeSecret`.

The crate has no dependencies and forbids unsafe code.

## What This Crate Does

This crate helps prevent accidental secret leaks in logs, error messages, debug
output, and diagnostic reports.

Instead of passing a raw password, token, or API key through a public API, wrap it
as a `Secret<T>`:

```rust
use reliakit_secret::{ExposeSecret, Secret};

fn connect(token: Secret<String>) {
    assert_eq!(format!("{token:?}"), "Secret([REDACTED])");
    let raw_token = token.expose_secret();
    assert!(!raw_token.is_empty());
}
```

## What This Crate Does Not Do

This crate does not provide memory zeroization, encryption, process isolation, or
protection against memory inspection. It is a formatting and diagnostics safety
primitive.

## Installation

```toml
[dependencies]
reliakit-secret = "0.1"
```

For `no_std` without allocation:

```toml
[dependencies]
reliakit-secret = { version = "0.1", default-features = false }
```

For `no_std` with string-backed secrets:

```toml
[dependencies]
reliakit-secret = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Examples

### Generic secret

```rust
use reliakit_secret::{ExposeSecret, Secret};

let token = Secret::new("ghp_example_token");

assert_eq!(format!("{token:?}"), "Secret([REDACTED])");
assert_eq!(format!("{token}"), "[REDACTED]");
assert_eq!(token.expose_secret(), &"ghp_example_token");
```

### String-backed secret

```rust
use reliakit_secret::{ExposeSecret, SecretString};

let password = SecretString::from_string("correct horse battery staple");

assert_eq!(password.expose_secret(), "correct horse battery staple");
assert_eq!(password.expose_str(), "correct horse battery staple");
assert_eq!(password.to_string(), "[REDACTED]");
```

### Consuming a secret

```rust
use reliakit_secret::Secret;

let secret = Secret::new(String::from("token"));
let token = secret.into_inner();

assert_eq!(token, "token");
```

## Available Types

| Type | Description |
|---|---|
| `Secret<T>` | Generic wrapper that redacts `Debug` and `Display` |
| `SecretString` | `String`-backed secret, available with `std` or `alloc` |
| `ExposeSecret<T>` | Trait for explicit shared access |
| `ExposeSecretMut<T>` | Trait for explicit mutable access |

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables the standard library |
| `alloc` | no | Enables `SecretString` without `std` |

## `no_std`

The crate supports `no_std`.

Generic `Secret<T>` works without allocation. `SecretString` requires `alloc` or
`std`.

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Active. The crate is intentionally small and focused on formatting-safe secret
wrappers.

## License

Licensed under the [MIT License](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

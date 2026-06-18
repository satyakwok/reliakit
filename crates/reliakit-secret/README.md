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
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

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
reliakit-secret = "1.0"
```

For `no_std` without allocation:

```toml
[dependencies]
reliakit-secret = { version = "1.0", default-features = false }
```

For `no_std` with string-backed secrets:

```toml
[dependencies]
reliakit-secret = { version = "1.0", default-features = false, features = ["alloc"] }
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

### Redacting a field inside a struct

The common case: a secret living in a config or request struct. Because
`Secret<T>` redacts itself, deriving `Debug` on the parent stays safe: the
secret field shows `[REDACTED]` and the rest prints normally, so you can log the
whole struct.

```rust
use reliakit_secret::SecretString;

#[derive(Debug)]
struct DbConfig {
    host: String,
    port: u16,
    password: SecretString,
}

let cfg = DbConfig {
    host: "db.internal".into(),
    port: 5432,
    password: SecretString::from_string("hunter2"),
};

let rendered = format!("{cfg:?}");
assert!(rendered.contains("db.internal"));
assert!(rendered.contains("[REDACTED]"));
assert!(!rendered.contains("hunter2"));
```

### Constant-time comparison

Checking a presented value against a stored secret with `==` on the exposed
bytes can leak the secret through timing. `ct_eq` compares in time that does not
depend on how many leading bytes match (best-effort, dependency-free; it depends
only on the input length, not the contents).

```rust
use reliakit_secret::SecretString;

let stored = SecretString::from_string("s3cr3t-token");
assert!(stored.ct_eq("s3cr3t-token"));
assert!(!stored.ct_eq("s3cr3t-wrong"));
```

## Available Types

| Type | Description |
|---|---|
| `Secret<T>` | Generic wrapper that redacts `Debug` and `Display` |
| `SecretString` | `String`-backed secret, available with `std` or `alloc` |
| `ExposeSecret<T>` | Trait for explicit shared access |
| `ExposeSecretMut<T>` | Trait for explicit mutable access |

`Secret<T>` where the value is byte-viewable (`String`, `Vec<u8>`, `&[u8]`,
`[u8; N]`, ...) also has `ct_eq` for constant-time comparison.

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

<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-collections

Bounded and reliability-oriented collection types for Rust.

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg)](https://codecov.io/gh/satyakwok/reliakit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

`reliakit-collections` provides collection types with enforced size constraints. Bounds are expressed as const generic parameters and checked at construction time. Mutations that would violate the bounds return errors instead of panicking.

The crate has no dependencies and forbids unsafe code.

## When To Use It

Use this crate when:

- a list must always have at least one element,
- a list must not exceed a known maximum size,
- you want mutation operations to be safe-by-default rather than checked at the call site,
- you are modeling domain concepts like a non-empty recipient list, a capped queue, or a fixed-size batch.

## When Not To Use It

Do not use this crate as a replacement for:

- runtime-sized collections without known bounds — use `std::collections` directly,
- fixed-size stack-allocated arrays — use `[T; N]`,
- `NonEmptyVec<T>` with no upper bound — that type is already in `reliakit-primitives`.

## Installation

```toml
[dependencies]
reliakit-collections = "0.1"
```

For `no_std` environments:

```toml
[dependencies]
reliakit-collections = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Examples

### Bounded recipient list

```rust
use reliakit_collections::BoundedVec;

type RecipientList = BoundedVec<String, 1, 10>;

let mut recipients = RecipientList::new(vec!["alice@example.com".into()]).unwrap();
recipients.push("bob@example.com".into()).unwrap();
assert_eq!(recipients.len(), 2);
```

### Push and pop with bound enforcement

```rust
use reliakit_collections::BoundedVec;

let mut v = BoundedVec::<i32, 1, 3>::new(vec![1, 2, 3]).unwrap();

assert!(v.push(4).is_err()); // at capacity
assert_eq!(v.pop().unwrap(), 3);
assert!(v.pop().is_ok());    // len = 2, above minimum
assert!(v.pop().is_err());   // would go below minimum (1)
```

### Exact-size collection

```rust
use reliakit_collections::BoundedVec;

// Must have exactly 3 elements
type Triple = BoundedVec<i32, 3, 3>;

assert!(Triple::new(vec![1, 2, 3]).is_ok());
assert!(Triple::new(vec![1, 2]).is_err());
assert!(Triple::new(vec![1, 2, 3, 4]).is_err());
```

## Available Types

| Type | Description |
|---|---|
| `BoundedVec<T, MIN, MAX>` | `Vec<T>` constrained to hold between `MIN` and `MAX` elements |

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables `std::error::Error` for `CollectionError` |
| `alloc` | no | Enables `BoundedVec` without `std` |

## `no_std`

The crate supports `no_std` environments when `std` is disabled and `alloc` is available.

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust 1.85 stable. No nightly features are used.

## Status

Active. Not yet published to crates.io.

## Contributing

See [CONTRIBUTING.md](https://github.com/satyakwok/reliakit/blob/main/CONTRIBUTING.md).

## License

Licensed under the [MIT License](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

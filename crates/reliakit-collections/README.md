<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-collections

Bounded and reliability-oriented collection types for Rust.

[![Crates.io](https://img.shields.io/crates/v/reliakit-collections.svg)](https://crates.io/crates/reliakit-collections)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-collections.svg)](https://crates.io/crates/reliakit-collections)
[![Docs.rs](https://docs.rs/reliakit-collections/badge.svg)](https://docs.rs/reliakit-collections)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-collections)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-collections)
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

This crate covers bounded collection types. The following are out of scope:

- runtime-sized collections without known bounds, served by `std::collections`,
- fixed-size stack-allocated arrays, served by `[T; N]`,
- unbounded non-empty vectors, which `reliakit-primitives` already provides as `NonEmptyVec<T>`.

## Installation

```toml
[dependencies]
reliakit-collections = "0.3"
```

For `no_std` environments:

```toml
[dependencies]
reliakit-collections = { version = "0.3", default-features = false, features = ["alloc"] }
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

### Rolling window with a ring buffer

```rust
use reliakit_collections::RingBuffer;

// Keep only the most recent 3 samples.
let mut last3 = RingBuffer::new(3).unwrap();
last3.push(1);
last3.push(2);
last3.push(3);

// Pushing onto a full buffer evicts (and returns) the oldest element.
assert_eq!(last3.push(4), Some(1));
assert_eq!(last3.iter().copied().collect::<Vec<_>>(), [2, 3, 4]);
```

### Bounded map with unique keys

```rust
use reliakit_collections::BoundedMap;

// At most 8 feature flags, keys unique, insertion order preserved.
let mut flags = BoundedMap::<String, bool, 0, 8>::new(vec![]).unwrap();

assert_eq!(flags.insert("dark_mode".into(), true).unwrap(), None);
assert_eq!(flags.insert("dark_mode".into(), false).unwrap(), Some(true)); // replaces, count unchanged
assert_eq!(flags.get(&"dark_mode".to_string()), Some(&false));
assert_eq!(flags.len(), 1);
```

### Bounded set of unique elements

```rust
use reliakit_collections::BoundedSet;

// Track up to 3 active sessions; duplicates are ignored, overflow is rejected.
let mut sessions = BoundedSet::<u32, 0, 3>::new(vec![1, 2]).unwrap();

assert!(sessions.insert(3).unwrap());      // added
assert!(!sessions.insert(2).unwrap());     // already present, no-op
assert!(sessions.insert(4).is_err());      // at capacity
assert!(sessions.contains(&3));
```

## Available Types

| Type | Description |
|---|---|
| `BoundedVec<T, MIN, MAX>` | `Vec<T>` constrained to hold between `MIN` and `MAX` elements |
| `BoundedMap<K, V, MIN, MAX>` | Insertion-ordered map with unique keys and an enforced entry-count range (vec-backed, linear lookup) |
| `BoundedSet<T, MIN, MAX>` | Insertion-ordered set of unique elements with an enforced count range (vec-backed, linear lookup) |
| `RingBuffer<T>` | Fixed-capacity circular buffer that overwrites the oldest element when full |

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables `std::error::Error` for `CollectionError`; implies `alloc` |
| `alloc` | no | Enables `BoundedVec`, `BoundedMap`, `BoundedSet`, and `RingBuffer` (backed by `alloc`) |

## `no_std`

The crate supports `no_std`. `BoundedVec`, `BoundedMap`, `BoundedSet`, and
`RingBuffer` require the `alloc` feature (enabled by default via `std`). The
error types (`CollectionError`, `CollectionResult`) are available without
`alloc`.

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

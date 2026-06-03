<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-primitives

Type-safe primitives for constrained and reliability-oriented Rust values.

[![Crates.io](https://img.shields.io/crates/v/reliakit-primitives.svg)](https://crates.io/crates/reliakit-primitives)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-primitives.svg)](https://crates.io/crates/reliakit-primitives)
[![Docs.rs](https://docs.rs/reliakit-primitives/badge.svg)](https://docs.rs/reliakit-primitives)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-primitives)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-primitives)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

`reliakit-primitives` provides small owned wrapper types for values that should satisfy common constraints before they move through an application or library boundary.

The crate has no dependencies and forbids unsafe code.

The goal is not to parse every domain-specific format perfectly. The goal is to
make common invariants explicit in types, keep validation close to input
boundaries, and avoid passing unchecked strings and numbers through public APIs.

## What This Crate Does

This crate turns common validated values into explicit Rust types.

Instead of passing unchecked `String`, `u16`, `u64`, or `f64` values through
your code, construct a primitive once and let function signatures carry the
invariant:

```rust
use reliakit_primitives::{Email, NonEmptyStr, Port};

fn configure_service(name: NonEmptyStr, contact: Email, port: Port) {
    // These values have already passed their basic validation rules.
}

let name = NonEmptyStr::new("api")?;
let contact = Email::new("ops@example.com")?;
let port = Port::new(8080)?;

configure_service(name, contact, port);
```

This is useful at boundaries such as configuration loading, request parsing,
CLI input, test fixtures, and library APIs.

## When To Use It

Use this crate when a value has simple validity rules that should be checked once and then carried as a typed value:

- names and identifiers that must not be empty,
- strings with minimum or maximum character lengths,
- slugs, email addresses, HTTP URLs, or hex strings,
- percentages, ports, positive integers, or positive floats,
- byte sizes that should display consistently,
- semantic version strings,
- UUIDs in canonical format,
- human-readable durations like `1h30m` or `500ms`.

## When Not To Use It

Use a domain-specific validator, parser, serializer, or schema library when a
value needs rules beyond these small general-purpose primitives.

For example, `Email` is a basic structural validator, not a full RFC 5321 or
deliverability checker. `HttpUrl` checks for an HTTP(S) scheme and a non-empty
host, but it is not a complete URL parser.

## Installation

```toml
[dependencies]
reliakit-primitives = "0.2"
```

For `no_std` environments:

```toml
[dependencies]
reliakit-primitives = { version = "0.2", default-features = false, features = ["alloc"] }
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

### Text primitives

```rust
use reliakit_primitives::{Email, HttpUrl, Slug};

let slug = Slug::new("service-api")?;
let email = Email::new("ops@example.com")?;
let url = HttpUrl::new("https://example.com/health")?;

assert_eq!(email.domain(), "example.com");
assert!(url.is_https());
```

### Structured values

```rust
use reliakit_primitives::{HumanDuration, SemVer, Uuid};

let version = SemVer::parse("1.2.3-beta.1+build.5")?;
let request_id = Uuid::parse("550e8400-e29b-41d4-a716-446655440000")?;
let timeout = HumanDuration::parse("1m30s")?;

assert!(version.is_pre_release());
assert_eq!(request_id.version(), 4);
assert_eq!(timeout.as_duration().as_secs(), 90);
```

`SemVer` implements Rust's total `Ord` consistently with `Eq`. Build metadata
is used as a final tie-breaker for ordering so `BTreeSet`, `BTreeMap`, sorting,
and binary search behave correctly. Use `SemVer::cmp_precedence` when you need
SemVer precedence rules that ignore build metadata.

### Error handling

All fallible constructors return `PrimitiveResult<T>`, an alias for
`Result<T, PrimitiveError>`.

```rust
use reliakit_primitives::{NonEmptyStr, PrimitiveError, PrimitiveErrorKind};

let err = NonEmptyStr::new("   ").unwrap_err();
assert_eq!(err, PrimitiveError::Empty);
assert_eq!(err.kind(), PrimitiveErrorKind::Empty);
```

Use `PrimitiveErrorKind` when you need stable programmatic matching without
depending on display text.

## Available Types

### Strings

| Type | Description |
|---|---|
| `NonEmptyStr` | Owned string that is not empty and not whitespace-only |
| `BoundedStr<MIN, MAX>` | Owned string constrained by character length |
| `Slug` | Lowercase alphanumeric + hyphens, URL-safe |
| `Email` | Basic structural email validation |
| `HttpUrl` | URL that must start with `http://` or `https://` |
| `HexString` | Hexadecimal characters with optional `0x`/`0X` prefix |

### Numbers

| Type | Description |
|---|---|
| `Percent` | Integer percentage from `0` to `100` inclusive |
| `PercentageF64` | Float percentage from `0.0` to `100.0` inclusive |
| `Port` | TCP/UDP port from `1` to `65535` inclusive |
| `PositiveInt` | `u64` strictly greater than zero |
| `PositiveFloat` | Finite `f64` strictly greater than zero |
| `ByteSize` | Byte size value with human-readable display output |

### Collections

| Type | Description |
|---|---|
| `NonEmptyVec<T>` | `Vec<T>` guaranteed to contain at least one element |

### Structured Values

| Type | Description |
|---|---|
| `SemVer` | Semantic version (`1.2.3`, `2.0.0-beta.1`, `1.0.0+build`) |
| `Uuid` | UUID in `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` format |
| `HumanDuration` | Duration parsed from `1h`, `30m`, `45s`, `500ms`, or combinations |

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables `std::error::Error` for `PrimitiveError`; implies `alloc` |
| `alloc` | no | Enables the allocation-backed owned types and collection helpers |

## `no_std`

The crate supports `no_std`. Building with `--no-default-features` (no `std`,
no `alloc`) provides the allocation-free primitives:

- numeric: `Percent`, `PercentageF64`, `Port`, `PositiveInt`, `PositiveFloat`,
  `ByteSize`,
- `Uuid` and `HumanDuration` (parsing and `Display` do not allocate),
- the error types (`PrimitiveError`, `PrimitiveErrorKind`, `PrimitiveResult`).

Enabling `alloc` adds the owned, allocation-backed types: `Slug`, `Email`,
`HttpUrl`, `HexString`, `NonEmptyStr`, `BoundedStr`, `NonEmptyVec`, and `SemVer`.
The default `std` build enables `alloc` for normal application use:

```toml
reliakit-primitives = { version = "0.3", default-features = false, features = ["alloc"] }
```

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Active. The crate is published for real use and follows normal Rust crate
versioning. APIs may still receive compatible refinements before a `1.0`
release.

## Contributing

See [CONTRIBUTING.md](https://github.com/satyakwok/reliakit/blob/main/CONTRIBUTING.md).

## License

Licensed under the [MIT License](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

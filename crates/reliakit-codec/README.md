<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-codec

[![Crates.io](https://img.shields.io/crates/v/reliakit-codec.svg)](https://crates.io/crates/reliakit-codec)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-codec.svg)](https://crates.io/crates/reliakit-codec)
[![Docs.rs](https://docs.rs/reliakit-codec/badge.svg)](https://docs.rs/reliakit-codec)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-codec)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-codec)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

Deterministic canonical binary encoding and decoding traits for Rust.

`reliakit-codec` is a small codec crate for reliability-oriented Rust code. It
defines one canonical binary representation for each supported type and keeps
encoding explicit through handwritten trait implementations.

The crate has no external dependencies by default, supports `no_std` with
`alloc`, and forbids unsafe code.

## Introduction

Binary codecs are often used at boundaries where ambiguity is expensive:
protocol messages, fixtures, cache keys, deterministic tests, and small storage
formats. This crate focuses on those use cases.

It does not try to infer schemas, derive implementations, or support every Rust
type. Instead, it provides simple traits, strict decode behavior, primitive
implementations, and helpers for exact slice decoding.

## What This Crate Does

This crate provides:

- `CanonicalEncode` for writing values in a deterministic binary format,
- `CanonicalDecode` for strict decoding,
- small sink/source traits that work without `std::io`,
- helpers such as `encode_to_vec` and `decode_from_slice_exact`,
- canonical implementations for integers, `bool`, strings, vectors, options,
  results, fixed arrays, and small tuples,
- optional integrations with `reliakit-primitives`.

## When To Use

Use this crate when you want a compact binary representation that is explicit
and deterministic:

- handwritten protocol messages,
- golden test fixtures,
- stable cache keys,
- small local persistence formats,
- library boundaries where canonical encoding matters.

## When Not To Use

Do not use this crate as a Serde replacement, schema language, RPC framework, or
general-purpose serialization ecosystem.

The first version intentionally does not include derive macros, proc macros,
schema negotiation, unordered map support, floats, pointer-sized integers, or
non-canonical alternatives.

`HashMap` and other unordered maps are not supported because their iteration
order is not canonical. Floats are not supported because NaN payloads, signed
zero, and normalization rules need an explicit design before they can be
reliably canonical. `usize` and `isize` are not supported because their width is
platform-dependent.

## Installation

Standalone use:

```toml
[dependencies]
reliakit-codec = "0.1"
```

With optional `reliakit-primitives` integrations:

```toml
[dependencies]
reliakit-codec = { version = "0.1", features = ["primitives"] }
```

For `no_std` with allocation:

```toml
[dependencies]
reliakit-codec = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Core Concepts

### Canonical Encoding

Every supported type has one valid byte representation. Integers use fixed-width
little-endian encoding. Booleans use exactly `0x00` or `0x01`. Length-prefixed
types use a `u32` little-endian length.

### Explicit Decode

Decode is performed through `CanonicalDecode`. Structs and enums implement the
trait manually so field order, enum tags, and validation rules are visible in
normal Rust code.

### Strict Errors

Invalid bytes fail. Invalid UTF-8 fails. Unknown enum tags fail. Exact decoding
fails when trailing bytes remain.

`CodecErrorKind` provides stable categories for programmatic handling, while
`CodecError::message()` gives a concise explanation.

### Manual Trait Implementations

There are no derive macros in this version.

```rust
use reliakit_codec::{CanonicalDecode, CanonicalEncode, CodecError, DecodeSource, EncodeSink};

struct Point {
    x: u16,
    y: u16,
}

impl CanonicalEncode for Point {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        self.x.encode(writer)?;
        self.y.encode(writer)
    }
}

impl CanonicalDecode for Point {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        Ok(Self {
            x: u16::decode(reader)?,
            y: u16::decode(reader)?,
        })
    }
}
```

## Binary Format Overview

| Type | Encoding |
|---|---|
| `u8`, `i8` | one byte |
| `u16`, `i16` | fixed-width little-endian |
| `u32`, `i32` | fixed-width little-endian |
| `u64`, `i64` | fixed-width little-endian |
| `u128`, `i128` | fixed-width little-endian |
| `bool` | `0x00` for false, `0x01` for true |
| `str`, `String` | `u32` little-endian byte length, then UTF-8 bytes |
| `Vec<T>` | `u32` little-endian item count, then each item |
| `Option<T>` | `0x00` for `None`, `0x01` then value for `Some` |
| `Result<T, E>` | `0x00` then value for `Ok`, `0x01` then error for `Err` |
| `[T; N]` | items in order |
| tuples up to arity 4 | fields in order |

Any other `bool`, `Option`, or `Result` tag is invalid. Enum tags in user-defined
types should follow the same strict pattern.

Generic `[T; N]` decoding requires the `alloc` feature in this version because
the crate forbids unsafe code and Rust 1.85 does not provide a stable fallible
array initializer. In no-alloc builds, `[u8; N]` decoding is supported directly.

## Examples

Encode and decode a primitive:

```rust
use reliakit_codec::{decode_from_slice_exact, encode_to_vec};

let encoded = encode_to_vec(&8080u16)?;
assert_eq!(encoded, [0x90, 0x1f]);
assert_eq!(decode_from_slice_exact::<u16>(&encoded)?, 8080);
# Ok::<(), reliakit_codec::CodecError>(())
```

Reject trailing bytes:

```rust
use reliakit_codec::{decode_from_slice_exact, CodecErrorKind};

let err = decode_from_slice_exact::<u8>(&[1, 2]).unwrap_err();
assert_eq!(err.kind(), CodecErrorKind::TrailingBytes);
```

See `examples/basic_encoding.rs` and `examples/protocol_message.rs` for complete
examples.

## Feature Flags

| Flag | Default | Description |
|---|---:|---|
| `std` | yes | Enables `std::error::Error` for `CodecError` and enables `alloc` |
| `alloc` | no | Enables allocation-backed helpers and types such as `String` and `Vec<T>` |
| `primitives` | no | Adds optional implementations for supported `reliakit-primitives` types |

The `primitives` feature is optional and is not enabled by default.

## `no_std` Support

The crate supports `no_std` when default features are disabled.

Use `features = ["alloc"]` when you need `String`, `Vec<T>`, arrays decoded via
temporary vectors, or `encode_to_vec`.

Without `alloc`, the core traits, byte-slice reader, integer implementations,
`bool`, `Option`, `Result`, tuple implementations, array encoding, and `[u8; N]`
decoding remain available.

## Safety

This crate uses `#![forbid(unsafe_code)]`.

Decode is strict by default:

- invalid bool bytes fail,
- unknown tags should fail in manual enum implementations,
- invalid UTF-8 fails,
- exact slice decoding rejects trailing bytes,
- length prefixes are checked before conversion to `usize`.

Allocation-backed decoders allocate according to canonical `u32` length
prefixes. Applications with tighter memory limits should validate framing before
calling into allocation-backed decoders.

## MSRV

The minimum supported Rust version is Rust 1.85.

## Status

This is the initial `0.1` crate skeleton. The first version focuses on clear
traits, documented format rules, primitive implementations, examples, tests, and
optional integration with `reliakit-primitives`.

Future features may add more integrations, such as a separate
`reliakit-collections` feature for collection-specific types. Derive macros,
schema negotiation, floats, pointer-sized integers, and unordered maps are
intentionally out of scope for this version.

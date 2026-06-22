<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-derive

[![Crates.io](https://img.shields.io/crates/v/reliakit-derive.svg)](https://crates.io/crates/reliakit-derive)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-derive.svg)](https://crates.io/crates/reliakit-derive)
[![Docs.rs](https://docs.rs/reliakit-derive/badge.svg)](https://docs.rs/reliakit-derive)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-derive)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-derive)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)
[![zero dependencies](https://img.shields.io/badge/dependencies-0-success)](https://github.com/satyakwok/reliakit#footprint)

Derive macros for `reliakit` traits, implemented with only the standard library
`proc-macro` API and no third-party dependencies.

`reliakit-derive` generates the same trait implementations a handwritten one
would: it reads only the type name and field shape it needs, emits one
`encode`/`decode` call per field in declaration order, and rejects anything
outside its supported subset with a clear compile error rather than guessing.

## Introduction

Some `reliakit` traits are deliberately implemented by hand so field order and
validation stay visible in normal Rust. For plain data structs where the
implementation is purely mechanical, one call per field, in order, writing it
out by hand is repetitive without adding clarity. This crate provides an opt-in
derive for exactly those cases.

It does not parse the full Rust grammar. It reads the type name and its field or
variant shape, which is all the generated code needs, and stops with a
descriptive error on constructs it does not handle.

## What This Crate Does

This crate provides:

- `#[derive(CanonicalEncode)]` and `#[derive(CanonicalDecode)]` for the
  same-named `reliakit-codec` (binary) traits,
- `#[derive(JsonEncode)]` and `#[derive(JsonDecode)]` for the same-named
  `reliakit-json` traits,
- generated implementations identical to a handwritten one, each field encoded
  and decoded in declaration order,
- clear compile errors for unsupported inputs.

One `#[derive(...)]` line can target both formats at once, so the same struct
round-trips through canonical binary and through JSON.

## Supported Types

Structs:

- structs with named fields
- tuple structs
- unit structs

Enums:

- unit variants
- tuple variants
- struct variants

The enum or struct itself may be public or private. Outer attributes (such as
`#[doc = "..."]` or `#[allow(...)]`) on the type, its fields, or its variants are
ignored.

Unions, generic types (including generic enums and `where` clauses), enums with
explicit discriminants or a `#[repr(...)]`, and empty enums are rejected with a
compile error that names exactly what is unsupported.

The `JsonEncode`/`JsonDecode` derives currently cover structs only (named fields
become a JSON object in declaration order, a tuple struct an array, a unit
struct `null`); enums are rejected for now. The `CanonicalEncode`/
`CanonicalDecode` derives cover both structs and enums as listed above.

## When To Use

Use this derive when a type's encoding is the mechanical field-by-field default
and a handwritten implementation would add nothing:

- plain data structs at protocol or storage boundaries,
- fixtures and cache-key types,
- aggregates of already-`Canonical*` fields.

## When Not To Use

Write the implementation by hand when the encoding is not a straight
field-by-field pass: custom field order, enum tag schemes, versioning,
validation on decode, or skipped/derived fields. `reliakit-codec` is designed
for those to be explicit, and this derive does not try to express them.

## Installation

The generated code refers to `reliakit-codec`, so the two crates are used
together:

```toml
[dependencies]
reliakit-codec = "1.0"
reliakit-derive = "1.1"
```

```rust
use reliakit_codec::{decode_from_slice_exact, encode_to_vec};
use reliakit_derive::{CanonicalDecode, CanonicalEncode};

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Point {
    x: u16,
    y: u16,
}

let encoded = encode_to_vec(&Point { x: 10, y: 20 }).unwrap();
assert_eq!(encoded, [10, 0, 20, 0]);
assert_eq!(decode_from_slice_exact::<Point>(&encoded).unwrap(), Point { x: 10, y: 20 });
```

The bytes are exactly what the handwritten implementation in the `reliakit-codec`
documentation produces.

## Enums

An enum value is encoded as a variant tag followed by that variant's fields:

- the tag is the variant's **zero-based declaration index**, encoded as a
  little-endian `u32`;
- a unit variant encodes only the tag;
- a tuple variant encodes the tag, then its fields in declaration order;
- a struct variant encodes the tag, then its named fields in declaration order.

Decoding reads the `u32` tag first, then decodes the matching variant's payload.
An unknown tag is an `InvalidValue` codec error; a field decode error propagates
unchanged. Trailing bytes are the caller's concern; use
`decode_from_slice_exact` to reject them, not the generated impl's.

Tags follow declaration order, so **reordering variants is a wire-format
change**. Adding new variants at the end is backward compatible for decoding
older payloads.

Unit variants:

```rust
#[derive(CanonicalEncode, CanonicalDecode)]
enum Message {
    Ping, // tag 0u32 => 00 00 00 00
    Pong, // tag 1u32 => 01 00 00 00
}
```

Tuple variants:

```rust
#[derive(CanonicalEncode, CanonicalDecode)]
enum Command {
    SetPort(u16),   // tag 0, then the u16
    SetName(String), // tag 1, then the string
}
```

Struct variants:

```rust
#[derive(CanonicalEncode, CanonicalDecode)]
enum Event {
    UserCreated { id: u64, name: String }, // tag 0, then id, then name
    UserDeleted { id: u64 },               // tag 1, then id
}
```

A complete runnable example, all three variant kinds plus a nested derived
struct inside a struct variant, is in `examples/protocol.rs`:

```sh
cargo run -p reliakit-derive --example protocol
```

### Unsupported enum forms

These are rejected with a compile error rather than encoded with a guessed
meaning:

- **Explicit discriminants** (`A = 1`). The wire tag is always the declaration
  index; honoring a `= N` discriminant would silently change the encoding and
  make the format depend on values the derive does not read.
- **`#[repr(...)]` enums.** `repr` controls the in-memory discriminant type, not
  this crate's wire format (always a `u32` tag), so accepting it would be
  misleading.
- **Generic enums, `where` clauses, lifetimes, and const generics.** The derive
  reads only names and shapes, not bounds, so it cannot generate correct
  `where` clauses; these are rejected rather than half-supported.
- **Empty enums** (`enum Never {}`). There is no variant to encode or decode.

A tuple field whose type contains a top-level comma inside angle brackets (for
example `Result<A, B>`) is also not supported in tuple structs or tuple
variants, because the field counter splits on those commas; use a type alias.

## How It Works

The derive reads the derive input as a token stream and extracts two things: the
type name and the field shape (named field identifiers, tuple field count, or
unit), for enums, that shape is read per variant along with its declaration
index. For decoding it calls the trait through its fully qualified path, so it
never needs to parse or reproduce field *types*, only their names or positions.
The generated implementation is then built as source text and parsed back into
tokens. This keeps the crate small enough to need no parsing library.

## Feature Flags

This crate has no feature flags.

## Safety

This crate uses `#![forbid(unsafe_code)]`.

It runs only at compile time and performs no I/O. On unsupported input it emits a
`compile_error!` with a specific message instead of generating questionable code.

## MSRV

The minimum supported Rust version is Rust 1.85.

## License

MIT

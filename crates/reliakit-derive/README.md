<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-derive

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

Derive macros for `reliakit` traits, implemented with only the standard library
`proc-macro` API and no third-party dependencies.

`reliakit-derive` generates the same trait implementations a handwritten one
would: it reads only the type name and field shape it needs, emits one
`encode`/`decode` call per field in declaration order, and rejects anything
outside its supported subset with a clear compile error rather than guessing.

## Introduction

Some `reliakit` traits are deliberately implemented by hand so field order and
validation stay visible in normal Rust. For plain data structs where the
implementation is purely mechanical — one call per field, in order — writing it
out by hand is repetitive without adding clarity. This crate provides an opt-in
derive for exactly those cases.

It does not parse the full Rust grammar. It reads the struct name and field
shape, which is all the generated code needs, and stops with a descriptive error
on constructs it does not handle.

## What This Crate Does

This crate provides:

- `#[derive(CanonicalEncode)]` and `#[derive(CanonicalDecode)]` for the
  same-named `reliakit-codec` traits,
- generated implementations identical to a handwritten one — each field encoded
  and decoded in declaration order,
- clear compile errors for unsupported inputs.

## Supported Types

- structs with named fields
- tuple structs
- unit structs

Enums, unions, and generic types are rejected with a compile error. They may be
added later; until then the error names exactly what is unsupported.

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
together. `reliakit-derive` ships as part of the reliakit workspace; the
crates.io install snippet and live badges are added when it is published.

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

## How It Works

The derive reads the derive input as a token stream and extracts two things: the
type name and the field shape (named field identifiers, tuple field count, or
unit). For decoding it calls the trait through its fully qualified path, so it
never needs to parse or reproduce field *types* — only their names or positions.
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

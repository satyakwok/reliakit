<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-json

[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-json)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-json)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

A **strict, bounded, and deterministic** JSON library for Rust.

`reliakit-json` is built for systems that process **untrusted** JSON or need
**predictable** output — API payloads, protocol messages, config, audit logs,
and signed or hashed documents. It parses a strict subset of [RFC 8259], rejects
duplicate object keys, enforces explicit resource limits, preserves number
precision, reports errors with location and path, and serializes
deterministically. It has **no external dependencies**, forbids unsafe code, and
runs on `no_std` (with `alloc`).

It is **not** a `serde_json` replacement and does not try to be: no derive
macros, no schema validation, no JSON5, no comments, no trailing commas, no
lenient parsing, no SIMD. The goal is predictable behavior on hostile input, not
maximum throughput or convenience.

## What it guarantees

- **Strict parsing.** A conservative, I-JSON-oriented subset of RFC 8259.
- **Duplicate keys are rejected**, not silently resolved — and detection happens
  *after* escape decoding, so `"role"` and `"role"` collide.
- **Explicit resource limits** are part of the API. Untrusted parsing cannot run
  away on input size, nesting depth, string/number length, item counts, or total
  nodes.
- **Numbers keep their exact text.** No silent rounding; conversions to
  `i64`/`u64`/`f64` are explicit and fallible. `NaN`/`Infinity` are never
  accepted.
- **Actionable errors** with a stable [kind], byte offset, line, column, and the
  JSON path (`$.users[4].email`) being processed.
- **Deterministic output.** The compact writer preserves member order and exact
  number text; the same value always serializes to the same bytes.
- **`no_std + alloc`, zero dependencies, `#![forbid(unsafe_code)]`.**

## Rejected by default

Invalid UTF-8, a leading byte-order mark, comments, trailing commas, trailing
data, unescaped control characters, invalid escapes, malformed `\uXXXX`,
unpaired UTF-16 surrogates, duplicate keys, `NaN`/`Infinity`, leading `+`,
leading zeros, malformed numbers, and anything exceeding the configured limits.

## Installation

```toml
[dependencies]
reliakit-json = "0.1"
```

This crate is `no_std`-compatible (`default-features = false`); the default
`std` feature only adds `std::error::Error` implementations. It always requires
`alloc`.

## Usage

```rust
use reliakit_json::{parse_str, to_compact_string};

let value = parse_str(r#"{"name":"reliakit","port":8080}"#).unwrap();
let object = value.as_object().unwrap();
assert_eq!(object.get("name").unwrap().as_str(), Some("reliakit"));
assert_eq!(object.get("port").unwrap().as_number().unwrap().to_u64().unwrap(), 8080);

// Deterministic, member-order-preserving output.
assert_eq!(to_compact_string(&value), r#"{"name":"reliakit","port":8080}"#);
```

Parse untrusted input under explicit limits:

```rust
use reliakit_json::{parse_with_limits, JsonLimits};

let limits = JsonLimits::conservative().with_max_depth(16);
match parse_with_limits(untrusted_bytes, limits) {
    Ok(value) => { /* use it */ }
    Err(err)  => eprintln!("rejected: {err}"), // e.g. "limit exceeded: nesting depth at byte ..."
}
```

[`parse`] and [`parse_str`] apply [`JsonLimits::new`] (a conservative default)
automatically — there is no implicitly unlimited entry point.

## API

| Item | Purpose |
|---|---|
| `parse(&[u8])` / `parse_str(&str)` | Parse with default limits. |
| `parse_with_limits(&[u8], JsonLimits)` | Parse with an explicit limit profile. |
| `JsonValue` | Owned value: `Null`, `Bool`, `Number`, `String`, `Array`, `Object`. |
| `JsonNumber` | Precision-preserving number; `to_i64` / `to_u64` / `to_f64` / `try_from_f64`. |
| `JsonObject` | Unique-key, insertion-ordered map (`get`, `insert`, `iter`). |
| `JsonLimits` | `new` / `conservative` / `permissive` profiles + `with_*` builders. |
| `JsonError` / `JsonErrorKind` / `JsonLimitKind` / `JsonPath` | Located, classified errors. |
| `to_compact_string` / `to_compact_vec` | Deterministic compact serialization. |

## Numbers

JSON numbers are kept as their exact source text and never auto-converted to
`f64`. Equality is **structural** — `1.0`, `1`, and `1e0` are distinct values;
compare numerically by converting first. Conversions report whether they failed
on range, integer-ness, or finiteness.

## Limits

`JsonLimits` bounds *logical* decoded data (counts and byte lengths), not exact
allocator memory. Profiles: `conservative()` (small, low-trust payloads),
`new()` (the default), and `permissive()` (larger trusted documents) — all
explicit and finite. Tune individual fields with the `with_*` builders.

## When to use it

- Parsing untrusted JSON where predictable failure matters more than speed.
- Producing stable, deterministic JSON for hashing, signing, or content
  addressing.
- Embedded or `no_std` services that still need a careful JSON reader/writer.

## When not to use it

- You want struct (de)serialization or derive macros — use `serde` + `serde_json`.
- You want the fastest possible throughput — use a SIMD parser.
- You need JSON5, comments, or lenient parsing — this crate rejects them by design.

## Roadmap

RFC 8785 (JSON Canonicalization Scheme) output is planned, but will only be
exposed once it passes the reference vectors, idempotence, differential, and
fuzzing gates. It is not part of this release.

## Safety

`#![forbid(unsafe_code)]`. Parsing uses depth-bounded descent and saturating
arithmetic; there is no known input that causes a panic or unbounded work.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

[RFC 8259]: https://www.rfc-editor.org/rfc/rfc8259
[kind]: https://docs.rs/reliakit-json/latest/reliakit_json/enum.JsonErrorKind.html
[`parse`]: https://docs.rs/reliakit-json/latest/reliakit_json/fn.parse.html
[`parse_str`]: https://docs.rs/reliakit-json/latest/reliakit_json/fn.parse_str.html
[`JsonLimits::new`]: https://docs.rs/reliakit-json/latest/reliakit_json/struct.JsonLimits.html

# RFC: reliakit-json

Status: Accepted (Phase 1 implemented)

## Summary

`reliakit-json` is a strict, bounded, and deterministic JSON library for
reliability-sensitive Rust. It targets untrusted input and predictable output,
not maximum throughput or convenience.

Priorities: strict parsing, explicit resource limits, duplicate-key rejection,
actionable errors, deterministic serialization, zero external runtime
dependencies, `#![forbid(unsafe_code)]`, `no_std + alloc`, auditability.

Out of scope (initially): derive/proc macros, struct mapping, schema validation,
JSON5, comments, trailing commas, lenient parsing, streaming I/O, SIMD.

## Scope

Parse JSON from UTF-8 bytes/strings; an owned `JsonValue` model; precision-
preserving `JsonNumber`; strict duplicate-key rejection; explicit `JsonLimits`;
compact deterministic serialization; located, classified errors. RFC 8785 (JCS)
canonical output is planned but gated behind conformance + fuzzing and is not in
the initial release.

## Compliance

Strict I-JSON-oriented subset of RFC 8259. Rejects: invalid UTF-8, leading BOM,
comments, trailing commas, trailing data, unescaped control characters, invalid
escapes, malformed `\uXXXX`, unpaired surrogates, duplicate keys, `NaN`,
`Infinity`, leading `+`, leading zeros, malformed numbers, and over-limit input.
Whitespace is only `0x20 0x09 0x0A 0x0D`. Strings decode to Unicode scalar
values; `"a"` and `"a"` are equal.

## Allocation & features

Allocation is always required (owned strings/arrays/objects + dup detection), so
there is no `alloc` feature. `default = ["std"]`; `std` only adds
`std::error::Error`. The crate is usable as `no_std + alloc`.

## Duplicate-key policy

Rejected, never silently resolved. Equality is compared on the decoded key.
Objects preserve insertion order with guaranteed-unique keys
(`Vec<JsonMember>`). Detection during parse uses an `alloc::collections::BTreeSet`
of seen keys — `O(n log n)`, guaranteed, and immune to hash-flooding (chosen
over a hand-rolled hash set for that reason).

## Resource limits

`JsonLimits` is part of the primary API (no implicitly unlimited parser):
`max_input_bytes`, `max_depth`, `max_string_bytes`, `max_key_bytes`,
`max_number_bytes`, `max_array_items`, `max_object_members`, `max_total_nodes`,
`max_total_decoded_string_bytes`. Profiles: `new` (default/conservative),
`conservative` (tighter), `permissive` (larger, still finite). Limits bound
logical decoded data, not exact allocator memory.

## Parser architecture

Depth-bounded recursive descent: `max_depth` is checked **before** each descent,
so recursion never exceeds the configured depth and cannot exhaust the stack —
the same safety guarantee the RFC's "explicit stack" requirement targets. May be
refactored to an explicit stack during hardening without any API change.

## Numbers

`JsonNumber` preserves the validated source text (`Box<str>`). Equality is
**structural** (`1.0` != `1` != `1e0`); compare numerically by converting first.
`to_i64`/`to_u64` require integer syntax and report `OutOfRange`/`NotAnInteger`;
`to_f64` reports `NotFinite` on overflow; `try_from_f64` rejects `NaN`/infinity.

## Errors

`JsonError { kind, offset, line, column, path }`. `JsonErrorKind`,
`JsonLimitKind`, and `JsonNumberError` are `#[non_exhaustive]` (stable
machine-readable classifications that may gain variants). `JsonValue` and
`JsonPathSegment` are exhaustive (closed sets). `JsonPath` displays as
`$.users[4].email`.

## Serialization

Compact writer is deterministic (member order + exact number text preserved) and
infallible for in-memory output (`-> String`/`-> Vec<u8>`); `WriteFailure` is
reserved for a future `std::io` adapter. Canonical (JCS) serialization is
separate and gated behind the off-by-default `canonical` feature
(`to_canonical_string` / `to_canonical_vec`, fallible). Number formatting is
validated against the RFC 8785 examples and round-tripped over a large
randomized `f64` sample; key ordering (UTF-16), escaping, and idempotence are
covered by tests.

## Testing & hardening

Phase 1 ships with unit tests for every required acceptance/rejection, escape and
surrogate handling, all limit kinds, error locations/paths, golden compact bytes,
roundtrip, and an arbitrary-input panic smoke test. Hardening (Phase 3, done):
JSONTestSuite-style conformance tests and a dependency-free, deterministic in-test
fuzzer (hand-written PRNG) covering parser safety (no panic), compact roundtrip,
and canonical idempotence. Coverage-guided fuzzing is intentionally omitted — it
needs a third-party engine, and reliakit takes no third-party dependencies.

## Release gates

Initial parser release (§10.4): all documented behavior tested, clean package,
`cargo publish --dry-run` passes, `no_std + alloc` builds, no known panic or
unbounded-input path, duplicate keys rejected, limit boundaries tested, errors
locate failures, README matches implementation.

JCS may be claimed stable only after RFC 8785 vectors, number-formatting vectors,
Unicode ordering, idempotence, differential checks, and fuzzing all pass.

## Implementation phases

1. Parser foundation — errors, limits, tokenizer, strict parser, dup-key
   rejection, `JsonValue`, `JsonNumber`. **(done)**
2. Compact writer + exact-byte and roundtrip tests. **(done)**
3. Hardening — JSONTestSuite-style conformance tests, limit-boundary tests, and
   a dependency-free in-test fuzzer (hand-written PRNG) covering parse safety,
   compact round-trip, and canonical idempotence. **(done)** Coverage-guided
   fuzzing is intentionally omitted: it would require a third-party engine, and
   reliakit takes no third-party dependencies.
4. Canonicalization — RFC 8785 design, number formatting (ECMAScript), key
   ordering (UTF-16). **(done behind the `canonical` feature; validated against
   the RFC 8785 number examples and a large randomized `f64` round-trip sample.
   A dedicated fuzz target remains optional future hardening.)**
5. Optional integrations — Reliakit primitive conversions, validation, std I/O
   adapters, manual typed encode/decode traits.

## Decisions recorded during review

- Dup detection via `BTreeSet` (DoS-safe `O(n log n)`), not linear scan against a
  large member limit.
- Structural number equality (lossless, deterministic).
- `#[non_exhaustive]` on the error/limit enums from day one.
- Depth-bounded recursion (refactorable to an explicit stack later).
- Compact writer is infallible.

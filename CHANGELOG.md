# Changelog

All notable changes to this workspace are documented here.

This project follows normal Rust crate versioning. Crate releases may use a
workspace tag such as `vMAJOR.MINOR.PATCH` or a crate-specific tag such as
`CRATE-vMAJOR.MINOR.PATCH`.

## Unreleased

### Added

- Hardened the JSON parser with JSONTestSuite-style accept/reject conformance
  tests and a dependency-free, deterministic in-test fuzzer (hand-written PRNG)
  that asserts parsing arbitrary bytes never panics and that every parsed value
  survives a compact round-trip and canonical re-serialization unchanged.
- Added a manual publish workflow for publishing one selected crate to
  crates.io after tests, version checks, and `cargo publish --dry-run`.

### Changed

- Switched crates.io publishing to Trusted Publishing over GitHub Actions OIDC.
  The tag-triggered and manual publish workflows now mint a short-lived token at
  publish time instead of reading a stored API token, so no long-lived registry
  token is kept in repository secrets.
- Renamed the `basic` example target in `reliakit-backoff`, `reliakit-circuit`,
  and `reliakit-ratelimit` to `backoff_basic`, `circuit_basic`, and
  `ratelimit_basic` so the workspace can build all examples together without
  output-filename collisions.

## reliakit-json 0.2.0 - 2026-06-04

### Added

- Added RFC 8785 (JCS) canonical serialization behind the off-by-default
  `canonical` feature — `to_canonical_string` / `to_canonical_vec`, with UTF-16
  key ordering, minimal string escaping, and ECMAScript number formatting.
  Numbers are treated as IEEE-754 doubles; a non-representable magnitude returns
  the new `JsonErrorKind::NonFiniteNumber`. Number formatting is checked against
  the RFC 8785 examples and round-tripped over a large randomized `f64` sample.

### Changed

- `JsonError`'s `Display` no longer appends a source position for
  serialization-side errors (`NonFiniteNumber`, `WriteFailure`).
- Renamed the example target from `basic` to `json_basic` (run it with
  `cargo run -p reliakit-json --example json_basic`).

## reliakit-timeout 0.1.0 - 2026-06-04

Initial release.

### Added

- Added the `reliakit-timeout` crate: clock-agnostic deadlines and timeouts.
  - `Timeout` is a reusable budget; `Timeout::start(now)` pins it to a
    `Deadline`.
  - `Deadline` tracks a budget against a `u64` timeline (any monotonic unit) and
    answers `remaining`, `elapsed`, `is_expired`, `check` (an `Option`),
    `allows`, and `clamp` (cap a delay by the time left).
  - Every method is a saturating `const fn`; a backwards clock or an overflowing
    `start + budget` cannot panic. Pure `core`, no features, zero dependencies,
    `#![no_std]`, `#![forbid(unsafe_code)]`.

## reliakit-json 0.1.0 - 2026-06-04

Initial release.

### Added

- Added the `reliakit-json` crate: a strict, bounded, and deterministic JSON
  library for untrusted input and predictable output.
  - Parses a strict RFC 8259 subset and rejects invalid UTF-8, a leading BOM,
    comments, trailing commas, trailing data, unescaped control characters,
    invalid escapes, malformed `\uXXXX`, unpaired surrogates, duplicate keys,
    `NaN`/`Infinity`, leading `+`, leading zeros, and malformed numbers.
  - `JsonValue`, `JsonNumber` (precision-preserving), and `JsonObject`
    (unique-key, insertion-ordered).
  - `JsonLimits` with `new`/`conservative`/`permissive` profiles and `with_*`
    builders; `parse`, `parse_str`, and `parse_with_limits` apply explicit
    limits with no implicitly unlimited entry point.
  - `JsonError` carrying a stable kind, byte offset, line, column, and JSON
    path.
  - Deterministic compact serialization via `to_compact_string` /
    `to_compact_vec`.
  - `#![no_std]` + `alloc`, zero dependencies, `#![forbid(unsafe_code)]`. RFC
    8785 canonicalization is planned but not yet exposed.

## reliakit-ratelimit 0.1.0 - 2026-06-04

Initial release.

### Added

- Added the `reliakit-ratelimit` crate: a clock-agnostic token-bucket rate
  limiter.
  - `RateLimiter::new(capacity, refill_amount, refill_interval)`; `try_acquire`,
    `try_acquire_one`, `available`, and `retry_after`. The caller supplies the
    clock.
  - `#![no_std]`, zero dependencies, `#![forbid(unsafe_code)]`. Integer-only
    saturating arithmetic; no method panics, including on a non-monotonic clock.

## reliakit-circuit 0.1.0 - 2026-06-04

Initial release.

### Added

- Added the `reliakit-circuit` crate: a clock-agnostic circuit breaker.
  - `CircuitBreaker`, a `Copy` state machine over `Closed`/`Open`/`HalfOpen`
    with `failure_threshold`, `cooldown`, and `success_threshold`.
  - `allow(now)`, `on_success()`, `on_failure(now)`, `state()`, and explicit
    `trip(now)` / `reset()`. The caller supplies the clock.
  - `#![no_std]`, zero dependencies, `#![forbid(unsafe_code)]`. All arithmetic
    saturates; no method panics, including on a non-monotonic clock.

### Changed

- Rewrote the workspace `README.md` with "Why Reliakit?", "When should I use
  this?", and a before/after section; corrected the workspace layout, status,
  and roadmap to reflect all published crates.

## reliakit-primitives 0.4.0 - 2026-06-04

### Changed

- **Breaking:** marked `PrimitiveErrorKind` `#[non_exhaustive]` so future error
  categories can be added without a breaking change. Match on it with a `_` arm.

## reliakit-validate 0.3.0 - 2026-06-04

### Changed

- **Breaking:** marked `Violation` `#[non_exhaustive]` so future fields can be
  added without a breaking change. Construct it via `Violation::new` /
  `Violation::with_field` rather than a struct literal.

## reliakit-codec 0.2.0 - 2026-06-04

### Changed

- **Breaking:** marked `CodecErrorKind` `#[non_exhaustive]` so future error
  categories can be added without a breaking change. Match on it with a `_` arm.

## reliakit-backoff 0.1.0 - 2026-06-04

Initial release.

### Added

- Added the `reliakit-backoff` crate: clock-agnostic retry backoff policies.
  - `Backoff` with `constant`, `linear`, and `exponential` strategies, plus
    `with_max_delay` and `with_max_retries`.
  - `Backoff::delay(attempt)` returns the delay to wait before a zero-based
    retry, or `None` once the retry limit is reached. All arithmetic saturates
    and the computation runs in bounded time.
  - `Backoff::delays()` iterator over successive delays.
  - `full_jitter` and `equal_jitter` pure helpers that take caller-supplied
    randomness (no RNG dependency).
  - `#![no_std]`, zero dependencies, `#![forbid(unsafe_code)]`.

## reliakit-primitives 0.3.0 - 2026-06-03

### Changed

- **Breaking:** made the `alloc` feature behavior match its documentation by
  gating the allocation-backed owned types (`Slug`, `Email`, `HttpUrl`,
  `HexString`, `NonEmptyStr`, `BoundedStr`, `NonEmptyVec`, `SemVer`) and the
  `String` equality impls on `Uuid`/`HumanDuration` behind the `alloc` feature.
  `std` now implies `alloc`. Building with `--no-default-features` now exposes
  only the allocation-free primitives (numeric types, `Uuid`, `HumanDuration`,
  and the error types), changing the public API available under
  `--no-default-features`.
- Clarified `BoundedStr::new` docs to state that, when `MIN > 0`, empty or
  whitespace-only input is rejected with `Empty`.

## reliakit-collections 0.2.0 - 2026-06-03

### Changed

- **Breaking:** gated `BoundedVec` behind the `alloc` feature (it is backed by
  `Vec<T>`), and `std` now implies `alloc`. Building with `--no-default-features`
  now exposes only the error types (`CollectionError`, `CollectionResult`);
  `BoundedVec` requires `alloc` (enabled by default via `std`). This changes the
  public API available under `--no-default-features`.

## reliakit-validate 0.2.0 - 2026-06-03

### Changed

- **Breaking:** gated `ValidationError` and `ValidateResult` behind the `alloc`
  feature (they collect `Violation`s in a `Vec`); `std` now implies `alloc`.
  The `Validate` trait, `Valid<T>`, and `Violation` remain available without
  `alloc`. Building with `--no-default-features` no longer exposes
  `ValidationError`/`ValidateResult`, changing the public API available under
  `--no-default-features`.

## reliakit-codec 0.1.0 - 2026-06-03

Initial release.

### Added

- Added the `reliakit-codec` crate with:
  - `CanonicalEncode` and `CanonicalDecode` traits for deterministic binary
    encoding and strict decoding.
  - `EncodeSink` and `DecodeSource` sink/source traits that work without
    `std::io`.
  - `SliceReader` for decoding from in-memory byte slices.
  - `CodecError` and `CodecErrorKind` for stable, programmatic error handling.
  - `encode_to_vec`, `decode_from_slice`, and `decode_from_slice_exact`
    helpers.
  - Canonical implementations for integers, `bool`, `str`/`String`, `Vec<T>`,
    `Option<T>`, `Result<T, E>`, fixed-size arrays, and tuples up to arity 4.
  - Optional `reliakit-primitives` integration behind the `primitives` feature.
  - `no_std` support with an optional `alloc` feature, and
    `#![forbid(unsafe_code)]`.

## reliakit-primitives 0.2.5 - 2026-06-03

### Added

- `reliakit-primitives`: added `PrimitiveErrorKind` and
  `PrimitiveError::kind()` for stable programmatic error matching without
  depending on display text.
- `reliakit-primitives`: added `SemVer::cmp_precedence()` for SemVer
  precedence comparisons that intentionally ignore build metadata.
- `reliakit-primitives`: added the `service_config` example, demonstrating
  `reliakit-primitives`, `reliakit-secret`, and `reliakit-validate` working
  together. The library's runtime dependencies remain zero; the secret and
  validate crates are dev-dependencies used only by the example.

### Changed

- `reliakit-primitives`: made `SemVer`'s `Ord` implementation consistent with
  `Eq` by using build metadata as a final total-ordering tie-breaker.
- `reliakit-primitives`: made string-backed text wrappers more consistent by
  adding missing string conversion and deref implementations.
- `reliakit-primitives`: removed an avoidable allocation from HTTP URL scheme
  validation.

## reliakit-primitives 0.2.4 - 2026-06-02

### Changed

- Release-automation and version-tagging verification. No functional or API
  changes from `0.2.3`.

## reliakit-primitives 0.2.3 - 2026-06-02

### Fixed

- Fixed silent `u64` truncation in `HumanDuration::parse` for very large hour
  values. Inputs such as `"18446744073709551615h"` previously returned `Ok`
  with a wrong `Duration`; they now return `Err(Invalid)`.
- Removed unreachable dead-code guard in `HumanDuration::parse`.
- Fixed potential `usize` overflow in `BoundedVec::push` error payload when
  `MAX == usize::MAX`.

## reliakit-validate 0.1.0 - 2026-06-02

### Added

- Added the `reliakit-validate` crate with:
  - `Validate` — trait for types that can validate themselves.
  - `Valid<T>` — zero-cost wrapper carrying proof of successful validation.
  - `ValidationError` — error type collecting one or more `Violation`s.
  - `Violation` — single failed constraint with optional field name.
  - `ValidateResult<T>` — `Result<T, ValidationError>` type alias.

## reliakit-collections 0.1.0 - 2026-06-02

### Added

- Added the `reliakit-collections` crate with:
  - `BoundedVec<T, MIN, MAX>` — owned `Vec<T>` constrained to hold between
    `MIN` and `MAX` elements. `push` and `pop` return errors instead of
    panicking when bounds would be violated.
  - `CollectionError` — error type with `TooFew`, `TooMany`, and
    `InvalidBounds` variants.

## reliakit-primitives 0.2.2 - 2026-06-02

### Added

- Added `FromStr` implementations for string-backed and parsed primitives:
  - `NonEmptyStr`
  - `BoundedStr<MIN, MAX>`
  - `Slug`
  - `Email`
  - `HttpUrl`
  - `HexString`
  - `SemVer`
  - `Uuid`
  - `HumanDuration`
- Added direct comparisons against `str`, `&str`, `String`, and `&String` for
  the same primitive types.

### Changed

- Enabled `missing_docs` warnings for `reliakit-primitives`.
- Made additional infallible or validation-only constructors `const fn` where
  supported by the current MSRV.

## reliakit-secret 0.1.0 - 2026-06-02

### Added

- Added the `reliakit-secret` crate with:
  - `Secret<T>`
  - `SecretString`
  - `ExposeSecret<T>`
  - `ExposeSecretMut<T>`
- Added a `secret_basic` example.

## reliakit-primitives 0.2.1 - 2026-06-02

### Changed

- Polished the `reliakit-primitives` crate README for crates.io.
- Clarified the crate purpose: typed validated values for library APIs and input
  boundaries.
- Added examples for text primitives, structured values, and error handling.
- Moved the `primitives_basic` example into the crate package so it is included
  in published crates.io sources.

### Fixed

- Rejected empty email domain labels such as `user@example..com`.
- Fixed the root README Star History embed.

## reliakit-primitives 0.2.0 - 2026-06-02

### Added

- Added additional primitive types to `reliakit-primitives`:
  - `Slug`
  - `Email`
  - `HttpUrl`
  - `HexString`
  - `PercentageF64`
  - `PositiveInt`
  - `PositiveFloat`
  - `NonEmptyVec<T>`
  - `SemVer`
  - `Uuid`
  - `HumanDuration`
- Added Codecov configuration and per-crate coverage flagging for
  `reliakit-primitives`.

### Fixed

- Tightened validation for email whitespace, HTTP URL whitespace, SemVer
  identifiers, and human duration unit ordering.
- Fixed SemVer pre-release ordering and numeric identifier comparison.
- Rejected HTTP(S) URLs with missing hosts such as `https:///path`.

## reliakit-primitives 0.1.0 - 2026-06-01

### Added

- Initialized the Reliakit Rust workspace.
- Added `reliakit-primitives` with:
  - `NonEmptyStr`
  - `BoundedStr<MIN, MAX>`
  - `Percent`
  - `Port`
  - `ByteSize`
- Added CI, docs, coverage, audit, and publish dry-run workflows.

# Changelog

All notable changes to this workspace are documented here.

This project follows normal Rust crate versioning. Crate releases may use a
workspace tag such as `vMAJOR.MINOR.PATCH` or a crate-specific tag such as
`CRATE-vMAJOR.MINOR.PATCH`.

## Unreleased

### Added

- `reliakit-backoff`: `Backoff::fibonacci(base)`: a Fibonacci backoff schedule
  where attempt `n` waits `base * fib(n)` (`1, 1, 2, 3, 5, 8, ...`), growth
  between linear and exponential. Saturating and bounded like the other
  strategies (ships as `reliakit-backoff` 1.1.0).
- `reliakit-primitives`: `Probability`, a finite `f64` in `0.0..=1.0` for rates,
  weights, and sampling. Has `new`/`TryFrom<f64>`/`Display` and a `complement`
  helper. Rejects NaN, infinity, and out-of-range values.
- `reliakit-primitives`: `PositiveDuration`, a `Duration` that rejects
  `Duration::ZERO`. Has `new`/`TryFrom<Duration>`/`Display`/`AsRef<Duration>`.
- `reliakit-primitives`: `InlineStr<MIN, MAX>`, a stack-allocated bounded string
  stored in a `[u8; MAX]` buffer with no heap allocation, so it works in `no_std`
  without `alloc`. Bounds the byte length (not the character count like
  `BoundedStr`) and is `Copy`. (All ship as `reliakit-primitives` 1.1.0.)
- `reliakit-bulkhead`: `Bulkhead::try_acquire_observed` (and a `_one` variant)
  plus an `Admission` enum: an opt-in hook that reports admitted-vs-rejected and
  the free permits left after each decision, for metrics, leaving the existing
  API unchanged. Allocation-free (ships as `reliakit-bulkhead` 1.1.0).
- `reliakit-retry`: `RetryPolicy::with_budget(Duration)` (and `budget()`): an
  optional cap on the cumulative backoff delay between attempts. The drivers stop
  with `Exhausted` once the next wait would exceed it, independent of
  `max_attempts`. It bounds the backoff the policy computes, not wall-clock time
  (the crate reads no clock). Existing drivers are unchanged when no budget is
  set. Allocation-free, `no_std` (ships as `reliakit-retry` 1.1.0).
- `reliakit-derive`: `#[reliakit(rename = "...")]` and `#[reliakit(skip)]` field
  attributes for the JSON and CSV derives: `rename` sets the object key / CSV
  header, and `skip` omits the field on encode and fills `Default::default()` on
  decode. The canonical codec ignores both (positional, names irrelevant), so its
  wire format is unchanged. Parsed by hand, no new dependencies (ships as
  `reliakit-derive` 1.1.0).
- `reliakit`: an `intake_pipeline` example carrying one batch end to end: typed
  CSV in, per-field validation, a bounded buffer that sheds when full, canonical
  encoding for the wire, a resilient flush behind retry/backoff/circuit, and a
  closing health report.

## reliakit-derive 1.0.2 - 2026-06-19

### Added

- `#[reliakit(crate = "...")]` container attribute: point the derives at an umbrella
  crate that re-exports `reliakit-csv`/`reliakit-codec`/`reliakit-json` as its `csv`/
  `codec`/`json` submodules. The generated code then resolves through it (e.g.
  `::reliakit::csv`) instead of the standalone crate paths.

### Fixed

- Deriving for a crate that depends only on the umbrella `reliakit` crate (not the
  individual `reliakit-*` crates) no longer fails to compile with
  `error[E0433]: use of undeclared crate or module reliakit_csv`. The generated paths
  hard-coded the standalone crates, which only resolve as direct dependencies; add
  `#[reliakit(crate = "reliakit")]` to resolve through the umbrella instead. Without the
  attribute, behavior is unchanged (standalone paths), so this is backward compatible.

## reliakit-derive 1.0.1 - 2026-06-19

### Fixed

- Tuple structs and tuple enum variants with a field whose type holds a generic
  argument list (`Result<T, E>`, `HashMap<K, V>`, and the like) are now counted
  correctly. Angle brackets are punctuation, not a token group, so the comma
  inside `<...>` was read as a field separator and the generated code referenced
  a field that does not exist.

## 1.0.0 - 2026-06-15

All crates are promoted to **1.0.0**: their public APIs are now stable and follow
standard semver, so a future breaking change will require a 2.0. The freeze
fixes from the 0.x line, `reliakit-primitives` `PercentFloat`, private
`reliakit-json` `JsonLimits`, `#[non_exhaustive]` on the `reliakit-health` and
`reliakit-decide` data types, and the documented-stable codec/JSON/CSV wire
formats, are part of this frozen surface.

### Changed

- **Breaking:** `reliakit-health`'s `Check` is now `#[non_exhaustive]`. Build it
  with `Check::new(...)` and the builder methods (`optional`, `with_detail`)
  instead of a struct literal, so fields can be added later without breaking.
- Switched crates.io publishing to Trusted Publishing over GitHub Actions OIDC,
  so no long-lived registry token is kept in repository secrets.
- Pointed the workspace `homepage` at `https://satyakwok.dev/projects/reliakit`.

### Added

- `reliakit-retry`: `retry_with_sleep_observed` and `retry_async_observed`, which
  take an `on_retry(attempt, delay, &error)` hook called before each wait for
  logging or metrics. The existing `retry`/`retry_with_sleep`/`retry_async` are
  unchanged and delegate to these with a no-op hook. Allocation-free, `no_std`,
  zero-dependency.
- `reliakit-csv`: `CsvField` impls for `char` and for `IpAddr`/`SocketAddr`
  (including the `V4`/`V6` forms).
- A `typed_csv` example in the `reliakit` umbrella; a zero-dependency benchmark
  harness (`benches`, see `BENCHMARKS.md`); a `deny.toml` and a zero-dependency
  CI gate; bare-metal `no_std` checks for every `no_std` crate; and `RELEASING.md`.

## reliakit-primitives 0.5.0 - 2026-06-14

### Changed

- **Breaking:** renamed `PercentageF64` to `PercentFloat`, for consistency with
  `Percent` (same root word) and `PositiveFloat` (same `Float` suffix for the
  floating-point variant).

## reliakit-json 0.3.0 - 2026-06-14

### Changed

- **Breaking:** `JsonLimits` fields are now private, with a getter and a `with_*`
  builder method for every limit (matching `CsvLimits`). This lets new limits be
  added without breaking callers; construct a profile with
  `new`/`conservative`/`permissive` and adjust it with the `with_*` methods.

## reliakit-codec 0.3.0 - 2026-06-14

### Changed

- The optional `primitives` integration now targets `reliakit-primitives` 0.5, so
  its `CanonicalEncode`/`CanonicalDecode` impls apply to the 0.5 primitive types.

## reliakit-health 0.2.0 - 2026-06-14

### Changed

- **Breaking:** marked `Component` and `Summary` `#[non_exhaustive]` so fields can
  be added later without breaking callers. Build them via their constructors.

## reliakit-decide 0.2.0 - 2026-06-14

### Changed

- **Breaking:** marked `Consideration`, `Action`, `Decision`, `Contribution`, and
  `Explanation` `#[non_exhaustive]` so fields can be added later without breaking
  callers. Build them via their constructors (`Action::new`,
  `Consideration::new`/`labeled`, …) rather than struct literals.

## reliakit 0.2.0 - 2026-06-14

### Changed

- Updated the re-exported sub-crates to their frozen 1.0-track releases:
  `reliakit-primitives` 0.5, `reliakit-json` 0.3, `reliakit-codec` 0.3,
  `reliakit-health` 0.2, and `reliakit-decide` 0.2. The breaking changes in those
  crates surface through the umbrella's re-exports.

## reliakit 0.1.5 - 2026-06-09

### Added

- Exposed `reliakit-csv` and `reliakit-retry` through the umbrella behind `csv`
  and `retry` features (both also included in `full`), with `std`/`alloc`
  forwarding (`retry` is pure `core`, so it has no `alloc` wiring).

## reliakit-retry 0.1.0 - 2026-06-09

### Added

- Initial release. Small, runtime-agnostic retry helpers built on
  `reliakit-backoff`: `RetryPolicy` (total-attempt limit plus a backoff
  schedule), and `retry` / `retry_with_sleep` / `retry_async` drivers with a
  `should_retry` error classifier. The crate never sleeps or spawns internally;
  `retry` runs attempts back-to-back, `retry_with_sleep` calls a caller-provided
  sleeper, and `retry_async` awaits a caller-provided sleep future using only
  `core::future::Future` (no Tokio/async-std/`futures`, no forced runtime).
  `RetryError::Exhausted` carries the attempt count and last error with no
  allocation and no `Error` bound. Pure `core`, `no_std`, zero-dependency.

## reliakit-derive 0.1.3 - 2026-06-10

### Added

- `#[derive(CsvEncode, CsvDecode)]` for the `reliakit-csv` traits. A struct with
  named fields becomes a CSV row, one column per field in declaration order,
  with the field names as the header. Only structs with named fields are
  supported (CSV columns need names); tuple structs, unit structs, and enums are
  rejected. Decoding is strict. Requires `reliakit-csv` 0.1.1 for the generated
  code's hidden re-exports. Existing codec and JSON derives are unchanged.

## reliakit-csv 0.1.1 - 2026-06-10

### Added

- A `#[doc(hidden)]` `__private` module that re-exports `alloc`'s `Vec` and
  `String`. It exists only so the `CsvEncode`/`CsvDecode` derives in
  `reliakit-derive` can name those types in generated code on `no_std`. It is
  not part of the public API and may change at any time. No existing API
  changes.

## reliakit-csv 0.1.0 - 2026-06-09

### Added

- Initial release. Strict, bounded, and deterministic CSV (a subset of
  RFC 4180): `read_str`/`read_str_with_limits` parse text into rectangular
  records and reject malformed quoting, bare carriage returns, and ragged rows;
  `CsvWriter` writes deterministically, quoting a field only when required and
  terminating every record with `\r\n`; `CsvLimits` bounds input size, record
  and field counts, and field length. A typed layer (`CsvField`, `CsvEncode`,
  `CsvDecode`, `to_csv_string`/`from_csv_str` with headerless variants) maps
  records to and from your own types. `no_std` with `alloc`, zero-dependency.

## reliakit-health 0.1.0 - 2026-06-08

### Added

- Initial release. Health status types and a criticality-aware aggregator for
  service health checks, probes, and status pages: `Health`
  (`Healthy`/`Degraded`/`Unhealthy`, ordered by severity), `Criticality`
  (`Optional` failures cap at `Degraded`), an allocation-free `Check` +
  `aggregate`, and an owned `HealthReport` with `overall`/`summary`/`reasons`.
  Reports only, never acts. `no_std`-friendly, zero-dependency.

## reliakit 0.1.4 - 2026-06-08

### Added

- Exposed `reliakit-health` through the umbrella behind a `health` feature (also
  included in `full`).

## reliakit-primitives 0.4.3 - 2026-06-08

### Added

- `Cidr`, an allocation-free IPv4/IPv6 network type (`address/prefix`) with
  prefix-length validation, `contains` membership testing, and a masked
  `network()`. Works in `no_std` without `alloc`.
- `Base32`, a standard (RFC 4648) base32 format-validated string.

## reliakit-collections 0.3.2 - 2026-06-08

### Added

- `BoundedMap<K, V, MIN, MAX>` and `BoundedSet<T, MIN, MAX>`: insertion-ordered,
  vec-backed types with unique keys/elements and an enforced count range, using
  linear lookup so they stay deterministic and dependency-free.
- `CollectionError::Duplicate` variant (the enum is `#[non_exhaustive]`).

## reliakit-secret 0.1.3 - 2026-06-08

### Added

- `Secret::ct_eq`, a best-effort constant-time byte comparison (available for
  byte-viewable secrets such as `String`, `Vec<u8>`, `&[u8]`, `[u8; N]`) so a
  presented value can be checked against a stored secret without leaking it
  through timing. Documented redacting a secret field inside a larger
  `#[derive(Debug)]` struct.

## reliakit-validate 0.3.3 - 2026-06-08

### Added

- Documented a real-world pattern, turning collected `Violation`s into an
  API-style `(field, message)` error list, and how to pair the crate with
  `reliakit-primitives` for ready-made typed fields. Docs only; no API change.

## reliakit-bulkhead 0.1.0 - 2026-06-08

### Added

- Initial release. A clock-agnostic concurrency limiter (counting semaphore)
  that caps in-flight operations and sheds load when full: `try_acquire` /
  `release` with saturating, panic-free integer math, capacity clamped to at
  least one, and the invariant `in_flight <= capacity` held on every public
  path. Pure `core`, `no_std`, zero-dependency, no unsafe.

## reliakit 0.1.3 - 2026-06-08

### Added

- Exposed `reliakit-bulkhead` through the umbrella behind a `bulkhead` feature
  (also included in `full`).

## reliakit 0.1.2 - 2026-06-08

### Changed

- The `reliakit` crate-level usage example is now a tested doctest rather than an
  `ignore`d block.

## reliakit 0.1.1 - 2026-06-08

### Added

- Examples reachable through the `reliakit` umbrella: `resilient_client`,
  `config_check`, and `typed_json`.

## reliakit 0.1.0 - 2026-06-08

### Added

- Initial release of the `reliakit` umbrella crate, which re-exports every
  `reliakit-*` building block behind a per-crate feature flag with `std`/`alloc`
  forwarding, a `core` feature that enables clock-aware methods, optional
  cross-crate integration features, and a `full` feature. It contains no logic of
  its own.

## reliakit-decide 0.1.0 - 2026-06-07

### Added

- Initial release of `reliakit-decide`: a deterministic, zero-dependency
  utility-based decision engine. `Score` (fixed-point), `Curve`, `Consideration`,
  and `Action` (product-veto utility) feed a `Reasoner` that can `decide`,
  `rank`, `decide_weighted` (roulette selection with a caller-supplied RNG),
  `decide_above` (abstain below a threshold), and `explain` a choice.
  `Action::gate` makes decisions constraint-aware with no dependency, and
  `Policy` tunes per-key weights from feedback (bounded integer moving average).
  `no_std` + `alloc`, `#![forbid(unsafe_code)]`, zero third-party dependencies.

## reliakit-codec 0.2.2 - 2026-06-05

### Changed

- Documentation only: the README now points to `reliakit-derive` for
  `#[derive(CanonicalEncode, CanonicalDecode)]` support. No API or behavior
  changes.

## reliakit-circuit 0.2.3 - 2026-06-05

### Changed

- Documentation only: add Feature Flags, `no_std`, and Status sections to the
  README. No API or behavior changes.

## reliakit-backoff 0.1.4 - 2026-06-08

### Added

- `decorrelated_jitter(base, prev, cap, rand)`: a pure jitter helper that
  returns a delay uniformly in `base ..= prev * 3`, capped at `cap`. The caller
  feeds each result back in as `prev`, so the delay walks up and down between
  retries while the function stays dependency-free and saturating.

## reliakit-backoff 0.1.3 - 2026-06-05

### Changed

- Documentation only: add Feature Flags, `no_std`, and Status sections to the
  README. No API or behavior changes.

## reliakit-ratelimit 0.1.3 - 2026-06-05

### Changed

- Documentation only: add Feature Flags, `no_std`, and Status sections to the
  README. No API or behavior changes.

## reliakit-timeout 0.1.3 - 2026-06-05

### Changed

- Documentation only: add Feature Flags, `no_std`, and Status sections to the
  README. No API or behavior changes.

## reliakit-core 0.1.2 - 2026-06-05

### Changed

- Documentation only: add a Status section to the README. No API or behavior
  changes.

## reliakit-json 0.2.5 - 2026-06-05

### Added

- Typed JSON encoding and decoding. `JsonEncode` turns a value into a
  deterministic `JsonValue` (and `to_json_string`/`to_json_vec` into compact
  text/bytes); `JsonDecode` reads it back strictly, with `from_json_str` parsing
  and decoding in one step. Implementations cover the integer types, `bool`,
  `String`/`str`, `Option<T>`, `Vec<T>`, and slices. Adds
  `JsonDecodeError`/`JsonDecodeErrorKind`, `JsonFromStrError`, and a
  `JsonValue::array` constructor. Strict, zero-dependency, `no_std` + `alloc`.

## reliakit-derive 0.1.2 - 2026-06-05

### Added

- `#[derive(JsonEncode, JsonDecode)]` for the `reliakit-json` traits. A struct
  with named fields becomes a JSON object in declaration order, a tuple struct
  becomes an array, and a unit struct becomes `null`; enums are rejected for
  now. Existing codec derives are unchanged.

## reliakit-backoff 0.1.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-circuit 0.2.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-collections 0.3.1 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-core 0.1.1 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-json 0.2.4 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-primitives 0.4.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-ratelimit 0.1.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-secret 0.1.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest, and add the workspace logo
  header and crates.io downloads badge to the README. Documentation and
  packaging only; no API or behavior changes.

## reliakit-timeout 0.1.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-validate 0.3.2 - 2026-06-05

### Changed

- Add the `documentation` link to the crate manifest. Packaging metadata only;
  no API or behavior changes.

## reliakit-derive 0.1.1 - 2026-06-05

### Added

- Enum derive support for `CanonicalEncode`/`CanonicalDecode`, covering unit,
  tuple, and struct variants. Each variant is tagged by its zero-based
  declaration index as a little-endian `u32`, followed by the variant's fields
  in declaration order; decoding reads the tag first and an unknown tag is a
  clear decode error. Unsupported enum forms, explicit discriminants,
  `#[repr(...)]`, generic enums, and empty enums, are rejected with descriptive
  compile errors. Existing struct derive behavior is unchanged.

## reliakit-derive 0.1.0 - 2026-06-05

### Added

- Initial release of `reliakit-derive`: `#[derive(CanonicalEncode)]` and
  `#[derive(CanonicalDecode)]` for the `reliakit-codec` traits. The generated
  code matches a handwritten implementation, one encode/decode call per field
  in declaration order, and supports named, tuple, and unit structs; enums,
  unions, and generics are rejected with a descriptive compile error. Built on
  only the standard library proc-macro API with no third-party dependencies.

## reliakit-json 0.2.3 - 2026-06-04

### Added

- Optional `validate` feature (implies `primitives`): `JsonForm` collects
  field-extraction failures from a `JsonObject` into a single
  `reliakit_validate::ValidationError` instead of stopping at the first one.
  `str_field::<T>(name)` extracts a string primitive and, on failure, records a
  named `Violation`; `finish()` returns `Ok(())` or every collected violation at
  once. Pulls in `reliakit-validate` only (`no_std` + `alloc`, zero third-party
  dependencies).

## reliakit-json 0.2.2 - 2026-06-04

### Added

- Optional `primitives` feature: typed extraction into `reliakit-primitives`
  constrained types. `JsonObject::get_str_as::<T>(key)` and
  `JsonValue::str_as::<T>()` run a JSON string through `T`'s `TryFrom<&str>`
  validating constructor (`Email`, `HttpUrl`, `Hostname`, `Base64`, …). Failures
  return the new `JsonExtractError`/`JsonExtractErrorKind` (`Missing`,
  `WrongType`, `Invalid(PrimitiveError)`) carrying the offending field's
  `JsonPath`. The feature pulls in `reliakit-primitives` only (`no_std` +
  `alloc`, zero third-party dependencies).

## reliakit-json 0.2.1 - 2026-06-04

### Added

- Hardened the parser with JSONTestSuite-style accept/reject conformance tests
  and a dependency-free, deterministic in-test fuzzer (hand-written PRNG) that
  asserts parsing arbitrary bytes never panics and that every parsed value
  survives a compact round-trip and canonical re-serialization unchanged.

### Changed

- Now ships its own `LICENSE` file in the published package, so the MIT license
  text travels with the crate on crates.io rather than only the SPDX identifier.

## reliakit-codec 0.2.1 - 2026-06-04

### Changed

- Now ships its own `LICENSE` file in the published package, so the MIT license
  text travels with the crate on crates.io rather than only the SPDX identifier.

## reliakit-backoff 0.1.1 - 2026-06-04

### Changed

- Renamed the `basic` example target to `backoff_basic` so the workspace can
  build all examples together without output-filename collisions.
- Now ships its own `LICENSE` file in the published package, so the MIT license
  text travels with the crate on crates.io rather than only the SPDX identifier.

## reliakit-secret 0.1.1 - 2026-06-04

### Changed

- Now ships its own `LICENSE` file in the published package, so the MIT license
  text travels with the crate on crates.io rather than only the SPDX identifier.

## reliakit-primitives 0.4.1 - 2026-06-04

### Added

- Four constrained types. `Base64` (RFC 4648 standard alphabet with correct
  padding), `Identifier` (ASCII identifier: a letter or `_`, then letters,
  digits, or `_`), and `Hostname` (RFC 1123 labels) are `alloc`-backed and join
  the other text primitives. `MacAddress` (six octets, `aa:bb:cc:dd:ee:ff` or
  `-` form) is allocation-free `no_std`, like `Uuid`. Each follows the existing
  constructor/`Display`/`AsRef`/`Deref`/`TryFrom`/`FromStr` conventions and uses
  `PrimitiveError`.

## reliakit-validate 0.3.1 - 2026-06-04

### Added

- `ValidationError` gained builder ergonomics for multi-field checks:
  `require(condition, violation)` and `require_field(condition, field, message)`
  record a violation only when a condition fails, and `finish()` turns the
  accumulated violations into a `ValidateResult` (`Ok(())` when empty), removing
  the easy-to-forget final emptiness check. Added `FromIterator<Violation>` and
  `Extend<Violation>` so violations can be collected from an iterator.

## reliakit-circuit 0.2.1 - 2026-06-04

### Added

- Optional `core` feature: `allow_now`, `on_failure_now`, and `trip_now` on
  `CircuitBreaker` and `RollingBreaker`, which read the current time from a
  `reliakit_core::Clock` instead of taking an explicit `now: u64`. The feature
  pulls in `reliakit-core` only (`no_std`, zero third-party dependencies); the
  `now: u64` methods are unchanged.

## reliakit-ratelimit 0.1.1 - 2026-06-04

### Added

- Optional `core` feature: `available_now`, `try_acquire_now`,
  `try_acquire_one_now`, and `retry_after_now` on `RateLimiter`, which read the
  current time from a `reliakit_core::Clock` instead of taking an explicit
  `now: u64`. The feature pulls in `reliakit-core` only (`no_std`, zero
  third-party dependencies); the `now: u64` methods are unchanged.

## reliakit-timeout 0.1.1 - 2026-06-04

### Added

- Optional `core` feature: `start_now` on `Timeout` and `elapsed_now`,
  `remaining_now`, `is_expired_now`, `check_now`, `allows_now`, and `clamp_now`
  on `Deadline`, which read the current time from a `reliakit_core::Clock`
  instead of taking an explicit `now: u64`. The feature pulls in `reliakit-core`
  only (`no_std`, zero third-party dependencies); the `now: u64` methods are
  unchanged.

## reliakit-circuit 0.2.0 - 2026-06-04

### Added

- `RollingBreaker<const WINDOW>`, a circuit breaker that trips on the failure
  *rate* (N failures within the last `WINDOW` calls) rather than on consecutive
  failures. The window is stored inline (`[bool; WINDOW]`, zero allocation,
  `no_std`); cooldown and half-open recovery match `CircuitBreaker`, which is
  unchanged.

## reliakit-collections 0.3.0 - 2026-06-04

### Added

- `RingBuffer<T>`, a fixed-capacity circular buffer that overwrites the oldest
  element when full (a rolling window whose `push` never fails, evicting and
  returning the oldest element instead). Behind the `alloc` feature.
- `CollectionError::ZeroCapacity` for a zero-capacity request.

## reliakit-core 0.1.0 - 2026-06-04

### Added

- Initial release. Shared building blocks for the workspace: a `Clock` trait
  (`now(&self) -> u64`) with `ManualClock` (settable, `no_std`, for deterministic
  tests) and `MonotonicClock` (milliseconds since creation, backed by
  `std::time::Instant`). Zero dependencies, `#![forbid(unsafe_code)]`, `no_std`
  for the trait and `ManualClock`.

## reliakit-json 0.2.0 - 2026-06-04

### Added

- Added RFC 8785 (JCS) canonical serialization behind the off-by-default
  `canonical` feature, `to_canonical_string` / `to_canonical_vec`, with UTF-16
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
  - `Validate`: trait for types that can validate themselves.
  - `Valid<T>`: zero-cost wrapper carrying proof of successful validation.
  - `ValidationError`: error type collecting one or more `Violation`s.
  - `Violation`: single failed constraint with optional field name.
  - `ValidateResult<T>`: `Result<T, ValidationError>` type alias.

## reliakit-collections 0.1.0 - 2026-06-02

### Added

- Added the `reliakit-collections` crate with:
  - `BoundedVec<T, MIN, MAX>`: owned `Vec<T>` constrained to hold between
    `MIN` and `MAX` elements. `push` and `pop` return errors instead of
    panicking when bounds would be violated.
  - `CollectionError`: error type with `TooFew`, `TooMany`, and
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

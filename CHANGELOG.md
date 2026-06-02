# Changelog

All notable changes to this workspace are documented here.

This project follows normal Rust crate versioning. Crate releases may use a
workspace tag such as `vMAJOR.MINOR.PATCH` or a crate-specific tag such as
`CRATE-vMAJOR.MINOR.PATCH`.

## Unreleased

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

# Reliakit

Reusable Rust reliability primitives and utility crates.

[![Crates.io](https://img.shields.io/crates/v/reliakit.svg)](https://crates.io/crates/reliakit)
[![Docs.rs](https://docs.rs/reliakit/badge.svg)](https://docs.rs/reliakit)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Reliakit is a Rust workspace for reusable reliability primitives and utility crates. It focuses on small, composable building blocks for writing correct, safe, and reliable Rust libraries and applications.

The workspace is intended to provide type-safe primitives, validation-oriented utility types, secret-safe wrappers, and small foundational crates with minimal dependencies and optional feature flags where appropriate. Each crate is designed to be usable independently.

Reliakit is experimental. The initial workspace crates are planned, but this repository does not yet contain crate manifests or public source APIs. This README describes the intended workspace shape without claiming that unpublished crates or APIs are available.

## Crates

### `reliakit-core`

Planned.

Shared core types, traits, and errors used across Reliakit crates.

Possible contents:

- common error types,
- result aliases,
- validation traits,
- shared utility traits.

No public API is documented here yet because the crate source has not landed.

### `reliakit-primitives`

Planned.

Reusable type-safe primitives for representing constrained values.

Possible contents:

- `NonEmptyStr`,
- `BoundedStr`,
- `BoundedVec`,
- `Percent`,
- `Port`,
- `ByteSize`.

These names describe the intended direction. They are not documented as implemented APIs until the crate source exists.

### `reliakit-secret`

Planned.

Secret-safe wrappers for values that should not leak through `Debug`, logs, reports, or diagnostic output.

Possible contents:

- `Secret<T>`,
- `Redacted<T>`,
- explicit secret exposure APIs,
- redaction helpers.

These APIs are planned. The final names and behavior may change before the first release.

### `reliakit-collections`

Planned.

Bounded and reliability-oriented collection utilities.

### `reliakit-validate`

Planned.

General validation helpers and traits.

### `reliakit-derive`

Planned.

Derive macros for validation and constrained types.

## Design Goals

- Reusable library primitives.
- Clear type semantics.
- Minimal dependencies.
- No hidden runtime.
- No framework lock-in.
- Optional feature flags.
- `no_std` support where practical.
- Safe diagnostic output.
- Stable, documented APIs.
- Composable crates.

## Non-Goals

Reliakit is not:

- an async runtime,
- a web framework,
- an ORM,
- a logging framework,
- a replacement for `serde`,
- a replacement for `tokio`,
- a replacement for `clap`,
- a replacement for `anyhow`,
- a replacement for `thiserror`,
- a replacement for `hashbrown`,
- a replacement for `syn`.

Reliakit is intended to provide focused primitives and utility crates, not replace mature ecosystem foundations.

## Installation

The crates are not published yet. Once the root crate is available on crates.io:

```toml
[dependencies]
reliakit = "0.1"
```

Individual crates will be usable independently:

```toml
[dependencies]
reliakit-core = "0.1"
reliakit-primitives = "0.1"
reliakit-secret = "0.1"
```

Before publication, Git dependencies may be used after the corresponding crate directories and manifests exist:

```toml
[dependencies]
reliakit-secret = { git = "https://github.com/satyakwok/reliakit", package = "reliakit-secret" }
```

## Workspace Layout

Planned workspace layout:

```text
reliakit/
|-- crates/
|   |-- reliakit-core/
|   |-- reliakit-primitives/
|   |-- reliakit-secret/
|   |-- reliakit-collections/
|   |-- reliakit-validate/
|   `-- reliakit-derive/
|-- examples/
|-- Cargo.toml
|-- README.md
`-- LICENSE
```

## Repository Status

This repository currently contains the README and license. The Rust workspace, crate manifests, source files, crate-level READMEs, examples, CI, and tests still need to be added.

When crate directories are introduced, each crate should include its own README describing only the APIs implemented by that crate.

## Roadmap

Initial workspace:

- `reliakit-core`
- `reliakit-primitives`
- `reliakit-secret`

Later:

- `reliakit-collections`
- `reliakit-validate`
- `reliakit-derive`
- crate-level examples
- CI and documentation checks
- crate-level README files derived from implemented APIs

## License

Licensed under the MIT License. See `LICENSE`.

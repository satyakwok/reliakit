# Reliakit

Reliable Rust utilities for CLIs, backend services, bots, and infrastructure tooling.

[![Crates.io](https://img.shields.io/crates/v/reliakit.svg)](https://crates.io/crates/reliakit)
[![Docs.rs](https://docs.rs/reliakit/badge.svg)](https://docs.rs/reliakit)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)

Reliakit provides practical Rust utilities for reliability-oriented applications. It is intended for CLIs, backend services, bots, RPC tools, automation scripts, and infrastructure tooling that need predictable startup checks, safe diagnostics, retries, and structured reports.

Reliakit is not a framework. Each crate should remain focused, composable, and usable independently, so applications can adopt only the pieces they need.

The project is experimental. APIs may change while the crate boundaries, examples, and tests are being refined.

## What Reliakit Provides

- Environment validation for startup configuration.
- Secret redaction for logs, errors, config dumps, and diagnostic output.
- Retry and backoff helpers for fallible sync and async operations.
- Health checks for HTTP, TCP, and JSON-RPC endpoints. Planned.
- CLI-friendly errors for startup and operational failures. Planned.
- Machine-readable reports for CLIs, CI, and monitoring tools. Planned.

## Crates

### `reliakit-redact`

Redacts secrets from logs, errors, config dumps, and diagnostic output.

Examples of secrets include API keys, bearer tokens, private keys, passwords, and database URLs.

### `reliakit-env`

Validates required and optional environment variables with clear startup errors.

Planned features include required variables, optional defaults, typed parsing, and secret-aware output.

### `reliakit-retry`

Retries fallible sync and async operations.

Planned features include max attempts, fixed delay, exponential backoff, jitter, and retryable error classification.

### `reliakit-health`

Planned crate for HTTP, TCP, and JSON-RPC health checks.

### `reliakit-report`

Planned crate for terminal and JSON reports for CLIs, CI, and monitoring tools.

## Examples

The APIs below show the intended direction. They may change while Reliakit is experimental.

### Secret Redaction

```rust
use reliakit_redact::redact;

let input = "DATABASE_URL=postgres://user:password@localhost:5432/app";
let output = redact(input);

println!("{output}");
```

### Environment Validation

```rust
use reliakit_env::EnvSchema;

let env = EnvSchema::new()
    .required("DATABASE_URL")
    .required("RPC_URL")
    .optional("PORT", "3000")
    .secret("PRIVATE_KEY")
    .load()?;
```

### Retry Helper

```rust
use reliakit_retry::Retry;

let result = Retry::new()
    .attempts(5)
    .run(|| async { call_external_service().await })
    .await?;
```

## Installation

After publishing, the workspace meta crate can be installed from crates.io:

```toml
[dependencies]
reliakit = "0.1"
```

Individual crates can be used directly:

```toml
[dependencies]
reliakit-redact = "0.1"
reliakit-env = "0.1"
reliakit-retry = "0.1"
```

Before crates are published, use a Git dependency:

```toml
[dependencies]
reliakit-redact = { git = "https://github.com/satyakwok/reliakit", package = "reliakit-redact" }
```

## Workspace Layout

Planned workspace layout:

```text
reliakit/
├── crates/
│   ├── reliakit-redact/
│   ├── reliakit-env/
│   ├── reliakit-retry/
│   ├── reliakit-health/
│   └── reliakit-report/
├── examples/
├── Cargo.toml
├── README.md
└── LICENSE
```

## Design Goals

- Focused APIs.
- Minimal dependencies.
- Useful defaults.
- Clear error messages.
- Safe diagnostic output.
- Machine-readable reports.
- Composable crates.

## Non-Goals

Reliakit is not:

- an async runtime,
- a web framework,
- an ORM,
- a logging framework,
- a full observability platform,
- a replacement for Tokio, Serde, Clap, Tracing, Anyhow, or Thiserror.

## Status

Reliakit is experimental. APIs may change before stable releases. Initial focus is correctness, tests, examples, and clear crate boundaries.

## Roadmap

v0.1:

- `reliakit-redact`
- `reliakit-env`
- `reliakit-retry`

Later:

- `reliakit-health`
- `reliakit-report`
- JSON output
- CI-friendly reports
- Examples for bots, CLIs, and infrastructure tools

## License

Licensed under the MIT License. See `LICENSE`.

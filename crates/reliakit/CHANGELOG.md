# Changelog

All notable changes to this crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-06-08

### Changed

- The crate-level usage example is now a tested doctest instead of an `ignore`d
  block, so it is verified rather than only illustrative.

## [0.1.1] - 2026-06-08

### Added

- Examples reachable through the umbrella: `resilient_client` (a deadline, rate
  limiter, circuit breaker, and backoff guarding one call), `config_check`
  (typed config parsing that reports every field error at once, secret redacted),
  and `typed_json` (parse untrusted JSON strictly, then lift fields into
  validated primitives).

## [0.1.0] - 2026-06-08

### Added

- Initial release of the `reliakit` umbrella crate. Re-exports the individual
  `reliakit-*` building blocks behind per-crate feature flags, with `std`/`alloc`
  forwarding, a `core` feature that enables clock-aware methods, optional
  cross-crate integration features, and a `full` feature. The crate contains no
  logic of its own.

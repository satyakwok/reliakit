# Changelog

All notable changes to this crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-06-08

### Added

- Initial release of the `reliakit` umbrella crate. Re-exports the individual
  `reliakit-*` building blocks behind per-crate feature flags, with `std`/`alloc`
  forwarding, a `core` feature that enables clock-aware methods, optional
  cross-crate integration features, and a `full` feature. The crate contains no
  logic of its own.

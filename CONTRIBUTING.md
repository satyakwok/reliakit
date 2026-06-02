# Contributing to Reliakit

Contributions are welcome. Please read this document before opening an issue or pull request.

## Before You Start

For non-trivial changes, open an issue first to discuss the direction. This avoids wasted effort if the change does not align with the project's goals.

For small fixes (typos, doc corrections, obvious bugs), a pull request without a prior issue is fine.

## Development Setup

```sh
git clone https://github.com/satyakwok/reliakit
cd reliakit
cargo build --workspace --all-features
cargo test --workspace --all-features
```

Rust stable is required. No additional tooling is needed beyond the standard Cargo toolchain.

## Before Submitting

Run these before opening a pull request:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```

All four must pass cleanly.

## Guidelines

- Keep each crate minimal and focused on its stated purpose.
- Add tests for any new public API surface. Coverage should not regress below 93%.
- Document public items. Every `pub fn`, `pub struct`, and `pub enum` needs at least a one-line doc comment.
- Avoid adding dependencies unless strictly necessary.
- `unsafe` code is not accepted in any crate that forbids it. Check the crate's `Cargo.toml` or `lib.rs` for `#![forbid(unsafe_code)]`.
- Keep commit messages concise and in the imperative mood: `Add TryFrom<u32> for Port`, not `Added` or `Adding`.

## Crate Scope

Each crate in this workspace has a narrow scope:

| Crate | Scope |
|---|---|
| `reliakit-primitives` | Owned wrapper types for constrained values. No dependencies. |

Proposed additions to a crate should fit within its stated scope. If they do not, consider proposing a new crate.

## Reporting Bugs

Open an issue with:

- A minimal reproduction (a code snippet or failing test is ideal).
- The Rust version (`rustc --version`).
- The expected versus actual behavior.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

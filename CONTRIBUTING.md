# Contributing to Reliakit

Thanks for your interest. Reliakit is a set of small, explicit,
reliability-oriented building blocks for Rust, and contributions that keep it
that way are very welcome.

## Project principles

A few constraints hold across every crate, and changes are reviewed against them:

- **Zero third-party dependencies.** Crates depend only on the standard library
  and, optionally, on other `reliakit-*` crates — no `syn`, `quote`,
  proc-macros, or serde-family crates. A CI job fails the build if any crate
  gains a third-party dependency of any kind.
- **No unsafe.** Every crate is `#![forbid(unsafe_code)]`.
- **`no_std`-friendly.** Most crates build without `std` (some need `alloc`).
  Reach for `core`/`alloc` before `std`.
- **Small, clear surface.** Validate at construction, reject invalid input, use
  exact error variants, and keep public APIs minimal.
- **MSRV is Rust 1.85.** Don't use language or library features newer than that
  in crate code.

## Where to start

Issues labelled [`good first issue`](https://github.com/satyakwok/reliakit/labels/good%20first%20issue)
are scoped for newcomers; [`help wanted`](https://github.com/satyakwok/reliakit/labels/help%20wanted)
ones need a bit more design judgment. Both usually list the files to touch and an
acceptance checklist, so they're a good way in.

For non-trivial changes, open an issue first so the direction can be discussed —
it avoids wasted effort. Small fixes (typos, doc corrections, obvious bugs) can
go straight to a pull request.

## Development setup

```sh
git clone https://github.com/satyakwok/reliakit
cd reliakit
cargo build --workspace --all-features
cargo test --workspace --all-features
```

Rust stable is enough; no tooling is needed beyond the standard Cargo toolchain.

## Before submitting

Run these and make sure they pass cleanly:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```

If you touched a `no_std` crate, also confirm it builds without `std` (and on a
bare-metal target if you have one installed):

```sh
cargo check -p <crate> --no-default-features
cargo check -p <crate> --no-default-features --features alloc   # if it has an alloc feature
```

## Guidelines

- Keep each crate focused on its stated purpose. If a change doesn't fit,
  consider proposing a new crate instead — see the
  [crate overview](./README.md#crate-overview) for what each one covers.
- Add tests for any new public behavior. Tests must be deterministic — no
  reliance on wall-clock time or iteration order. Codec and serialization
  changes need exact-output (byte- or text-level) tests.
- Don't let coverage regress; most crates target 90% (a couple are higher).
- Document public items. Every `pub fn`, `struct`, and `enum` gets at least a
  one-line doc comment, with a runnable example where it helps.
- Mark new public error enums and plain-data structs `#[non_exhaustive]` so they
  can gain variants or fields later without breaking callers.
- Don't change or rename public APIs without a reason and a heads-up in the
  issue first.

### Serialization and wire formats

Typed serialization in Reliakit is a set of per-format trait pairs —
`CanonicalEncode`/`CanonicalDecode` for binary, `JsonEncode`/`JsonDecode` for
JSON, `CsvEncode`/`CsvDecode` for CSV — not a single serde-style abstraction. A
new format follows the same pattern.

Wire formats are permanent once published. If you change what a format reads or
writes, pin the exact bytes/text with tests and call out the compatibility
impact in your pull request.

## Commit and pull request style

- **Sign your commits.** Every commit in a pull request must be cryptographically
  signed and show as *Verified* on GitHub — CI rejects unsigned commits. Set up
  GPG or SSH commit signing once and enable it (`git config commit.gpgsign true`);
  see [GitHub's guide](https://docs.github.com/authentication/managing-commit-signature-verification).
- Write plain, human commit messages in the imperative mood: `Add TryFrom<u32>
  for Port`, not `Added` or `Adding`.
- Keep each pull request focused on one logical change; avoid unrelated
  reformatting.
- Say what changed and why, and link the issue it addresses.

## Reporting bugs

Open an issue with:

- A minimal reproduction (a snippet or failing test is ideal).
- Your Rust version (`rustc --version`).
- The expected versus actual behavior.

## License

By contributing, you agree that your contributions will be licensed under the
MIT License.

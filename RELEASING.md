# Releasing

How to cut a release of a Reliakit crate. Each crate is versioned and published
independently.

## Principles

- **Zero third-party dependencies.** Every crate depends only on the standard
  library and on other `reliakit-*` crates. CI enforces this (the "Zero
  dependencies" job); never add an external crate to pass a release.
- **Semantic versioning.** Crates are pre-1.0, so a breaking change bumps the
  minor (`0.2.x` → `0.3.0`) and a backward-compatible change bumps the patch
  (`0.2.0` → `0.2.1`).
- Publishing uses **crates.io Trusted Publishing over GitHub Actions OIDC**;
  there is no API token stored in the repository.

## Per-crate release

1. **Pick the version.** Decide the new version from the changes since the last
   release (breaking → minor, additive/fix → patch).
2. **Bump `crates/<crate>/Cargo.toml`** to that version.
3. **Update `CHANGELOG.md`.** Move the crate's entries from `Unreleased` into a
   dated `## <crate> <version> - YYYY-MM-DD` section.
4. **Update install snippets** that mention the crate (its own `README.md` and
   the root `README.md`) to the new version.
5. **Verify locally:**
   ```sh
   cargo fmt --all --check
   cargo test --workspace --all-features --locked
   cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
   cargo doc --workspace --all-features --no-deps --locked
   cargo publish -p <crate> --dry-run --locked
   ```
6. **Open a PR, let CI pass, and merge** to `main`.
7. **Tag and push** from the merge commit:
   ```sh
   git tag -a <crate>-v<version> -m "<crate> <version>"
   git push origin <crate>-v<version>
   ```
   The tag triggers `.github/workflows/publish.yml`, which authenticates via
   OIDC and runs `cargo publish`. The job is idempotent: if the version is
   already on crates.io it skips.
8. **Create the GitHub release** for the tag with notes drawn from the changelog.

## Dependency order

Publish leaf crates before dependents, because `cargo publish` verifies that a
dependency's version already exists on crates.io. The only intra-workspace build
dependency today is **`reliakit-codec` → `reliakit-primitives`** (optional), so
publish `reliakit-primitives` first if both are being released. All other
intra-workspace references are dev-dependencies and do not constrain order.

## First publish of a new crate

Trusted Publishing cannot mint a token for a crate that does not exist on
crates.io yet, and a Trusted Publisher cannot be configured until the crate
exists. Bootstrap the first publish with a crates.io API token instead:

```sh
cargo publish -p <crate> --locked   # uses ~/.cargo/credentials.toml
```

Then add the crate's Trusted Publisher on crates.io so subsequent releases go
through OIDC.

## crates.io Trusted Publisher settings

Configured per crate at `https://crates.io/crates/<crate>/settings`:

| Field | Value |
|---|---|
| Repository owner | `satyakwok` |
| Repository name | `reliakit` |
| Workflow filename | `publish.yml` |
| Environment | `crates-io-publish` |

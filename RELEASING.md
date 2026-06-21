# Releasing

How to cut a release of a Reliakit crate. Each crate is versioned and published
independently.

## Principles

- **Zero third-party dependencies.** Every crate depends only on the standard
  library and on other `reliakit-*` crates. CI enforces this (the "Zero
  dependencies" job); never add an external crate to pass a release.
- **Semantic versioning.** Crates are 1.0, so a backward-compatible addition
  bumps the minor (`1.0.x` → `1.1.0`), a fix bumps the patch (`1.0.0` →
  `1.0.1`), and a breaking change bumps the major (`1.x` → `2.0.0`) and is
  avoided.
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
7. **Tag and push** a signed tag from the merge commit:
   ```sh
   git tag -s <crate>-v<version> -m "<crate> <version>"
   git push origin <crate>-v<version>
   ```
   The tag fires two workflows automatically:
   - `.github/workflows/publish.yml` authenticates via OIDC and runs
     `cargo publish`. It is idempotent: if the version is already on crates.io
     it skips.
   - `.github/workflows/release.yml` creates the GitHub release, taking the body
     from the matching `## <crate> <version>` section of `CHANGELOG.md` (falling
     back to a link if no section is found).
8. **Confirm both succeeded:** the crate shows the new version on crates.io and
   the GitHub release exists. There is no manual release step.

## Dependency order

Publish a crate's dependencies before the crate itself, because `cargo publish`
checks that each dependency requirement is already satisfiable on crates.io. The
runtime intra-workspace dependencies are:

- `reliakit-retry` → `reliakit-backoff`
- `reliakit-codec` → `reliakit-primitives` (optional `primitives` feature)
- `reliakit-json` → `reliakit-primitives`, `reliakit-validate` (optional)
- `reliakit-circuit` / `reliakit-ratelimit` / `reliakit-timeout` →
  `reliakit-core` (optional `core` feature)
- the `reliakit` umbrella re-exports every crate

All other intra-workspace references are dev-dependencies and do not constrain
order. Because every dependent declares a `"1"` requirement, an additive release
of a dependency does not force its dependents to re-release: they pick up the new
minor through `cargo update`. Only a breaking (major) release forces the
dependents to update and move together.

## Releasing several crates at once

For a coordinated release (a milestone), bump and changelog every crate in one
PR, merge it, then push the tags **leaf-first, in batches of at most three**.
Pushing more than three tags in a single `git push` does not trigger the Publish
workflow, so split them and wait for each batch to finish before the next:

```sh
git tag -s reliakit-primitives-v1.1.0 -m "reliakit-primitives 1.1.0"
git tag -s reliakit-backoff-v1.1.0    -m "reliakit-backoff 1.1.0"
git tag -s reliakit-bulkhead-v1.1.0   -m "reliakit-bulkhead 1.1.0"
git push origin reliakit-primitives-v1.1.0 reliakit-backoff-v1.1.0 reliakit-bulkhead-v1.1.0
# wait for these three to publish, then push the next batch
```

Only the changed crates are released; an unchanged crate, including the
`reliakit` umbrella, needs no re-release.

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

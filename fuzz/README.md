# reliakit fuzzing

Coverage-guided fuzz targets for the untrusted-input paths, built with
[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer). This crate
is detached from the workspace and is never published.

## Targets

| Target | Checks |
|---|---|
| `json_parse` | `reliakit_json::parse` never panics, hangs, or runs unbounded on arbitrary bytes. |
| `json_roundtrip` | A parsed value survives `to_compact_string` → reparse unchanged, and the compact form is stable. |
| `json_canonical` | RFC 8785 canonicalization never panics and is idempotent. |
| `codec_decode` | `reliakit_codec` decoding of arbitrary bytes into representative types never panics. |

## Running

Requires a nightly toolchain and `cargo-fuzz` (`cargo install cargo-fuzz`):

```sh
cargo +nightly fuzz run json_parse
cargo +nightly fuzz run json_roundtrip -- -max_total_time=60
cargo +nightly fuzz run json_canonical
cargo +nightly fuzz run codec_decode
```

Build all targets without running:

```sh
cargo +nightly fuzz build
```

Any crash inputs are written under `fuzz/artifacts/` and should be added to the
crate test suites as regression cases.

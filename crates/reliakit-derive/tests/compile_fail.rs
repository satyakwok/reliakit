//! Behavior coverage for the derive's rejection paths.
//!
//! Each unsupported construct must be rejected with a clear `compile_error!`,
//! not encoded with a guessed meaning. Covering a `compile_error!` branch needs
//! a build that *fails*, which a normal passing test cannot produce, so this
//! test drives a real `cargo build` of a generated fixture crate in a temporary
//! directory and asserts the build fails with each expected message.
//!
//! It uses only the standard library: no `trybuild` or other dev-dependency.
//! When the sibling `reliakit-codec` crate is not reachable by path (e.g. a
//! packaged copy of this crate), the test skips itself.

use std::path::PathBuf;
use std::process::Command;

/// One bad construct and the substring its compile error must contain. Each
/// uses a distinct type name so the only errors are the derive's own.
const CASES: &[(&str, &str)] = &[
    (
        "#[derive(CanonicalEncode)] enum Disc { A = 1, B = 2 }",
        "does not support explicit enum discriminants",
    ),
    (
        "#[derive(CanonicalEncode)] #[repr(u8)] enum Repr { A, B }",
        "does not support `#[repr(...)]` on enums",
    ),
    (
        "#[derive(CanonicalEncode)] enum GenericEnum<T> { A(T) }",
        "does not support generic types yet",
    ),
    (
        "#[derive(CanonicalEncode)] struct GenericStruct<T> { a: T }",
        "does not support generic types yet",
    ),
    (
        "#[derive(CanonicalEncode)] enum Empty {}",
        "cannot derive for an empty enum",
    ),
    (
        "#[derive(CanonicalEncode)] union Onion { a: u32 }",
        "does not support unions",
    ),
];

#[test]
fn rejection_paths_fail_to_compile_with_clear_messages() {
    let derive_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let codec_dir = derive_dir.parent().unwrap().join("reliakit-codec");
    if !codec_dir.join("Cargo.toml").exists() {
        eprintln!("skipping: sibling reliakit-codec not reachable by path");
        return;
    }

    let tmp = std::env::temp_dir().join(format!(
        "reliakit-derive-compile-fail-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let src = tmp.join("src");
    std::fs::create_dir_all(&src).expect("create temp src dir");

    // One fixture file holds every bad construct; the compiler reports all of
    // their `compile_error!`s in a single build.
    let mut body = String::from("use reliakit_derive::CanonicalEncode;\n");
    for (snippet, _) in CASES {
        body.push_str(snippet);
        body.push('\n');
    }
    body.push_str("fn main() {}\n");

    let cargo_toml = format!(
        "[package]\n\
         name = \"reliakit-derive-compile-fail\"\n\
         version = \"0.0.0\"\n\
         edition = \"2021\"\n\
         publish = false\n\n\
         [workspace]\n\n\
         [dependencies]\n\
         reliakit-derive = {{ path = {derive:?} }}\n\
         reliakit-codec = {{ path = {codec:?}, default-features = false, features = [\"alloc\"] }}\n",
        derive = derive_dir,
        codec = codec_dir,
    );
    std::fs::write(tmp.join("Cargo.toml"), cargo_toml).expect("write fixture manifest");
    std::fs::write(src.join("main.rs"), body).expect("write fixture source");

    let output = Command::new(env!("CARGO"))
        .arg("build")
        .arg("--manifest-path")
        .arg(tmp.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(tmp.join("target"))
        // Isolate from any coverage instrumentation in the outer environment so
        // the inner build behaves the same under `cargo test` and `cargo llvm-cov`.
        .env_remove("RUSTFLAGS")
        .env_remove("RUSTDOCFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("spawn cargo build");

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&tmp);

    assert!(
        !output.status.success(),
        "fixture must fail to compile; stderr:\n{stderr}"
    );
    for (snippet, needle) in CASES {
        assert!(
            stderr.contains(needle),
            "expected a compile error containing {needle:?} for `{snippet}`;\nstderr:\n{stderr}"
        );
    }
}

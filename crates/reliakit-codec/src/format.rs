//! Canonical binary format rules.
//!
//! Version 0.1 intentionally defines one binary representation per supported
//! value:
//!
//! - integers are fixed-width little-endian,
//! - `bool` is exactly `0x00` or `0x01`,
//! - strings are UTF-8 bytes prefixed by a `u32` little-endian byte length,
//! - vectors are prefixed by a `u32` little-endian item count,
//! - `Option<T>` and `Result<T, E>` use one-byte tags,
//! - fixed arrays and tuples encode fields in declaration order.
//!
//! Floats, pointer-sized integers, hash maps, unordered maps, schema
//! negotiation, and non-canonical alternatives are not part of this initial
//! format. They are omitted because their representation or ordering can be
//! platform-dependent, ambiguous, or outside this crate's first-version scope.
//!
//! Generic fixed-array decoding (`[T; N]`) requires the `alloc` feature in this
//! version because the crate forbids unsafe code and Rust 1.85 does not provide
//! a stable fallible array initializer. In no-alloc builds, `[u8; N]` decoding is
//! available because it can be filled directly from the source without heap
//! allocation.

/// Tag used for `false`.
pub const BOOL_FALSE: u8 = 0x00;
/// Tag used for `true`.
pub const BOOL_TRUE: u8 = 0x01;
/// Tag used for `None`.
pub const OPTION_NONE: u8 = 0x00;
/// Tag used for `Some`.
pub const OPTION_SOME: u8 = 0x01;
/// Tag used for `Ok`.
pub const RESULT_OK: u8 = 0x00;
/// Tag used for `Err`.
pub const RESULT_ERR: u8 = 0x01;

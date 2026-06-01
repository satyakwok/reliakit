//! Type-safe primitives for constrained values.
//!
//! `reliakit-primitives` provides small owned wrapper types for values that
//! should satisfy common constraints before they move through an application or
//! library boundary.
//!
//! The crate has no dependencies and forbids unsafe code.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod bounded;
pub mod error;
pub mod non_empty;
pub mod numeric;

pub use bounded::BoundedStr;
pub use error::{PrimitiveError, PrimitiveResult};
pub use non_empty::NonEmptyStr;
pub use numeric::{ByteSize, Percent, Port};

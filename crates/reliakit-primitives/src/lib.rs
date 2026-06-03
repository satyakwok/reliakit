//! Type-safe primitives for constrained and reliability-oriented values.
//!
//! `reliakit-primitives` provides small owned wrapper types for values that
//! should satisfy common constraints before they move through an application or
//! library boundary.
//!
//! The crate is useful when a public API should accept a validated value instead
//! of an unchecked `String`, integer, or float. Constructors validate once at the
//! boundary and then carry the invariant in the type.
//!
//! The crate has no dependencies and forbids unsafe code.
//!
//! # Examples
//!
//! Basic service configuration:
//!
//! ```
//! use reliakit_primitives::{NonEmptyStr, Port};
//!
//! fn configure(name: NonEmptyStr, port: Port) {
//!     assert_eq!(name.as_str(), "api");
//!     assert_eq!(port.get(), 8080);
//! }
//!
//! let name = NonEmptyStr::new("api")?;
//! let port = Port::new(8080)?;
//!
//! configure(name, port);
//! # Ok::<(), reliakit_primitives::PrimitiveError>(())
//! ```
//!
//! Text and structured values:
//!
//! ```
//! use reliakit_primitives::{Email, HumanDuration, HttpUrl, SemVer};
//!
//! let contact = Email::new("ops@example.com")?;
//! let healthcheck = HttpUrl::new("https://example.com/health")?;
//! let version = SemVer::parse("1.2.3-beta.1")?;
//! let timeout = HumanDuration::parse("30s")?;
//!
//! assert_eq!(contact.domain(), "example.com");
//! assert!(healthcheck.is_https());
//! assert!(version.is_pre_release());
//! assert_eq!(timeout.as_secs(), 30);
//! # Ok::<(), reliakit_primitives::PrimitiveError>(())
//! ```
//!
//! # Feature flags
//!
//! - `std` enables `std::error::Error` for [`PrimitiveError`] and is enabled by
//!   default.
//! - `alloc` enables allocation-backed types without `std`.
//!
//! # `no_std`
//!
//! The crate supports `no_std` when default features are disabled and `alloc` is
//! enabled for string-backed and vector-backed primitives.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

/// Bounded string primitive.
pub mod bounded;
/// Non-empty vector primitive.
pub mod collections;
/// Human-readable duration primitive.
pub mod duration;
/// Shared primitive error type.
pub mod error;
/// Non-empty string primitive.
pub mod non_empty;
/// Numeric primitives.
pub mod numeric;
/// Semantic version primitive.
pub mod semver;
/// Text validation primitives.
pub mod text;
/// UUID primitive.
pub mod uuid;

pub use bounded::BoundedStr;
pub use collections::NonEmptyVec;
pub use duration::HumanDuration;
pub use error::{PrimitiveError, PrimitiveErrorKind, PrimitiveResult};
pub use non_empty::NonEmptyStr;
pub use numeric::{ByteSize, Percent, PercentageF64, Port, PositiveFloat, PositiveInt};
pub use semver::SemVer;
pub use text::{Email, HexString, HttpUrl, Slug};
pub use uuid::Uuid;

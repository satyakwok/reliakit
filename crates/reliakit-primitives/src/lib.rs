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
//! - `std` (default) enables `std::error::Error` for [`PrimitiveError`] and
//!   implies `alloc`.
//! - `alloc` enables the allocation-backed owned types and collection helpers
//!   listed below.
//!
//! # `no_std`
//!
//! The crate supports `no_std`. Building with `--no-default-features` (no `std`,
//! no `alloc`) provides the allocation-free primitives:
//!
//! - numeric: [`Percent`], [`PercentFloat`], [`Port`], [`PositiveInt`],
//!   [`PositiveFloat`], [`Probability`], [`ByteSize`],
//! - [`Uuid`], [`MacAddress`], [`HumanDuration`], and [`PositiveDuration`]
//!   (parsing and `Display` do not allocate),
//! - the error types ([`PrimitiveError`], [`PrimitiveErrorKind`],
//!   [`PrimitiveResult`]).
//!
//! Enabling the `alloc` feature additionally provides the owned, allocation-backed
//! types: [`Slug`], [`Email`], [`HttpUrl`], [`HexString`], [`Base64`],
//! [`Identifier`], [`Hostname`], [`NonEmptyStr`], [`BoundedStr`], [`NonEmptyVec`],
//! and [`SemVer`]. The default `std` build enables `alloc` for normal application
//! use.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Bounded string primitive.
#[cfg(feature = "alloc")]
pub mod bounded;
/// Non-empty vector primitive.
#[cfg(feature = "alloc")]
pub mod collections;
/// Human-readable duration primitive.
pub mod duration;
/// Shared primitive error type.
pub mod error;
/// Inline, stack-allocated bounded string primitive.
pub mod inline;
/// MAC address primitive.
pub mod mac;
/// IP network (CIDR) primitive.
pub mod net;
/// Non-empty string primitive.
#[cfg(feature = "alloc")]
pub mod non_empty;
/// Numeric primitives.
pub mod numeric;
/// Semantic version primitive.
#[cfg(feature = "alloc")]
pub mod semver;
/// Text validation primitives.
#[cfg(feature = "alloc")]
pub mod text;
/// UUID primitive.
pub mod uuid;

#[cfg(feature = "alloc")]
pub use bounded::BoundedStr;
#[cfg(feature = "alloc")]
pub use collections::NonEmptyVec;
pub use duration::{HumanDuration, PositiveDuration};
pub use error::{PrimitiveError, PrimitiveErrorKind, PrimitiveResult};
pub use inline::InlineStr;
pub use mac::MacAddress;
pub use net::Cidr;
#[cfg(feature = "alloc")]
pub use non_empty::NonEmptyStr;
pub use numeric::{ByteSize, Percent, PercentFloat, Port, PositiveFloat, PositiveInt, Probability};
#[cfg(feature = "alloc")]
pub use semver::SemVer;
#[cfg(feature = "alloc")]
pub use text::{Base32, Base64, Email, HexString, Hostname, HttpUrl, Identifier, Slug};
pub use uuid::Uuid;

//! Composable validation traits and error types for Rust structs and values.
//!
//! `reliakit-validate` provides a small, focused toolkit for expressing
//! validation rules as types. The core pieces are:
//!
//! - [`Validate`] — a trait that types implement to describe their validity
//!   rules.
//! - [`Valid<T>`] — a zero-cost wrapper that carries proof of successful
//!   validation in the type system.
//! - [`ValidationError`] — an error type that collects one or more
//!   [`Violation`]s, useful for validating multiple fields at once and
//!   returning all failures together.
//!
//! # Examples
//!
//! ## Single-field validation
//!
//! ```
//! use reliakit_validate::{Validate, Valid, ValidationError};
//!
//! struct Username(String);
//!
//! impl Validate for Username {
//!     type Error = ValidationError;
//!
//!     fn validate(&self) -> Result<(), Self::Error> {
//!         if self.0.is_empty() {
//!             return Err(ValidationError::new("username must not be empty"));
//!         }
//!         if self.0.len() > 32 {
//!             return Err(ValidationError::new("username must not exceed 32 characters"));
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let user = Valid::new(Username("alice".into())).unwrap();
//! assert_eq!(user.0, "alice");
//! ```
//!
//! ## Multi-field struct validation
//!
//! ```
//! use reliakit_validate::{Validate, ValidationError, Violation};
//!
//! struct CreateUser {
//!     name: String,
//!     age: u8,
//! }
//!
//! impl Validate for CreateUser {
//!     type Error = ValidationError;
//!
//!     fn validate(&self) -> Result<(), Self::Error> {
//!         let mut errors = ValidationError::empty();
//!
//!         if self.name.is_empty() {
//!             errors.push(Violation::with_field("name", "must not be empty"));
//!         }
//!         if self.age < 18 {
//!             errors.push(Violation::with_field("age", "must be at least 18"));
//!         }
//!
//!         if errors.is_empty() { Ok(()) } else { Err(errors) }
//!     }
//! }
//!
//! let result = CreateUser { name: String::new(), age: 15 }.validate();
//! assert!(result.is_err());
//! assert_eq!(result.unwrap_err().len(), 2);
//! ```
//!
//! ## One error list for an API response
//!
//! Collecting every [`Violation`] lets a request handler report all field
//! problems at once (e.g. as an HTTP 422 body) instead of making the client fix
//! one error per round-trip:
//!
//! ```
//! use reliakit_validate::{Validate, ValidationError, Violation};
//!
//! struct Signup {
//!     email: String,
//!     password: String,
//! }
//!
//! impl Validate for Signup {
//!     type Error = ValidationError;
//!
//!     fn validate(&self) -> Result<(), Self::Error> {
//!         let mut errors = ValidationError::empty();
//!         if !self.email.contains('@') {
//!             errors.push(Violation::with_field("email", "must contain @"));
//!         }
//!         if self.password.len() < 8 {
//!             errors.push(Violation::with_field("password", "must be at least 8 characters"));
//!         }
//!         if errors.is_empty() { Ok(()) } else { Err(errors) }
//!     }
//! }
//!
//! let bad = Signup { email: "nope".into(), password: "x".into() };
//! let errors = bad.validate().unwrap_err();
//!
//! // Render to the (field, message) pairs a JSON error body would carry.
//! let body: Vec<(&str, &str)> = errors
//!     .violations()
//!     .iter()
//!     .map(|v| (v.field.unwrap_or("(root)"), v.message))
//!     .collect();
//! assert_eq!(
//!     body,
//!     vec![("email", "must contain @"), ("password", "must be at least 8 characters")]
//! );
//! ```
//!
//! For ready-made typed fields to validate — email, port, percentages, bounded
//! strings, and more — pair this crate with
//! [`reliakit-primitives`](https://docs.rs/reliakit-primitives). The
//! `config_check` example in the `reliakit` umbrella crate shows primitives,
//! validate, and secret working together on one config.

//! # Feature flags
//!
//! - `std` (default) enables `std::error::Error` for [`ValidationError`] and
//!   implies `alloc`.
//! - `alloc` enables [`ValidationError`] and [`ValidateResult`], which collect
//!   multiple [`Violation`]s in a `Vec`.
//!
//! # `no_std`
//!
//! The crate supports `no_std`. The [`Validate`] trait, [`Valid<T>`], and
//! [`Violation`] are available without `alloc`; implement [`Validate`] with your
//! own error type in allocation-free contexts. [`ValidationError`] and
//! [`ValidateResult`] require the `alloc` feature (enabled by default via `std`).

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod valid;

pub use error::Violation;
#[cfg(feature = "alloc")]
pub use error::{ValidateResult, ValidationError};
pub use valid::Valid;

/// A type that can validate itself.
///
/// Implement this trait to express the validity rules of a type. Use
/// [`Valid<T>`] to wrap validated values and carry the proof in the type
/// system.
///
/// # Example
///
/// ```
/// use reliakit_validate::{Validate, ValidationError};
///
/// struct Score(u8);
///
/// impl Validate for Score {
///     type Error = ValidationError;
///
///     fn validate(&self) -> Result<(), Self::Error> {
///         if self.0 > 100 {
///             return Err(ValidationError::new("score must not exceed 100"));
///         }
///         Ok(())
///     }
/// }
///
/// assert!(Score(100).validate().is_ok());
/// assert!(Score(101).validate().is_err());
/// ```
pub trait Validate {
    /// The error type returned when validation fails.
    type Error;

    /// Checks whether `self` satisfies its validity rules.
    ///
    /// Returns `Ok(())` if valid, or an error describing what failed.
    fn validate(&self) -> Result<(), Self::Error>;
}

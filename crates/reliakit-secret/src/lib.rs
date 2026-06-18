//! Secret-safe wrappers for values that should not leak through formatting or
//! diagnostics.
//!
//! `reliakit-secret` provides [`Secret<T>`], a small wrapper that redacts its
//! value in [`Debug`](core::fmt::Debug) and [`Display`](core::fmt::Display)
//! output. Access to the wrapped value is explicit through [`ExposeSecret`].
//!
//! The crate does not claim memory zeroization, process isolation, or protection
//! against memory inspection. Its purpose is to prevent accidental leaks through
//! logs, error messages, debug output, and diagnostic reports.
//!
//! # Examples
//!
//! ```
//! use reliakit_secret::{ExposeSecret, Secret};
//!
//! let token = Secret::new("ghp_example_token");
//!
//! assert_eq!(format!("{token:?}"), "Secret([REDACTED])");
//! assert_eq!(format!("{token}"), "[REDACTED]");
//! assert_eq!(token.expose_secret(), &"ghp_example_token");
//! ```
//!
//! String-backed secrets are available when `alloc` is available:
//!
//! ```
//! use reliakit_secret::{ExposeSecret, SecretString};
//!
//! let password = SecretString::from_string("correct horse battery staple");
//! assert_eq!(password.expose_secret(), "correct horse battery staple");
//! ```
//!
//! # Redacting a field inside a larger struct
//!
//! The common case is a secret that lives in a config or request struct. Because
//! [`Secret<T>`] redacts itself, deriving `Debug` on the parent stays safe: the
//! secret field renders as `[REDACTED]` while every other field prints normally,
//! so the whole struct can be logged without leaking:
//!
//! ```
//! use reliakit_secret::SecretString;
//!
//! #[derive(Debug)]
//! struct DbConfig {
//!     host: String,
//!     port: u16,
//!     password: SecretString,
//! }
//!
//! let cfg = DbConfig {
//!     host: "db.internal".into(),
//!     port: 5432,
//!     password: SecretString::from_string("hunter2"),
//! };
//!
//! let rendered = format!("{cfg:?}");
//! assert!(rendered.contains("db.internal"));
//! assert!(rendered.contains("[REDACTED]"));
//! assert!(!rendered.contains("hunter2")); // the secret never appears
//! ```
//!
//! # Comparing secrets
//!
//! Checking a presented value against a stored secret with `==` on the exposed
//! bytes can leak information through timing. Use [`Secret::ct_eq`], which
//! compares in time that does not depend on how many leading bytes match:
//!
//! ```
//! use reliakit_secret::SecretString;
//!
//! let stored = SecretString::from_string("s3cr3t-token");
//! assert!(stored.ct_eq("s3cr3t-token"));
//! assert!(!stored.ct_eq("s3cr3t-wrong"));
//! ```
//!
//! # Feature flags
//!
//! - `std` is enabled by default.
//! - `alloc` enables [`SecretString`] without `std`.
//!
//! # `no_std`
//!
//! The crate supports `no_std`. Use `default-features = false` for non-alloc
//! generic secrets, or add `features = ["alloc"]` for [`SecretString`].

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

use core::fmt;

#[cfg(any(feature = "alloc", feature = "std"))]
use alloc::string::String;

const REDACTED: &str = "[REDACTED]";

/// Explicit access to a wrapped secret value.
///
/// This trait makes secret exposure visible at call sites:
///
/// ```
/// use reliakit_secret::{ExposeSecret, Secret};
///
/// let secret = Secret::new("token");
/// assert_eq!(secret.expose_secret(), &"token");
/// ```
pub trait ExposeSecret<T: ?Sized> {
    /// Returns a shared reference to the wrapped secret value.
    fn expose_secret(&self) -> &T;
}

/// Mutable access to a wrapped secret value.
///
/// Use this only when mutation is necessary. Prefer constructing a new
/// [`Secret<T>`] when possible.
pub trait ExposeSecretMut<T: ?Sized>: ExposeSecret<T> {
    /// Returns a mutable reference to the wrapped secret value.
    fn expose_secret_mut(&mut self) -> &mut T;
}

/// A value that redacts itself in formatting and diagnostics.
///
/// `Secret<T>` intentionally does not expose `T` through `Debug`, `Display`, or
/// `AsRef`. Callers must use [`ExposeSecret::expose_secret`] or
/// [`Secret::into_inner`] explicitly.
pub struct Secret<T> {
    inner: T,
}

impl<T> Secret<T> {
    /// Wraps a value as a secret.
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Maps a secret value into another secret value.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Secret<U> {
        Secret::new(f(self.inner))
    }
}

impl<T: AsRef<[u8]>> Secret<T> {
    /// Compares this secret's bytes to `other` in time that does not depend on
    /// how many leading bytes match.
    ///
    /// Comparing a presented value against a stored secret with `==` returns as
    /// soon as the first differing byte is found, which can leak the secret one
    /// byte at a time through timing. `ct_eq` always inspects every byte, so the
    /// duration reveals only the input length, not its contents. Inputs of
    /// different lengths always compare unequal; the length itself is not
    /// treated as secret.
    ///
    /// This is a best-effort, dependency-free implementation built on
    /// [`core::hint::black_box`] to discourage the optimizer from
    /// short-circuiting. If you require audited constant-time guarantees, use a
    /// dedicated cryptographic comparison crate.
    ///
    /// Available for any secret whose value is byte-viewable (`String`,
    /// `Vec<u8>`, `&str`, `&[u8]`, `[u8; N]`, ...). To compare two secrets, pass
    /// the other's exposed value: `a.ct_eq(b.expose_secret())`.
    ///
    /// ```
    /// use reliakit_secret::Secret;
    ///
    /// let stored = Secret::new(*b"abc123");
    /// assert!(stored.ct_eq(b"abc123"));
    /// assert!(!stored.ct_eq(b"abc124"));
    /// assert!(!stored.ct_eq(b"abc")); // different length
    /// ```
    pub fn ct_eq(&self, other: impl AsRef<[u8]>) -> bool {
        let a = self.inner.as_ref();
        let b = other.as_ref();
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        core::hint::black_box(diff) == 0
    }
}

impl<T> ExposeSecret<T> for Secret<T> {
    fn expose_secret(&self) -> &T {
        &self.inner
    }
}

impl<T> ExposeSecretMut<T> for Secret<T> {
    fn expose_secret_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<T: Clone> Clone for Secret<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}

impl<T: Default> Default for Secret<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(")?;
        f.write_str(REDACTED)?;
        f.write_str(")")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// String-backed secret value.
#[cfg(any(feature = "alloc", feature = "std"))]
pub type SecretString = Secret<String>;

#[cfg(any(feature = "alloc", feature = "std"))]
impl SecretString {
    /// Creates a string-backed secret from any string-like value.
    pub fn from_string(value: impl Into<String>) -> Self {
        Self::new(value.into())
    }

    /// Returns the secret as `str`.
    pub fn expose_str(&self) -> &str {
        self.inner.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::{ExposeSecret, ExposeSecretMut, Secret, SecretString};
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn debug_redacts_secret() {
        let secret = Secret::new("token-123");
        assert_eq!(format!("{secret:?}"), "Secret([REDACTED])");
    }

    #[test]
    fn display_redacts_secret() {
        let secret = Secret::new("token-123");
        assert_eq!(secret.to_string(), "[REDACTED]");
    }

    #[test]
    fn expose_secret_returns_inner_reference() {
        let secret = Secret::new("token-123");
        assert_eq!(secret.expose_secret(), &"token-123");
    }

    #[test]
    fn expose_secret_mut_allows_explicit_mutation() {
        let mut secret = Secret::new(1_u8);
        *secret.expose_secret_mut() = 2;
        assert_eq!(secret.expose_secret(), &2);
    }

    #[test]
    fn into_inner_returns_inner_value() {
        let secret = Secret::new("token-123");
        assert_eq!(secret.into_inner(), "token-123");
    }

    #[test]
    fn map_returns_new_secret() {
        let secret = Secret::new("token").map(|value| value.len());
        assert_eq!(secret.expose_secret(), &5);
        assert_eq!(format!("{secret:?}"), "Secret([REDACTED])");
    }

    #[test]
    fn clone_clones_inner_value_without_leaking_debug() {
        let secret = Secret::new("token".to_string());
        let cloned = secret.clone();
        assert_eq!(cloned.expose_secret(), "token");
        assert_eq!(format!("{cloned:?}"), "Secret([REDACTED])");
    }

    #[test]
    fn secret_string_wraps_owned_string() {
        let secret = SecretString::from_string("password");
        assert_eq!(secret.expose_secret(), "password");
        assert_eq!(secret.expose_str(), "password");
        assert_eq!(secret.to_string(), "[REDACTED]");
    }

    #[test]
    fn ct_eq_matches_equal_bytes() {
        let secret = SecretString::from_string("s3cr3t-token");
        assert!(secret.ct_eq("s3cr3t-token"));
        assert!(secret.ct_eq(b"s3cr3t-token"));
    }

    #[test]
    fn ct_eq_rejects_different_content_same_length() {
        let secret = SecretString::from_string("s3cr3t-token");
        assert!(!secret.ct_eq("s3cr3t-tokeX"));
        // First and last bytes differ but length matches.
        assert!(!secret.ct_eq("X3cr3t-tokeX"));
    }

    #[test]
    fn ct_eq_rejects_different_length() {
        let secret = SecretString::from_string("abc");
        assert!(!secret.ct_eq("ab"));
        assert!(!secret.ct_eq("abcd"));
        assert!(!secret.ct_eq(""));
    }

    #[test]
    fn ct_eq_works_for_byte_arrays_without_alloc_types() {
        let secret = Secret::new(*b"key-bytes");
        assert!(secret.ct_eq(b"key-bytes"));
        assert!(!secret.ct_eq(b"key-byteX"));
    }

    #[test]
    fn ct_eq_empty_secret_matches_empty() {
        let secret = SecretString::from_string("");
        assert!(secret.ct_eq(""));
        assert!(!secret.ct_eq("x"));
    }

    #[test]
    fn ct_eq_between_two_secrets_via_expose() {
        let a = SecretString::from_string("same-value");
        let b = SecretString::from_string("same-value");
        assert!(a.ct_eq(b.expose_secret()));
    }
}

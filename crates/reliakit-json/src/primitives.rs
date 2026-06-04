//! Optional integration with [`reliakit-primitives`].
//!
//! Available with the `primitives` feature. These helpers pull a value out of a
//! parsed JSON document and run it through a `reliakit-primitives` validating
//! constructor, so you get a typed, validated value instead of a raw string —
//! and on failure the error carries the [`JsonPath`] of the offending location.
//!
//! ```
//! use reliakit_json::parse_str;
//! use reliakit_primitives::{Email, Hostname};
//!
//! let doc = parse_str(r#"{ "email": "ops@example.com", "host": "api.example.com" }"#).unwrap();
//! let obj = doc.as_object().unwrap();
//!
//! let email: Email = obj.get_str_as("email").unwrap();
//! let host: Hostname = obj.get_str_as("host").unwrap();
//! assert_eq!(email.domain(), "example.com");
//! assert_eq!(host.as_str(), "api.example.com");
//!
//! // A missing key, the wrong JSON type, or a value the primitive rejects all
//! // produce an error that points at the field.
//! let bad = parse_str(r#"{ "email": "not-an-email" }"#).unwrap();
//! let err = bad.as_object().unwrap().get_str_as::<Email>("email").unwrap_err();
//! assert!(err.to_string().starts_with("$.email"));
//! ```
//!
//! [`reliakit-primitives`]: https://docs.rs/reliakit-primitives

use alloc::string::ToString;
use alloc::vec;
use core::fmt;

use reliakit_primitives::PrimitiveError;

use crate::error::{JsonPath, JsonPathSegment};
use crate::{JsonObject, JsonValue};

/// Why extracting a typed [`reliakit-primitives`](https://docs.rs/reliakit-primitives)
/// value from JSON failed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum JsonExtractErrorKind {
    /// The requested object key was not present.
    Missing,
    /// The value was present but not the expected JSON shape.
    WrongType {
        /// The JSON kind that was expected, e.g. `"string"`.
        expected: &'static str,
    },
    /// The value had the right shape but failed primitive validation.
    Invalid(PrimitiveError),
}

/// An error from extracting a typed value out of JSON, carrying the
/// [`JsonPath`] of the offending location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonExtractError {
    path: JsonPath,
    kind: JsonExtractErrorKind,
}

impl JsonExtractError {
    /// The location of the failure, from the document root (`$`).
    pub fn path(&self) -> &JsonPath {
        &self.path
    }

    /// The reason extraction failed.
    pub fn kind(&self) -> &JsonExtractErrorKind {
        &self.kind
    }
}

impl fmt::Display for JsonExtractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            JsonExtractErrorKind::Missing => write!(f, "{}: required value is missing", self.path),
            JsonExtractErrorKind::WrongType { expected } => {
                write!(f, "{}: expected a JSON {expected}", self.path)
            }
            JsonExtractErrorKind::Invalid(err) => write!(f, "{}: {err}", self.path),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for JsonExtractError {}

/// Builds the single-segment path `$.<key>` for a field error.
fn key_path(key: &str) -> JsonPath {
    JsonPath::from_segments(vec![JsonPathSegment::Key(key.to_string())])
}

impl JsonObject {
    /// Extracts member `key` as a string-backed primitive `T`, validating it
    /// through `T`'s `TryFrom<&str>` constructor.
    ///
    /// Returns [`JsonExtractErrorKind::Missing`] if the key is absent,
    /// [`JsonExtractErrorKind::WrongType`] if the member is not a JSON string,
    /// and [`JsonExtractErrorKind::Invalid`] (wrapping the [`PrimitiveError`]) if
    /// the string fails validation. The error carries the `$.key` path.
    pub fn get_str_as<'a, T>(&'a self, key: &str) -> Result<T, JsonExtractError>
    where
        T: TryFrom<&'a str, Error = PrimitiveError>,
    {
        let value = self.get(key).ok_or_else(|| JsonExtractError {
            path: key_path(key),
            kind: JsonExtractErrorKind::Missing,
        })?;
        match value.as_str() {
            None => Err(JsonExtractError {
                path: key_path(key),
                kind: JsonExtractErrorKind::WrongType { expected: "string" },
            }),
            Some(text) => T::try_from(text).map_err(|err| JsonExtractError {
                path: key_path(key),
                kind: JsonExtractErrorKind::Invalid(err),
            }),
        }
    }
}

impl JsonValue {
    /// Reads this value as a string-backed primitive `T`, validating it through
    /// `T`'s `TryFrom<&str>` constructor.
    ///
    /// Returns [`JsonExtractErrorKind::WrongType`] if this is not a JSON string,
    /// or [`JsonExtractErrorKind::Invalid`] if it fails validation. The error
    /// path is the document root (`$`); use
    /// [`JsonObject::get_str_as`] when the value lives under a key so the error
    /// points at that field.
    pub fn str_as<'a, T>(&'a self) -> Result<T, JsonExtractError>
    where
        T: TryFrom<&'a str, Error = PrimitiveError>,
    {
        match self.as_str() {
            None => Err(JsonExtractError {
                path: JsonPath::default(),
                kind: JsonExtractErrorKind::WrongType { expected: "string" },
            }),
            Some(text) => T::try_from(text).map_err(|err| JsonExtractError {
                path: JsonPath::default(),
                kind: JsonExtractErrorKind::Invalid(err),
            }),
        }
    }
}

#[cfg(all(test, feature = "primitives"))]
mod tests {
    use super::{JsonExtractErrorKind, PrimitiveError};
    use crate::parse_str;
    use reliakit_primitives::{Email, Hostname};

    fn obj(input: &str) -> crate::JsonObject {
        parse_str(input).unwrap().as_object().unwrap().clone()
    }

    #[test]
    fn extracts_valid_string_primitive() {
        let o = obj(r#"{ "email": "ops@example.com", "host": "api.example.com" }"#);
        let email: Email = o.get_str_as("email").unwrap();
        assert_eq!(email.as_str(), "ops@example.com");
        let host: Hostname = o.get_str_as("host").unwrap();
        assert_eq!(host.as_str(), "api.example.com");
    }

    #[test]
    fn missing_key_reports_missing_with_path() {
        let o = obj(r#"{ "host": "api.example.com" }"#);
        let err = o.get_str_as::<Email>("email").unwrap_err();
        assert_eq!(err.kind(), &JsonExtractErrorKind::Missing);
        assert_eq!(err.path().to_string(), "$.email");
        assert_eq!(err.to_string(), "$.email: required value is missing");
    }

    #[test]
    fn wrong_json_type_reports_wrong_type() {
        let o = obj(r#"{ "email": 42 }"#);
        let err = o.get_str_as::<Email>("email").unwrap_err();
        assert_eq!(
            err.kind(),
            &JsonExtractErrorKind::WrongType { expected: "string" }
        );
        assert_eq!(err.to_string(), "$.email: expected a JSON string");
    }

    #[test]
    fn invalid_value_wraps_primitive_error_with_path() {
        let o = obj(r#"{ "email": "not-an-email" }"#);
        let err = o.get_str_as::<Email>("email").unwrap_err();
        assert!(matches!(
            err.kind(),
            JsonExtractErrorKind::Invalid(PrimitiveError::Invalid { .. })
        ));
        assert!(err.to_string().starts_with("$.email: "));
    }

    #[test]
    fn value_str_as_uses_root_path() {
        let doc = parse_str(r#""ops@example.com""#).unwrap();
        let email: Email = doc.str_as().unwrap();
        assert_eq!(email.as_str(), "ops@example.com");

        let num = parse_str("42").unwrap();
        let err = num.str_as::<Email>().unwrap_err();
        assert_eq!(err.path().to_string(), "$");
        assert_eq!(
            err.kind(),
            &JsonExtractErrorKind::WrongType { expected: "string" }
        );
    }
}

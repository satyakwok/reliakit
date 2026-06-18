//! Optional integration with [`reliakit-validate`].
//!
//! Available with the `validate` feature (which also enables `primitives`).
//! [`JsonForm`] pulls several fields out of a JSON object and collects *every*
//! failure into a single [`ValidationError`], rather than stopping at the first
//! one, the usual shape for validating an untrusted request body.
//!
//! ```
//! use reliakit_json::{JsonForm, parse_str};
//! use reliakit_primitives::{Email, Hostname};
//!
//! let doc = parse_str(r#"{ "email": "nope", "host": 42 }"#).unwrap();
//! let obj = doc.as_object().unwrap();
//!
//! let mut form = JsonForm::new(obj);
//! let _email: Option<Email> = form.str_field("email");
//! let _host: Option<Hostname> = form.str_field("host");
//!
//! // Both fields failed, and both are reported together.
//! let errors = form.finish().unwrap_err();
//! assert_eq!(errors.violations().len(), 2);
//! ```
//!
//! Because [`Violation`] messages are `&'static str`, the per-field message is a
//! fixed summary (`is required` / `must be a string` / `is invalid`). For the
//! full [`PrimitiveError`](reliakit_primitives::PrimitiveError) detail of a
//! single field, use [`JsonObject::get_str_as`](crate::JsonObject::get_str_as).

use reliakit_primitives::PrimitiveError;
use reliakit_validate::{ValidateResult, ValidationError, Violation};

use crate::JsonObject;
use crate::primitives::JsonExtractErrorKind;

/// Accumulates field-extraction failures from a [`JsonObject`] into a single
/// [`ValidationError`].
///
/// Create one with [`new`](Self::new), pull each field with
/// [`str_field`](Self::str_field), then call [`finish`](Self::finish) to get
/// `Ok(())` or every collected [`Violation`] at once.
pub struct JsonForm<'a> {
    object: &'a JsonObject,
    errors: ValidationError,
}

impl<'a> JsonForm<'a> {
    /// Starts a form over `object` with no recorded violations.
    pub fn new(object: &'a JsonObject) -> Self {
        Self {
            object,
            errors: ValidationError::empty(),
        }
    }

    /// Extracts `field` as a string-backed primitive `T`.
    ///
    /// On success returns `Some(value)`. On failure records a [`Violation`] named
    /// `field` (with a fixed summary message) and returns `None`, so validation
    /// continues and every bad field is reported by [`finish`](Self::finish).
    pub fn str_field<T>(&mut self, field: &'static str) -> Option<T>
    where
        T: TryFrom<&'a str, Error = PrimitiveError>,
    {
        let object = self.object;
        match object.get_str_as::<T>(field) {
            Ok(value) => Some(value),
            Err(error) => {
                let message = match error.kind() {
                    JsonExtractErrorKind::Missing => "is required",
                    JsonExtractErrorKind::WrongType { .. } => "must be a string",
                    _ => "is invalid",
                };
                self.errors.push(Violation::with_field(field, message));
                None
            }
        }
    }

    /// Returns `true` if no violations have been recorded yet.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Borrows the violations recorded so far.
    pub fn errors(&self) -> &ValidationError {
        &self.errors
    }

    /// Consumes the form: `Ok(())` if every field validated, otherwise `Err`
    /// with all collected violations.
    pub fn finish(self) -> ValidateResult {
        self.errors.finish()
    }
}

#[cfg(all(test, feature = "validate"))]
mod tests {
    use super::JsonForm;
    use crate::parse_str;
    use reliakit_primitives::{Email, Hostname};

    fn obj(input: &str) -> crate::JsonObject {
        parse_str(input).unwrap().as_object().unwrap().clone()
    }

    #[test]
    fn all_fields_valid_finishes_ok() {
        let o = obj(r#"{ "email": "ops@example.com", "host": "api.example.com" }"#);
        let mut form = JsonForm::new(&o);
        let email: Option<Email> = form.str_field("email");
        let host: Option<Hostname> = form.str_field("host");
        assert_eq!(email.unwrap().as_str(), "ops@example.com");
        assert_eq!(host.unwrap().as_str(), "api.example.com");
        assert!(form.is_valid());
        assert!(form.finish().is_ok());
    }

    #[test]
    fn collects_every_failure_together() {
        let o = obj(r#"{ "email": "nope", "host": 42 }"#);
        let mut form = JsonForm::new(&o);
        assert!(form.str_field::<Email>("email").is_none());
        assert!(form.str_field::<Hostname>("host").is_none());
        assert!(form.str_field::<Email>("missing").is_none());
        assert!(!form.is_valid());

        let errors = form.finish().unwrap_err();
        let v = errors.violations();
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].field, Some("email"));
        assert_eq!(v[0].message, "is invalid");
        assert_eq!(v[1].field, Some("host"));
        assert_eq!(v[1].message, "must be a string");
        assert_eq!(v[2].field, Some("missing"));
        assert_eq!(v[2].message, "is required");
    }

    #[test]
    fn errors_accessor_tracks_progress() {
        let o = obj(r#"{ "email": "ops@example.com" }"#);
        let mut form = JsonForm::new(&o);
        let _: Option<Email> = form.str_field("email");
        assert!(form.errors().is_empty());
        let _: Option<Hostname> = form.str_field("host"); // missing
        assert_eq!(form.errors().len(), 1);
    }
}

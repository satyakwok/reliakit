use alloc::vec::Vec;
use core::fmt;

/// A single failed constraint, optionally associated with a named field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The field name, if validation was run on a named field.
    pub field: Option<&'static str>,
    /// A human-readable description of the constraint that failed.
    pub message: &'static str,
}

impl Violation {
    /// Creates a violation without a field name.
    pub const fn new(message: &'static str) -> Self {
        Self {
            field: None,
            message,
        }
    }

    /// Creates a violation associated with a named field.
    pub const fn with_field(field: &'static str, message: &'static str) -> Self {
        Self {
            field: Some(field),
            message,
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.field {
            Some(field) => write!(f, "{field}: {}", self.message),
            None => f.write_str(self.message),
        }
    }
}

/// One or more validation failures collected during validation.
///
/// `ValidationError` is designed for multi-field struct validation where all
/// fields should be checked and all violations reported together. For
/// single-value validation, a simpler error type may be more appropriate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    violations: Vec<Violation>,
}

/// Result alias for validation operations.
pub type ValidateResult<T = ()> = Result<T, ValidationError>;

impl ValidationError {
    /// Creates a `ValidationError` with a single unnamed violation.
    pub fn new(message: &'static str) -> Self {
        Self {
            violations: alloc::vec![Violation::new(message)],
        }
    }

    /// Creates a `ValidationError` with a single named field violation.
    pub fn field(field: &'static str, message: &'static str) -> Self {
        Self {
            violations: alloc::vec![Violation::with_field(field, message)],
        }
    }

    /// Creates an empty `ValidationError`. Useful for building up violations.
    ///
    /// Always check [`is_empty`](Self::is_empty) before returning this as
    /// `Err`. Returning an empty `ValidationError` is valid Rust but conveys
    /// no information to the caller.
    pub fn empty() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    /// Adds a violation and returns `self` for chaining.
    pub fn with(mut self, violation: Violation) -> Self {
        self.violations.push(violation);
        self
    }

    /// Adds a violation in place.
    pub fn push(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// Merges another `ValidationError` into this one.
    pub fn merge(mut self, other: Self) -> Self {
        self.violations.extend(other.violations);
        self
    }

    /// Returns all violations.
    pub fn violations(&self) -> &[Violation] {
        &self.violations
    }

    /// Returns `true` if there are no violations.
    pub fn is_empty(&self) -> bool {
        self.violations.is_empty()
    }

    /// Returns the number of violations.
    pub fn len(&self) -> usize {
        self.violations.len()
    }
}

impl From<Violation> for ValidationError {
    fn from(v: Violation) -> Self {
        Self {
            violations: alloc::vec![v],
        }
    }
}

impl From<&'static str> for ValidationError {
    fn from(message: &'static str) -> Self {
        Self::new(message)
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.violations.as_slice() {
            [] => f.write_str("validation failed"),
            [single] => fmt::Display::fmt(single, f),
            violations => {
                for (i, v) in violations.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    fmt::Display::fmt(v, f)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::{ValidateResult, ValidationError, Violation};
    use alloc::string::ToString;

    #[test]
    fn violation_new() {
        let v = Violation::new("must not be empty");
        assert_eq!(v.field, None);
        assert_eq!(v.message, "must not be empty");
    }

    #[test]
    fn violation_with_field() {
        let v = Violation::with_field("email", "invalid format");
        assert_eq!(v.field, Some("email"));
        assert_eq!(v.message, "invalid format");
    }

    #[test]
    fn violation_display_no_field() {
        assert_eq!(Violation::new("bad value").to_string(), "bad value");
    }

    #[test]
    fn violation_display_with_field() {
        assert_eq!(
            Violation::with_field("age", "must be positive").to_string(),
            "age: must be positive"
        );
    }

    #[test]
    fn validation_error_single_violation() {
        let e = ValidationError::new("value is required");
        assert_eq!(e.len(), 1);
        assert!(!e.is_empty());
        assert_eq!(e.to_string(), "value is required");
    }

    #[test]
    fn validation_error_field() {
        let e = ValidationError::field("name", "too short");
        assert_eq!(e.violations()[0].field, Some("name"));
        assert_eq!(e.to_string(), "name: too short");
    }

    #[test]
    fn validation_error_empty() {
        let e = ValidationError::empty();
        assert!(e.is_empty());
        assert_eq!(e.len(), 0);
        assert_eq!(e.to_string(), "validation failed");
    }

    #[test]
    fn validation_error_add_chaining() {
        let e = ValidationError::empty()
            .with(Violation::with_field("name", "too short"))
            .with(Violation::with_field("email", "invalid format"));
        assert_eq!(e.len(), 2);
    }

    #[test]
    fn validation_error_push() {
        let mut e = ValidationError::empty();
        e.push(Violation::new("first"));
        e.push(Violation::new("second"));
        assert_eq!(e.len(), 2);
    }

    #[test]
    fn validation_error_merge() {
        let a = ValidationError::new("first");
        let b = ValidationError::new("second");
        let merged = a.merge(b);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn validation_error_display_multiple() {
        let e = ValidationError::empty()
            .with(Violation::new("first error"))
            .with(Violation::new("second error"));
        assert_eq!(e.to_string(), "first error; second error");
    }

    #[test]
    fn validation_error_from_violation() {
        let e = ValidationError::from(Violation::new("bad"));
        assert_eq!(e.len(), 1);
    }

    #[test]
    fn validation_error_from_str() {
        let e = ValidationError::from("bad input");
        assert_eq!(e.violations()[0].message, "bad input");
    }

    #[test]
    fn validate_result_type_alias() {
        let ok: ValidateResult = Ok(());
        let err: ValidateResult = Err(ValidationError::new("fail"));
        assert!(ok.is_ok());
        assert!(err.is_err());
    }
}

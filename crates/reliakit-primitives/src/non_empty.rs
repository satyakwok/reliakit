use crate::{PrimitiveError, PrimitiveResult};
use alloc::string::String;
use core::{fmt, hash::Hash, ops::Deref};

/// Owned string that is not empty and not whitespace-only.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyStr(String);

impl NonEmptyStr {
    /// Creates a new `NonEmptyStr`.
    ///
    /// The original string is preserved, but empty and whitespace-only inputs
    /// are rejected.
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(PrimitiveError::Empty);
        }
        Ok(Self(value))
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the owned inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns the byte length of the inner string.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always returns `false`.
    ///
    /// This method is provided for compatibility with string-like APIs.
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl fmt::Display for NonEmptyStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for NonEmptyStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for NonEmptyStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<String> for NonEmptyStr {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for NonEmptyStr {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NonEmptyStr> for String {
    fn from(value: NonEmptyStr) -> Self {
        value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::NonEmptyStr;
    use crate::PrimitiveError;

    #[test]
    fn accepts_valid_strings() {
        let value = NonEmptyStr::new("service-api").unwrap();
        assert_eq!(value.as_str(), "service-api");
        assert!(!value.is_empty());
    }

    #[test]
    fn rejects_empty_string() {
        assert_eq!(NonEmptyStr::new("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn rejects_whitespace_only_string() {
        assert_eq!(NonEmptyStr::new("   ").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn preserves_original_string() {
        let value = NonEmptyStr::new("  api  ").unwrap();
        assert_eq!(value.as_str(), "  api  ");
    }
}

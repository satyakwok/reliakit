use crate::{PrimitiveError, PrimitiveResult};
use alloc::string::String;
use core::{fmt, hash::Hash, ops::Deref, str::FromStr};

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

    /// Returns the character length of the inner string (Unicode scalar values).
    pub fn len(&self) -> usize {
        self.0.chars().count()
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

impl FromStr for NonEmptyStr {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl From<NonEmptyStr> for String {
    fn from(value: NonEmptyStr) -> Self {
        value.into_inner()
    }
}

impl PartialEq<str> for NonEmptyStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for NonEmptyStr {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for NonEmptyStr {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for NonEmptyStr {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::NonEmptyStr;
    use crate::PrimitiveError;
    use alloc::string::{String, ToString};

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

    #[test]
    fn into_inner_returns_string() {
        let value = NonEmptyStr::new("hello").unwrap();
        assert_eq!(value.into_inner(), "hello");
    }

    #[test]
    fn len_returns_char_count() {
        let value = NonEmptyStr::new("hello").unwrap();
        assert_eq!(value.len(), 5);
        let unicode = NonEmptyStr::new("éàü").unwrap();
        assert_eq!(unicode.len(), 3);
    }

    #[test]
    fn display_formats_inner_string() {
        let value = NonEmptyStr::new("hello").unwrap();
        assert_eq!(value.to_string(), "hello");
    }

    #[test]
    fn as_ref_returns_str() {
        let value = NonEmptyStr::new("hello").unwrap();
        let s: &str = value.as_ref();
        assert_eq!(s, "hello");
    }

    #[test]
    fn deref_to_str() {
        let value = NonEmptyStr::new("hello").unwrap();
        assert_eq!(&*value, "hello");
    }

    #[test]
    fn try_from_string() {
        let value = NonEmptyStr::try_from(String::from("hello")).unwrap();
        assert_eq!(value.as_str(), "hello");
    }

    #[test]
    fn try_from_str_ref() {
        let value = NonEmptyStr::try_from("hello").unwrap();
        assert_eq!(value.as_str(), "hello");
    }

    #[test]
    fn from_non_empty_str_into_string() {
        let value = NonEmptyStr::new("hello").unwrap();
        let s = String::from(value);
        assert_eq!(s, "hello");
    }

    #[test]
    fn from_str_and_string_comparisons() {
        let value = "hello".parse::<NonEmptyStr>().unwrap();
        let owned = String::from("hello");
        assert_eq!(value, "hello");
        assert_eq!(value, owned);
        assert!(NonEmptyStr::try_from("   ").is_err());
    }
}

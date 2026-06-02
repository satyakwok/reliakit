use crate::{PrimitiveError, PrimitiveResult};
use alloc::string::String;
use core::{fmt, hash::Hash, ops::Deref, str::FromStr};

/// Owned string constrained by inclusive character length bounds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundedStr<const MIN: usize, const MAX: usize>(String);

impl<const MIN: usize, const MAX: usize> BoundedStr<MIN, MAX> {
    /// Creates a new bounded string.
    ///
    /// Length is measured in Unicode scalar values via `chars().count()`, not
    /// bytes. If `MIN > MAX`, construction returns `OutOfRange`.
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        let actual = value.chars().count();

        if MIN > MAX {
            return Err(PrimitiveError::OutOfRange {
                min: MIN as u128,
                max: MAX as u128,
                actual: actual as u128,
            });
        }

        if actual < MIN {
            return Err(PrimitiveError::TooShort { min: MIN, actual });
        }

        if actual > MAX {
            return Err(PrimitiveError::TooLong { max: MAX, actual });
        }

        if MIN > 0 && value.trim().is_empty() {
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

    /// Returns the character length of the inner string.
    pub fn len(&self) -> usize {
        self.0.chars().count()
    }

    /// Returns whether the inner string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the minimum allowed character length.
    pub fn min_len(&self) -> usize {
        MIN
    }

    /// Returns the maximum allowed character length.
    pub fn max_len(&self) -> usize {
        MAX
    }
}

impl<const MIN: usize, const MAX: usize> fmt::Display for BoundedStr<MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<const MIN: usize, const MAX: usize> AsRef<str> for BoundedStr<MIN, MAX> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const MIN: usize, const MAX: usize> Deref for BoundedStr<MIN, MAX> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const MIN: usize, const MAX: usize> TryFrom<String> for BoundedStr<MIN, MAX> {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MIN: usize, const MAX: usize> TryFrom<&str> for BoundedStr<MIN, MAX> {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MIN: usize, const MAX: usize> FromStr for BoundedStr<MIN, MAX> {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl<const MIN: usize, const MAX: usize> From<BoundedStr<MIN, MAX>> for String {
    fn from(value: BoundedStr<MIN, MAX>) -> Self {
        value.into_inner()
    }
}

impl<const MIN: usize, const MAX: usize> PartialEq<str> for BoundedStr<MIN, MAX> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const MIN: usize, const MAX: usize> PartialEq<&str> for BoundedStr<MIN, MAX> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const MIN: usize, const MAX: usize> PartialEq<String> for BoundedStr<MIN, MAX> {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const MIN: usize, const MAX: usize> PartialEq<&String> for BoundedStr<MIN, MAX> {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedStr;
    use crate::PrimitiveError;
    use alloc::string::{String, ToString};

    #[test]
    fn accepts_valid_length() {
        let value = BoundedStr::<3, 12>::new("service").unwrap();
        assert_eq!(value.as_str(), "service");
        assert_eq!(value.len(), 7);
        assert_eq!(value.min_len(), 3);
        assert_eq!(value.max_len(), 12);
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(
            BoundedStr::<3, 12>::new("ab").unwrap_err(),
            PrimitiveError::TooShort { min: 3, actual: 2 }
        );
    }

    #[test]
    fn rejects_too_long() {
        assert_eq!(
            BoundedStr::<3, 5>::new("service").unwrap_err(),
            PrimitiveError::TooLong { max: 5, actual: 7 }
        );
    }

    #[test]
    fn counts_unicode_chars() {
        let value = BoundedStr::<2, 2>::new("éå").unwrap();
        assert_eq!(value.len(), 2);
        assert_eq!(value.as_str().len(), 4);
    }

    #[test]
    fn rejects_whitespace_only_when_min_positive() {
        assert_eq!(
            BoundedStr::<1, 5>::new("  ").unwrap_err(),
            PrimitiveError::Empty
        );
    }

    #[test]
    fn handles_invalid_bounds() {
        assert_eq!(
            BoundedStr::<5, 3>::new("abcd").unwrap_err(),
            PrimitiveError::OutOfRange {
                min: 5,
                max: 3,
                actual: 4
            }
        );
    }

    #[test]
    fn into_inner_returns_string() {
        let value = BoundedStr::<3, 10>::new("hello").unwrap();
        assert_eq!(value.into_inner(), "hello");
    }

    #[test]
    fn is_empty_returns_false_for_valid() {
        let value = BoundedStr::<3, 10>::new("hello").unwrap();
        assert!(!value.is_empty());
    }

    #[test]
    fn display_formats_inner_string() {
        let value = BoundedStr::<3, 10>::new("hello").unwrap();
        assert_eq!(value.to_string(), "hello");
    }

    #[test]
    fn as_ref_returns_str() {
        let value = BoundedStr::<3, 10>::new("hello").unwrap();
        let s: &str = value.as_ref();
        assert_eq!(s, "hello");
    }

    #[test]
    fn deref_to_str() {
        let value = BoundedStr::<3, 10>::new("hello").unwrap();
        assert_eq!(&*value, "hello");
    }

    #[test]
    fn try_from_string() {
        let value = BoundedStr::<3, 10>::try_from(String::from("hello")).unwrap();
        assert_eq!(value.as_str(), "hello");
    }

    #[test]
    fn try_from_str_ref() {
        let value = BoundedStr::<3, 10>::try_from("hello").unwrap();
        assert_eq!(value.as_str(), "hello");
    }

    #[test]
    fn from_bounded_str_into_string() {
        let value = BoundedStr::<3, 10>::new("hello").unwrap();
        let s = String::from(value);
        assert_eq!(s, "hello");
    }

    #[test]
    fn allows_zero_min_whitespace_only() {
        let value = BoundedStr::<0, 5>::new("   ").unwrap();
        assert_eq!(value.as_str(), "   ");
    }

    #[test]
    fn from_str_and_string_comparisons() {
        let value = "hello".parse::<BoundedStr<3, 10>>().unwrap();
        let owned = String::from("hello");
        assert_eq!(value, "hello");
        assert_eq!(value, owned);
        assert!("hi".parse::<BoundedStr<3, 10>>().is_err());
    }
}

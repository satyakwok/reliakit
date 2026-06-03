use core::fmt;

/// Stable category for a primitive validation error.
///
/// This lets callers match on broad failure kinds without depending on display
/// text or static validation messages.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PrimitiveErrorKind {
    /// The value was empty or whitespace-only.
    Empty,
    /// The value was shorter than the minimum allowed length.
    TooShort,
    /// The value was longer than the maximum allowed length.
    TooLong,
    /// The value was outside the inclusive allowed range.
    OutOfRange,
    /// The value did not match the expected format.
    InvalidFormat,
}

/// Error returned when a primitive value fails validation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    /// The value was empty or contained only whitespace.
    Empty,
    /// The value was shorter than the minimum allowed length.
    TooShort {
        /// Minimum allowed length.
        min: usize,
        /// Actual observed length.
        actual: usize,
    },
    /// The value was longer than the maximum allowed length.
    TooLong {
        /// Maximum allowed length.
        max: usize,
        /// Actual observed length.
        actual: usize,
    },
    /// The value was outside the inclusive allowed range.
    OutOfRange {
        /// Minimum allowed value.
        min: u128,
        /// Maximum allowed value.
        max: u128,
        /// Actual observed value.
        actual: u128,
    },
    /// The value did not match the expected format or pattern.
    Invalid {
        /// Static validation message describing why the value is invalid.
        message: &'static str,
    },
}

/// Result alias used by Reliakit primitive constructors.
pub type PrimitiveResult<T> = Result<T, PrimitiveError>;

impl PrimitiveError {
    /// Returns the stable category for this error.
    pub const fn kind(&self) -> PrimitiveErrorKind {
        match self {
            Self::Empty => PrimitiveErrorKind::Empty,
            Self::TooShort { .. } => PrimitiveErrorKind::TooShort,
            Self::TooLong { .. } => PrimitiveErrorKind::TooLong,
            Self::OutOfRange { .. } => PrimitiveErrorKind::OutOfRange,
            Self::Invalid { .. } => PrimitiveErrorKind::InvalidFormat,
        }
    }
}

impl fmt::Display for PrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("value must not be empty"),
            Self::TooShort { min, actual } => {
                write!(
                    f,
                    "value is too short: minimum is {min}, actual is {actual}"
                )
            }
            Self::TooLong { max, actual } => {
                write!(f, "value is too long: maximum is {max}, actual is {actual}")
            }
            Self::OutOfRange { min, max, actual } => {
                write!(
                    f,
                    "value is out of range: expected {min}..={max}, actual is {actual}"
                )
            }
            Self::Invalid { message } => write!(f, "invalid value: {message}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PrimitiveError {}

#[cfg(test)]
mod tests {
    use super::{PrimitiveError, PrimitiveErrorKind};
    use alloc::string::ToString;

    #[test]
    fn display_empty() {
        assert_eq!(PrimitiveError::Empty.to_string(), "value must not be empty");
    }

    #[test]
    fn display_too_short() {
        assert_eq!(
            PrimitiveError::TooShort { min: 3, actual: 1 }.to_string(),
            "value is too short: minimum is 3, actual is 1"
        );
    }

    #[test]
    fn display_too_long() {
        assert_eq!(
            PrimitiveError::TooLong { max: 5, actual: 8 }.to_string(),
            "value is too long: maximum is 5, actual is 8"
        );
    }

    #[test]
    fn display_out_of_range() {
        assert_eq!(
            PrimitiveError::OutOfRange {
                min: 1,
                max: 100,
                actual: 200
            }
            .to_string(),
            "value is out of range: expected 1..=100, actual is 200"
        );
    }

    #[test]
    fn display_invalid() {
        assert_eq!(
            PrimitiveError::Invalid {
                message: "bad format"
            }
            .to_string(),
            "invalid value: bad format"
        );
    }

    #[test]
    fn kind_returns_stable_error_category() {
        assert_eq!(PrimitiveError::Empty.kind(), PrimitiveErrorKind::Empty);
        assert_eq!(
            PrimitiveError::TooShort { min: 3, actual: 1 }.kind(),
            PrimitiveErrorKind::TooShort
        );
        assert_eq!(
            PrimitiveError::TooLong { max: 5, actual: 8 }.kind(),
            PrimitiveErrorKind::TooLong
        );
        assert_eq!(
            PrimitiveError::OutOfRange {
                min: 1,
                max: 100,
                actual: 200
            }
            .kind(),
            PrimitiveErrorKind::OutOfRange
        );
        assert_eq!(
            PrimitiveError::Invalid {
                message: "bad format"
            }
            .kind(),
            PrimitiveErrorKind::InvalidFormat
        );
    }
}

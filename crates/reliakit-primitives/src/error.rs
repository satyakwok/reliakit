use core::fmt;

/// Error returned when a primitive value fails validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    /// The value was empty or contained only whitespace.
    Empty,
    /// The value was shorter than the minimum allowed length.
    TooShort { min: usize, actual: usize },
    /// The value was longer than the maximum allowed length.
    TooLong { max: usize, actual: usize },
    /// The value was outside the inclusive allowed range.
    OutOfRange { min: u128, max: u128, actual: u128 },
}

/// Result alias used by Reliakit primitive constructors.
pub type PrimitiveResult<T> = Result<T, PrimitiveError>;

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
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PrimitiveError {}

#[cfg(test)]
mod tests {
    use super::PrimitiveError;
    use alloc::string::ToString;

    #[test]
    fn display_empty() {
        assert_eq!(PrimitiveError::Empty.to_string(), "value must not be empty");
    }
}

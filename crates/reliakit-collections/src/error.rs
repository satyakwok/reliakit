use core::fmt;

/// Error returned by bounded collection operations.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionError {
    /// The collection has fewer elements than the required minimum.
    TooFew {
        /// The required minimum number of elements.
        min: usize,
        /// The actual number of elements.
        actual: usize,
    },
    /// The collection has more elements than the allowed maximum.
    TooMany {
        /// The allowed maximum number of elements.
        max: usize,
        /// The actual number of elements.
        actual: usize,
    },
    /// The const generic bounds are invalid (`MIN > MAX`).
    InvalidBounds {
        /// The configured minimum bound.
        min: usize,
        /// The configured maximum bound.
        max: usize,
    },
    /// A capacity of zero was requested where a positive capacity is required.
    ZeroCapacity,
    /// A duplicate key or element was supplied where uniqueness is required.
    Duplicate,
    /// The provided range is invalid for this operation (e.g. end exceeds the collection's length).
    InvalidRange {
        /// The starting index of the requested range.
        start: usize,
        /// The ending index (exclusive) of the requested range.
        end: usize,
        /// The actual total length of the container.
        len: usize,
    },
}

/// Result alias for bounded collection operations.
pub type CollectionResult<T = ()> = Result<T, CollectionError>;

impl fmt::Display for CollectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFew { min, actual } => {
                write!(
                    f,
                    "collection is too small: minimum is {min}, actual is {actual}"
                )
            }
            Self::TooMany { max, actual } => {
                write!(
                    f,
                    "collection is too large: maximum is {max}, actual is {actual}"
                )
            }
            Self::InvalidBounds { min, max } => {
                write!(f, "invalid bounds: MIN ({min}) must not exceed MAX ({max})")
            }
            Self::ZeroCapacity => write!(f, "capacity must be greater than zero"),
            Self::Duplicate => write!(f, "collection contains a duplicate key or element"),
            Self::InvalidRange { start, end, len } => write!(
                f,
                "invalid range {}..{} for collection of length {}",
                start, end, len
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CollectionError {}

#[cfg(test)]
mod tests {
    use super::CollectionError;
    use alloc::string::ToString;

    #[test]
    fn display_too_few() {
        assert_eq!(
            CollectionError::TooFew { min: 2, actual: 0 }.to_string(),
            "collection is too small: minimum is 2, actual is 0"
        );
    }

    #[test]
    fn display_too_many() {
        assert_eq!(
            CollectionError::TooMany { max: 5, actual: 6 }.to_string(),
            "collection is too large: maximum is 5, actual is 6"
        );
    }

    #[test]
    fn display_invalid_bounds() {
        assert_eq!(
            CollectionError::InvalidBounds { min: 5, max: 3 }.to_string(),
            "invalid bounds: MIN (5) must not exceed MAX (3)"
        );
    }

    #[test]
    fn display_zero_capacity() {
        assert_eq!(
            CollectionError::ZeroCapacity.to_string(),
            "capacity must be greater than zero"
        );
    }

    #[test]
    fn display_duplicate() {
        assert_eq!(
            CollectionError::Duplicate.to_string(),
            "collection contains a duplicate key or element"
        );
    }
}

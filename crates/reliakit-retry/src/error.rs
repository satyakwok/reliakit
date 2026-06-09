//! The retry error type.

use core::fmt;

/// The error returned when a retried operation does not succeed.
///
/// It carries how many attempts were made and the last error the operation
/// produced. There is intentionally no `E: core::error::Error` bound and no
/// allocation: the failing value is moved in by value.
///
/// `#[non_exhaustive]`: new variants may be added in a future release, so match
/// with a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryError<E> {
    /// No further attempts will be made. This covers both running out of the
    /// allowed attempts and the retry predicate declining to retry: in either
    /// case `attempts` is the number of attempts actually made and `last_error`
    /// is the error from the final attempt.
    Exhausted {
        /// The number of attempts made (always `>= 1`).
        attempts: u32,
        /// The error returned by the final attempt.
        last_error: E,
    },
}

impl<E> RetryError<E> {
    /// The number of attempts that were made.
    pub fn attempts(&self) -> u32 {
        match self {
            Self::Exhausted { attempts, .. } => *attempts,
        }
    }

    /// A reference to the error returned by the final attempt.
    pub fn last_error(&self) -> &E {
        match self {
            Self::Exhausted { last_error, .. } => last_error,
        }
    }

    /// Consumes the error and returns the final attempt's error.
    pub fn into_last_error(self) -> E {
        match self {
            Self::Exhausted { last_error, .. } => last_error,
        }
    }
}

impl<E: fmt::Display> fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted {
                attempts,
                last_error,
            } => write!(f, "retry gave up after {attempts} attempt(s): {last_error}"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + 'static> std::error::Error for RetryError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Exhausted { last_error, .. } => Some(last_error),
        }
    }
}

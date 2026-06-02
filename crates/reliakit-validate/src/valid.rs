use crate::Validate;
use core::ops::Deref;

/// A value that has been successfully validated.
///
/// `Valid<T>` is a zero-cost wrapper that carries proof of validation in the
/// type system. Construction requires the value to pass [`Validate::validate`].
/// Once inside `Valid<T>`, the value is considered correct by the validation
/// rules of `T`.
///
/// # Examples
///
/// ```
/// use reliakit_validate::{Validate, Valid, ValidationError};
///
/// struct Age(u8);
///
/// impl Validate for Age {
///     type Error = ValidationError;
///     fn validate(&self) -> Result<(), Self::Error> {
///         if self.0 > 120 {
///             return Err(ValidationError::new("age must not exceed 120"));
///         }
///         Ok(())
///     }
/// }
///
/// let age = Valid::new(Age(25)).unwrap();
/// assert_eq!(age.0, 25);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Valid<T>(T);

impl<T: Validate> Valid<T> {
    /// Validates and wraps the value. Returns an error if validation fails.
    pub fn new(value: T) -> Result<Self, T::Error> {
        value.validate()?;
        Ok(Self(value))
    }

    /// Returns a reference to the inner value.
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Consumes the wrapper and returns the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: Validate> Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Validate + core::fmt::Display> core::fmt::Display for Valid<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::Valid;
    use crate::{Validate, ValidationError};

    struct MinAge(u8);

    impl Validate for MinAge {
        type Error = ValidationError;
        fn validate(&self) -> Result<(), Self::Error> {
            if self.0 < 18 {
                return Err(ValidationError::new("must be at least 18"));
            }
            Ok(())
        }
    }

    #[test]
    fn valid_new_accepts_valid_value() {
        let v = Valid::new(MinAge(18)).unwrap();
        assert_eq!(v.get().0, 18);
    }

    #[test]
    fn valid_new_rejects_invalid_value() {
        assert!(Valid::new(MinAge(17)).is_err());
    }

    #[test]
    fn valid_into_inner() {
        let v = Valid::new(MinAge(25)).unwrap();
        assert_eq!(v.into_inner().0, 25);
    }

    #[test]
    fn valid_deref() {
        let v = Valid::new(MinAge(30)).unwrap();
        assert_eq!((*v).0, 30);
    }

    #[test]
    fn valid_get() {
        let v = Valid::new(MinAge(21)).unwrap();
        assert_eq!(v.get().0, 21);
    }
}

use crate::{CollectionError, CollectionResult};
use alloc::vec::Vec;
use core::ops::Deref;

/// Owned vector constrained by inclusive element count bounds.
///
/// `BoundedVec<T, MIN, MAX>` guarantees that the number of elements is always
/// in the range `MIN..=MAX`. Mutations that would violate the bounds return a
/// [`CollectionError`] instead of panicking.
///
/// # Const parameters
///
/// - `MIN`: minimum number of elements (inclusive). Use `0` for no lower
///   bound.
/// - `MAX`: maximum number of elements (inclusive).
///
/// Construction fails with [`CollectionError::InvalidBounds`] if `MIN > MAX`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoundedVec<T, const MIN: usize, const MAX: usize>(Vec<T>);

impl<T, const MIN: usize, const MAX: usize> BoundedVec<T, MIN, MAX> {
    /// Creates a `BoundedVec` from a `Vec`.
    ///
    /// Returns an error if the bounds are invalid or the length is out of
    /// range.
    pub fn new(vec: Vec<T>) -> CollectionResult<Self> {
        if MIN > MAX {
            return Err(CollectionError::InvalidBounds { min: MIN, max: MAX });
        }
        let actual = vec.len();
        if actual < MIN {
            return Err(CollectionError::TooFew { min: MIN, actual });
        }
        if actual > MAX {
            return Err(CollectionError::TooMany { max: MAX, actual });
        }
        Ok(Self(vec))
    }

    /// Appends an element. Returns an error if the collection is already at
    /// capacity (`len == MAX`).
    pub fn push(&mut self, item: T) -> CollectionResult<()> {
        if self.0.len() >= MAX {
            return Err(CollectionError::TooMany {
                max: MAX,
                actual: self.0.len() + 1,
            });
        }
        self.0.push(item);
        Ok(())
    }

    /// Removes and returns the last element. Returns an error if removing
    /// would bring the length below `MIN`, or if the collection is empty.
    pub fn pop(&mut self) -> CollectionResult<T> {
        if self.0.is_empty() {
            return Err(CollectionError::TooFew {
                min: MIN,
                actual: 0,
            });
        }
        let after_pop = self.0.len() - 1;
        if after_pop < MIN {
            return Err(CollectionError::TooFew {
                min: MIN,
                actual: after_pop,
            });
        }
        Ok(self.0.pop().unwrap())
    }

    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the collection contains no elements.
    ///
    /// Always returns `false` when `MIN > 0`, since construction guarantees
    /// at least `MIN` elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a reference to the first element, or `None` if empty.
    pub fn first(&self) -> Option<&T> {
        self.0.first()
    }

    /// Returns a reference to the last element, or `None` if empty.
    pub fn last(&self) -> Option<&T> {
        self.0.last()
    }

    /// Returns the inner slice.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Consumes the wrapper and returns the inner `Vec`.
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Returns an iterator over the elements.
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Returns the minimum allowed number of elements.
    pub const fn min_len() -> usize {
        MIN
    }

    /// Returns the maximum allowed number of elements.
    pub const fn max_len() -> usize {
        MAX
    }
}

impl<T, const MIN: usize, const MAX: usize> Deref for BoundedVec<T, MIN, MAX> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const MIN: usize, const MAX: usize> TryFrom<Vec<T>> for BoundedVec<T, MIN, MAX> {
    type Error = CollectionError;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(vec)
    }
}

impl<T, const MIN: usize, const MAX: usize> From<BoundedVec<T, MIN, MAX>> for Vec<T> {
    fn from(value: BoundedVec<T, MIN, MAX>) -> Self {
        value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedVec;
    use crate::CollectionError;

    #[test]
    fn accepts_valid_length() {
        let v = BoundedVec::<i32, 1, 5>::new(alloc::vec![1, 2, 3]).unwrap();
        assert_eq!(v.len(), 3);
        assert!(!v.is_empty());
    }

    #[test]
    fn rejects_too_few() {
        assert_eq!(
            BoundedVec::<i32, 2, 5>::new(alloc::vec![1]).unwrap_err(),
            CollectionError::TooFew { min: 2, actual: 1 }
        );
    }

    #[test]
    fn rejects_too_many() {
        assert_eq!(
            BoundedVec::<i32, 1, 3>::new(alloc::vec![1, 2, 3, 4]).unwrap_err(),
            CollectionError::TooMany { max: 3, actual: 4 }
        );
    }

    #[test]
    fn rejects_invalid_bounds() {
        assert_eq!(
            BoundedVec::<i32, 5, 3>::new(alloc::vec![]).unwrap_err(),
            CollectionError::InvalidBounds { min: 5, max: 3 }
        );
    }

    #[test]
    fn push_within_capacity() {
        let mut v = BoundedVec::<i32, 0, 3>::new(alloc::vec![1, 2]).unwrap();
        assert!(v.push(3).is_ok());
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn push_at_capacity_returns_error() {
        let mut v = BoundedVec::<i32, 0, 2>::new(alloc::vec![1, 2]).unwrap();
        assert_eq!(
            v.push(3).unwrap_err(),
            CollectionError::TooMany { max: 2, actual: 3 }
        );
    }

    #[test]
    fn pop_above_minimum() {
        let mut v = BoundedVec::<i32, 1, 5>::new(alloc::vec![1, 2, 3]).unwrap();
        assert_eq!(v.pop().unwrap(), 3);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn pop_at_minimum_returns_error() {
        let mut v = BoundedVec::<i32, 2, 5>::new(alloc::vec![1, 2]).unwrap();
        assert!(v.pop().is_err());
    }

    #[test]
    fn first_and_last() {
        let v = BoundedVec::<i32, 1, 5>::new(alloc::vec![10, 20, 30]).unwrap();
        assert_eq!(v.first(), Some(&10));
        assert_eq!(v.last(), Some(&30));
    }

    #[test]
    fn deref_to_slice() {
        let v = BoundedVec::<i32, 1, 5>::new(alloc::vec![1, 2, 3]).unwrap();
        assert_eq!(&v[..], &[1, 2, 3]);
    }

    #[test]
    fn into_inner() {
        let v = BoundedVec::<i32, 1, 5>::new(alloc::vec![1, 2]).unwrap();
        assert_eq!(v.into_inner(), alloc::vec![1, 2]);
    }

    #[test]
    fn from_into_vec() {
        let v = BoundedVec::<i32, 1, 5>::new(alloc::vec![1, 2]).unwrap();
        let inner: alloc::vec::Vec<i32> = alloc::vec::Vec::from(v);
        assert_eq!(inner, alloc::vec![1, 2]);
    }

    #[test]
    fn try_from_vec() {
        assert!(BoundedVec::<i32, 1, 3>::try_from(alloc::vec![1]).is_ok());
        assert!(BoundedVec::<i32, 1, 3>::try_from(alloc::vec![]).is_err());
    }

    #[test]
    fn iter() {
        let v = BoundedVec::<i32, 1, 5>::new(alloc::vec![1, 2, 3]).unwrap();
        let sum: i32 = v.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn min_max_len_constants() {
        assert_eq!(BoundedVec::<i32, 2, 8>::min_len(), 2);
        assert_eq!(BoundedVec::<i32, 2, 8>::max_len(), 8);
    }

    #[test]
    fn zero_min_allows_empty() {
        let v = BoundedVec::<i32, 0, 5>::new(alloc::vec![]).unwrap();
        assert!(v.is_empty());
        assert_eq!(v.first(), None);
        assert_eq!(v.last(), None);
    }

    #[test]
    fn pop_min_zero_empty_vec_returns_error() {
        let mut v = BoundedVec::<i32, 0, 5>::new(alloc::vec![]).unwrap();
        let err = v.pop().unwrap_err();
        assert_eq!(err, CollectionError::TooFew { min: 0, actual: 0 });
    }

    #[test]
    fn pop_min_zero_nonempty_succeeds() {
        let mut v = BoundedVec::<i32, 0, 5>::new(alloc::vec![1, 2]).unwrap();
        assert_eq!(v.pop().unwrap(), 2);
        assert_eq!(v.pop().unwrap(), 1);
        assert!(v.pop().is_err());
    }

    #[test]
    fn min_equals_max_exact_size() {
        assert!(BoundedVec::<i32, 3, 3>::new(alloc::vec![1, 2, 3]).is_ok());
        assert!(BoundedVec::<i32, 3, 3>::new(alloc::vec![1, 2]).is_err());
        assert!(BoundedVec::<i32, 3, 3>::new(alloc::vec![1, 2, 3, 4]).is_err());
    }
}

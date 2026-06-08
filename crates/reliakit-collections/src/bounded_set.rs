use crate::{CollectionError, CollectionResult};
use alloc::vec::Vec;
use core::ops::Deref;

/// Owned set of unique elements constrained by inclusive count bounds.
///
/// `BoundedSet<T, MIN, MAX>` guarantees that the number of elements is always in
/// the range `MIN..=MAX` and that every element is unique. It is backed by a
/// `Vec<T>` in insertion order, so iteration is deterministic and pulls in no
/// hashing or ordering machinery — membership tests are a linear scan, which is
/// fine for the small, bounded sizes this type is meant for.
///
/// Mutations that would violate the bounds return a [`CollectionError`] instead
/// of panicking. Construction rejects duplicate elements with
/// [`CollectionError::Duplicate`].
///
/// # Const parameters
///
/// - `MIN`: minimum number of elements (inclusive). Use `0` for no lower bound.
/// - `MAX`: maximum number of elements (inclusive).
///
/// Construction fails with [`CollectionError::InvalidBounds`] if `MIN > MAX`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoundedSet<T, const MIN: usize, const MAX: usize>(Vec<T>);

impl<T, const MIN: usize, const MAX: usize> BoundedSet<T, MIN, MAX> {
    /// Returns the number of elements.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// Always returns `false` when `MIN > 0`, since construction guarantees at
    /// least `MIN` elements.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the elements in insertion order.
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Returns the elements as a slice in insertion order.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Consumes the wrapper and returns the inner `Vec<T>`.
    pub fn into_inner(self) -> Vec<T> {
        self.0
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

impl<T: PartialEq, const MIN: usize, const MAX: usize> BoundedSet<T, MIN, MAX> {
    /// Creates a `BoundedSet` from a vector of elements.
    ///
    /// Returns an error if the bounds are invalid (`MIN > MAX`), the element
    /// count is out of range, or the vector contains duplicates.
    pub fn new(items: Vec<T>) -> CollectionResult<Self> {
        if MIN > MAX {
            return Err(CollectionError::InvalidBounds { min: MIN, max: MAX });
        }
        let actual = items.len();
        if actual < MIN {
            return Err(CollectionError::TooFew { min: MIN, actual });
        }
        if actual > MAX {
            return Err(CollectionError::TooMany { max: MAX, actual });
        }
        for i in 0..items.len() {
            for j in (i + 1)..items.len() {
                if items[i] == items[j] {
                    return Err(CollectionError::Duplicate);
                }
            }
        }
        Ok(Self(items))
    }

    /// Inserts an element.
    ///
    /// Returns `Ok(true)` if the element was added, or `Ok(false)` if it was
    /// already present (no change). Returns [`CollectionError::TooMany`] if the
    /// element is new and the set is already at capacity (`len == MAX`), leaving
    /// the set unchanged.
    pub fn insert(&mut self, item: T) -> CollectionResult<bool> {
        if self.0.contains(&item) {
            return Ok(false);
        }
        if self.0.len() >= MAX {
            return Err(CollectionError::TooMany {
                max: MAX,
                actual: self.0.len().saturating_add(1),
            });
        }
        self.0.push(item);
        Ok(true)
    }

    /// Removes an element.
    ///
    /// Returns `Ok(true)` if the element was present and removed, or `Ok(false)`
    /// if it was absent (no change). Returns [`CollectionError::TooFew`] if
    /// removing would bring the count below `MIN`, leaving the set unchanged.
    pub fn remove(&mut self, item: &T) -> CollectionResult<bool> {
        let Some(idx) = self.0.iter().position(|existing| existing == item) else {
            return Ok(false);
        };
        let after_remove = self.0.len() - 1;
        if after_remove < MIN {
            return Err(CollectionError::TooFew {
                min: MIN,
                actual: after_remove,
            });
        }
        self.0.remove(idx);
        Ok(true)
    }

    /// Returns `true` if the set contains `item`.
    pub fn contains(&self, item: &T) -> bool {
        self.0.contains(item)
    }
}

impl<T, const MIN: usize, const MAX: usize> Deref for BoundedSet<T, MIN, MAX> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: PartialEq, const MIN: usize, const MAX: usize> TryFrom<Vec<T>> for BoundedSet<T, MIN, MAX> {
    type Error = CollectionError;

    fn try_from(items: Vec<T>) -> Result<Self, Self::Error> {
        Self::new(items)
    }
}

impl<T, const MIN: usize, const MAX: usize> From<BoundedSet<T, MIN, MAX>> for Vec<T> {
    fn from(value: BoundedSet<T, MIN, MAX>) -> Self {
        value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedSet;
    use crate::CollectionError;

    fn set_5() -> BoundedSet<i32, 0, 5> {
        BoundedSet::new(alloc::vec![1, 2, 3]).unwrap()
    }

    #[test]
    fn accepts_unique_elements() {
        let s = set_5();
        assert_eq!(s.len(), 3);
        assert!(!s.is_empty());
        assert!(s.contains(&2));
        assert!(!s.contains(&9));
    }

    #[test]
    fn rejects_duplicates() {
        assert_eq!(
            BoundedSet::<i32, 0, 5>::new(alloc::vec![1, 2, 2]).unwrap_err(),
            CollectionError::Duplicate
        );
    }

    #[test]
    fn rejects_too_few() {
        assert_eq!(
            BoundedSet::<i32, 2, 5>::new(alloc::vec![1]).unwrap_err(),
            CollectionError::TooFew { min: 2, actual: 1 }
        );
    }

    #[test]
    fn rejects_too_many() {
        assert_eq!(
            BoundedSet::<i32, 0, 2>::new(alloc::vec![1, 2, 3]).unwrap_err(),
            CollectionError::TooMany { max: 2, actual: 3 }
        );
    }

    #[test]
    fn rejects_invalid_bounds() {
        assert_eq!(
            BoundedSet::<i32, 5, 3>::new(alloc::vec![]).unwrap_err(),
            CollectionError::InvalidBounds { min: 5, max: 3 }
        );
    }

    #[test]
    fn insert_new_element() {
        let mut s = set_5();
        assert!(s.insert(4).unwrap());
        assert_eq!(s.len(), 4);
        assert!(s.contains(&4));
    }

    #[test]
    fn insert_existing_element_is_noop() {
        let mut s = set_5();
        assert!(!s.insert(2).unwrap());
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn insert_at_capacity_new_element_errors() {
        let mut s = BoundedSet::<i32, 0, 3>::new(alloc::vec![1, 2, 3]).unwrap();
        assert_eq!(
            s.insert(4).unwrap_err(),
            CollectionError::TooMany { max: 3, actual: 4 }
        );
        // Re-inserting an existing element at capacity is still a no-op Ok(false).
        assert!(!s.insert(1).unwrap());
    }

    #[test]
    fn remove_present_element() {
        let mut s = set_5();
        assert!(s.remove(&2).unwrap());
        assert_eq!(s.len(), 2);
        assert!(!s.contains(&2));
    }

    #[test]
    fn remove_absent_element_is_noop() {
        let mut s = set_5();
        assert!(!s.remove(&9).unwrap());
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn remove_below_minimum_errors() {
        let mut s = BoundedSet::<i32, 2, 5>::new(alloc::vec![1, 2]).unwrap();
        assert_eq!(
            s.remove(&1).unwrap_err(),
            CollectionError::TooFew { min: 2, actual: 1 }
        );
        assert_eq!(s.len(), 2); // unchanged
    }

    #[test]
    fn iter_in_insertion_order() {
        let s = set_5();
        let collected: alloc::vec::Vec<&i32> = s.iter().collect();
        assert_eq!(collected, alloc::vec![&1, &2, &3]);
    }

    #[test]
    fn deref_to_slice() {
        let s = set_5();
        assert_eq!(&s[..], &[1, 2, 3]);
    }

    #[test]
    fn into_inner_and_from() {
        let s = set_5();
        let inner: alloc::vec::Vec<i32> = alloc::vec::Vec::from(s);
        assert_eq!(inner, alloc::vec![1, 2, 3]);
    }

    #[test]
    fn try_from_vec() {
        assert!(BoundedSet::<i32, 1, 3>::try_from(alloc::vec![1, 2]).is_ok());
        assert_eq!(
            BoundedSet::<i32, 1, 3>::try_from(alloc::vec![1, 1]).unwrap_err(),
            CollectionError::Duplicate
        );
    }

    #[test]
    fn min_equals_max_exact_size() {
        assert!(BoundedSet::<i32, 3, 3>::new(alloc::vec![1, 2, 3]).is_ok());
        assert!(BoundedSet::<i32, 3, 3>::new(alloc::vec![1, 2]).is_err());
    }

    #[test]
    fn min_max_len_constants() {
        assert_eq!(BoundedSet::<i32, 2, 8>::min_len(), 2);
        assert_eq!(BoundedSet::<i32, 2, 8>::max_len(), 8);
    }
}

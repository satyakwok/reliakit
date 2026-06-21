use crate::{CollectionError, CollectionResult};
use alloc::vec::Vec;
use core::ops::{Bound, Deref, RangeBounds};

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
                actual: self.0.len().saturating_add(1),
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

    /// Retains only the elements for which `op` returns `true`.
    ///
    /// Counts survivors first; if keeping them would leave fewer than `MIN`
    /// elements, returns [`CollectionError::TooFew`] and leaves the collection unchanged.
    /// *Note: The closure `op` is evaluated twice per element to guarantee atomicity.*
    pub fn retain<F>(&mut self, op: F) -> CollectionResult<()>
    where
        F: Fn(&T) -> bool,
    {
        let actual = self.0.iter().filter(|v| op(v)).count();

        if actual < MIN {
            return Err(CollectionError::TooFew { min: MIN, actual });
        }

        self.0.retain(op);
        Ok(())
    }
    /// Removes the specified range from the collection and returns the drained elements.
    ///
    /// Returns [`CollectionError::InvalidRange`] if the range is malformed, or
    /// [`CollectionError::TooFew`] if draining would drop below `MIN`. Collection unchanged on either error.
    pub fn drain<R>(&mut self, range: R) -> CollectionResult<Vec<T>>
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.saturating_add(1),
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n.saturating_add(1),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => self.0.len(),
        };

        if start > end || end > self.0.len() {
            return Err(CollectionError::InvalidRange {
                start,
                end,
                len: self.0.len(),
            });
        }

        let drained = end - start;
        let remaining = self.0.len() - drained;

        if remaining < MIN {
            return Err(CollectionError::TooFew {
                min: MIN,
                actual: remaining,
            });
        }
        Ok(self.0.drain(start..end).collect())
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

    #[cfg(test)]
    mod retain_tests {
        use super::*;

        #[test]
        fn retain_fails_when_result_would_be_below_min() {
            let mut collection: BoundedVec<i32, 2, 6> =
                BoundedVec::try_from(vec![1, 2, 3]).unwrap();

            let result = collection.retain(|x| *x == 1);

            assert!(
                matches!(result, Err(CollectionError::TooFew { min: 2, actual: 1 })),
                "expected TooFew {{ min: 2, actual: 1 }}, got: {:?}",
                result
            );

            assert_eq!(
                collection.as_slice(),
                &[1, 2, 3],
                "collection must be unchanged after a failed retain"
            );
        }
        #[test]
        fn retain_succeeds_when_result_stays_above_min() {
            let mut collection: BoundedVec<i32, 2, 6> =
                BoundedVec::try_from(vec![1, 2, 3, 4]).unwrap();

            let result = collection.retain(|x| *x >= 2);

            assert!(result.is_ok(), "expected Ok, got: {:?}", result);
            assert_eq!(
                collection.as_slice(),
                &[2, 3, 4],
                "collection should contain only retained items"
            );
        }

        #[test]
        fn retain_succeeds_at_exactly_min_boundary() {
            let mut collection: BoundedVec<i32, 2, 6> =
                BoundedVec::try_from(vec![1, 2, 3]).unwrap();

            let result = collection.retain(|x| *x >= 2);

            assert!(
                result.is_ok(),
                "expected Ok at exactly MIN, got: {:?}",
                result
            );
            assert_eq!(collection.as_slice(), &[2, 3]);
        }
    }

    #[cfg(test)]
    mod drain_tests {
        use super::*;

        // ---- Happy path -------------------------------------------------------

        #[test]
        fn drain_range_within_bounds_succeeds() {
            // Start with 5 elements, MIN=2. Drain 2 elements, leaving 3 (>= MIN).
            let mut v = BoundedVec::<i32, 2, 10>::new(vec![1, 2, 3, 4, 5]).unwrap();
            let drained = v.drain(1..3).unwrap();
            assert_eq!(drained, vec![2, 3]);
            assert_eq!(v.as_slice(), &[1, 4, 5]);
        }

        #[test]
        fn drain_leaving_exactly_min_succeeds() {
            // Boundary: drain just enough that remaining equals MIN exactly.
            let mut v = BoundedVec::<i32, 2, 10>::new(vec![1, 2, 3, 4]).unwrap();
            let drained = v.drain(0..2).unwrap();
            assert_eq!(drained, vec![1, 2]);
            assert_eq!(v.as_slice(), &[3, 4]);
        }

        #[test]
        fn drain_inclusive_range_succeeds() {
            // RangeInclusive (1..=2) should work just like 1..3.
            let mut v = BoundedVec::<i32, 1, 10>::new(vec![1, 2, 3, 4, 5]).unwrap();
            let drained = v.drain(1..=2).unwrap();
            assert_eq!(drained, vec![2, 3]);
            assert_eq!(v.as_slice(), &[1, 4, 5]);
        }

        #[test]
        fn drain_unbounded_start_succeeds() {
            // RangeTo (..3) drains from beginning.
            let mut v = BoundedVec::<i32, 1, 10>::new(vec![1, 2, 3, 4, 5]).unwrap();
            let drained = v.drain(..3).unwrap();
            assert_eq!(drained, vec![1, 2, 3]);
            assert_eq!(v.as_slice(), &[4, 5]);
        }

        #[test]
        fn drain_unbounded_end_succeeds() {
            // RangeFrom (2..) drains from index to end.
            let mut v = BoundedVec::<i32, 1, 10>::new(vec![1, 2, 3, 4, 5]).unwrap();
            let drained = v.drain(2..).unwrap();
            assert_eq!(drained, vec![3, 4, 5]);
            assert_eq!(v.as_slice(), &[1, 2]);
        }

        #[test]
        fn drain_empty_range_succeeds() {
            // Empty range (2..2) drains nothing — no length change, empty result.
            let mut v = BoundedVec::<i32, 2, 10>::new(vec![1, 2, 3, 4]).unwrap();
            let drained = v.drain(2..2).unwrap();
            assert!(drained.is_empty());
            assert_eq!(v.as_slice(), &[1, 2, 3, 4]);
        }

        // ---- Failure path -----------------------------------------------------

        #[test]
        fn drain_below_min_fails() {
            // Start with 4 elements, MIN=3. Drain 2 → would leave 2 (< MIN).
            let mut v = BoundedVec::<i32, 3, 10>::new(vec![1, 2, 3, 4]).unwrap();
            let result = v.drain(0..2);
            assert!(matches!(
                result,
                Err(CollectionError::TooFew { min: 3, actual: 2 })
            ));
        }

        #[test]
        fn drain_full_range_fails_when_min_nonzero() {
            // drain(..) with MIN > 0 must always fail — would leave the vec empty.
            let mut v = BoundedVec::<i32, 1, 10>::new(vec![1, 2, 3]).unwrap();
            let result = v.drain(..);
            assert!(matches!(
                result,
                Err(CollectionError::TooFew { min: 1, actual: 0 })
            ));
        }

        #[test]
        fn drain_failure_leaves_collection_unchanged() {
            // On failure, the original vec must not be mutated.
            let mut v = BoundedVec::<i32, 3, 10>::new(vec![1, 2, 3, 4]).unwrap();
            let _ = v.drain(0..2); // would fail (4 - 2 = 2, below MIN=3)
            assert_eq!(
                v.as_slice(),
                &[1, 2, 3, 4],
                "vec must be unchanged after failed drain"
            );
        }

        #[test]
        fn drain_with_end_past_length_fails_with_invalid_range() {
            let mut v = BoundedVec::<i32, 0, 10>::new(vec![1, 2, 3]).unwrap();
            let result = v.drain(0..10);
            assert!(matches!(
                result,
                Err(CollectionError::InvalidRange {
                    start: 0,
                    end: 10,
                    len: 3
                })
            ));
        }

        #[test]
        fn drain_invalid_range_leaves_collection_unchanged() {
            let mut v = BoundedVec::<i32, 0, 10>::new(vec![1, 2, 3, 4, 5]).unwrap();
            let _ = v.drain(0..10); // InvalidRange
            assert_eq!(v.as_slice(), &[1, 2, 3, 4, 5]);
        }

        #[test]
        fn drain_reversed_dynamic_range_fails_gracefully() {
            let mut v = BoundedVec::<i32, 0, 10>::new(vec![1, 2, 3, 4, 5]).unwrap();
            let start_index = 4;
            let end_index = 2;
            let result = v.drain(start_index..end_index);
            assert!(matches!(
                result,
                Err(CollectionError::InvalidRange {
                    start: 4,
                    end: 2,
                    len: 5
                })
            ));
            assert_eq!(v.as_slice(), &[1, 2, 3, 4, 5]);
        }
    }
    #[test]
    fn drain_range_inclusive_max_fails_gracefully() {
        let mut v = BoundedVec::<i32, 0, 10>::new(vec![1, 2, 3]).unwrap();
        let result = v.drain(0..=usize::MAX);
        assert!(matches!(
            result,
            Err(CollectionError::InvalidRange {
                start: 0,
                end: usize::MAX,
                len: 3
            })
        ));
    }
}

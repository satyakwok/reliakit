use crate::{CollectionError, CollectionResult};
use alloc::vec::Vec;
use core::mem;

/// Owned key-value map constrained by inclusive entry-count bounds.
///
/// `BoundedMap<K, V, MIN, MAX>` guarantees that the number of entries is always
/// in the range `MIN..=MAX` and that every key is unique. It is backed by a
/// `Vec<(K, V)>` in insertion order, so iteration is deterministic and pulls in
/// no hashing or ordering machinery, lookups are a linear scan, which is fine
/// for the small, bounded sizes this type is meant for.
///
/// Mutations that would violate the bounds return a [`CollectionError`] instead
/// of panicking. Construction rejects duplicate keys with
/// [`CollectionError::Duplicate`].
///
/// # Const parameters
///
/// - `MIN`: minimum number of entries (inclusive). Use `0` for no lower bound.
/// - `MAX`: maximum number of entries (inclusive).
///
/// Construction fails with [`CollectionError::InvalidBounds`] if `MIN > MAX`.
///
/// # Equality
///
/// Equality is order-sensitive: two maps with the same entries inserted in a
/// different order are not equal, matching the insertion-order semantics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoundedMap<K, V, const MIN: usize, const MAX: usize>(Vec<(K, V)>);

impl<K, V, const MIN: usize, const MAX: usize> BoundedMap<K, V, MIN, MAX> {
    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the map contains no entries.
    ///
    /// Always returns `false` when `MIN > 0`, since construction guarantees at
    /// least `MIN` entries.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an iterator over the `(K, V)` entries in insertion order.
    pub fn iter(&self) -> core::slice::Iter<'_, (K, V)> {
        self.0.iter()
    }

    /// Returns an iterator over the keys in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.0.iter().map(|(k, _)| k)
    }

    /// Returns an iterator over the values in insertion order.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.0.iter().map(|(_, v)| v)
    }

    /// Returns the entries as a slice in insertion order.
    pub fn as_slice(&self) -> &[(K, V)] {
        &self.0
    }

    /// Consumes the wrapper and returns the inner `Vec<(K, V)>`.
    pub fn into_inner(self) -> Vec<(K, V)> {
        self.0
    }

    /// Returns the minimum allowed number of entries.
    pub const fn min_len() -> usize {
        MIN
    }

    /// Returns the maximum allowed number of entries.
    pub const fn max_len() -> usize {
        MAX
    }
}

impl<K: PartialEq, V, const MIN: usize, const MAX: usize> BoundedMap<K, V, MIN, MAX> {
    /// Creates a `BoundedMap` from a vector of entries.
    ///
    /// Returns an error if the bounds are invalid (`MIN > MAX`), the entry count
    /// is out of range, or two entries share a key.
    pub fn new(entries: Vec<(K, V)>) -> CollectionResult<Self> {
        if MIN > MAX {
            return Err(CollectionError::InvalidBounds { min: MIN, max: MAX });
        }
        let actual = entries.len();
        if actual < MIN {
            return Err(CollectionError::TooFew { min: MIN, actual });
        }
        if actual > MAX {
            return Err(CollectionError::TooMany { max: MAX, actual });
        }
        for i in 0..entries.len() {
            for j in (i + 1)..entries.len() {
                if entries[i].0 == entries[j].0 {
                    return Err(CollectionError::Duplicate);
                }
            }
        }
        Ok(Self(entries))
    }

    /// Inserts a key-value pair.
    ///
    /// If the key already exists, its value is replaced and the old value is
    /// returned as `Ok(Some(old))` (the entry count does not change). If the key
    /// is new, it is appended and `Ok(None)` is returned, unless the map is
    /// already at capacity (`len == MAX`), in which case
    /// [`CollectionError::TooMany`] is returned and nothing changes.
    pub fn insert(&mut self, key: K, value: V) -> CollectionResult<Option<V>> {
        if let Some(entry) = self.0.iter_mut().find(|entry| entry.0 == key) {
            return Ok(Some(mem::replace(&mut entry.1, value)));
        }
        if self.0.len() >= MAX {
            return Err(CollectionError::TooMany {
                max: MAX,
                actual: self.0.len().saturating_add(1),
            });
        }
        self.0.push((key, value));
        Ok(None)
    }

    /// Removes the entry for `key`, returning its value.
    ///
    /// Returns `Ok(None)` if the key is absent (no change). Returns
    /// [`CollectionError::TooFew`] if removing would bring the entry count below
    /// `MIN`, leaving the map unchanged.
    pub fn remove(&mut self, key: &K) -> CollectionResult<Option<V>> {
        let Some(idx) = self.0.iter().position(|entry| &entry.0 == key) else {
            return Ok(None);
        };
        let after_remove = self.0.len() - 1;
        if after_remove < MIN {
            return Err(CollectionError::TooFew {
                min: MIN,
                actual: after_remove,
            });
        }
        let (_, value) = self.0.remove(idx);
        Ok(Some(value))
    }

    /// Retains only the entries for which `op` returns `true` on the `key` , `value`.
    ///
    /// Counts survivors first; if keeping them would leave fewer than `MIN`
    /// entries, returns [`CollectionError::TooFew`] and leaves the map unchanged.
    /// *Note: The closure `op` is evaluated twice per element to guarantee atomicity.*
    pub fn retain<F>(&mut self, op: F) -> CollectionResult<()>
    where
        F: Fn(&K, &V) -> bool,
    {
        let actual = self.0.iter().filter(|entry| op(&entry.0, &entry.1)).count();

        if actual < MIN {
            return Err(CollectionError::TooFew { min: MIN, actual });
        }

        self.0.retain(|entry| op(&entry.0, &entry.1));
        Ok(())
    }

    /// Returns a reference to the value for `key`, or `None` if absent.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0
            .iter()
            .find(|entry| &entry.0 == key)
            .map(|entry| &entry.1)
    }

    /// Returns a mutable reference to the value for `key`, or `None` if absent.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0
            .iter_mut()
            .find(|entry| &entry.0 == key)
            .map(|entry| &mut entry.1)
    }

    /// Returns `true` if the map contains `key`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.0.iter().any(|entry| &entry.0 == key)
    }
}

/// `drain` is only available when `MIN == 0`. With `MIN > 0`, draining
/// would always violate the bounds, so the operation is excluded by the
/// type system entirely.
impl<K, V, const MAX: usize> BoundedMap<K, V, 0, MAX> {
    /// Removes all entries from the map and returns them as a `Vec<(K, V)>`,
    /// preserving insertion order. The map's allocation is retained.
    pub fn drain(&mut self) -> Vec<(K, V)> {
        self.0.drain(..).collect()
    }
}

impl<K: PartialEq, V, const MIN: usize, const MAX: usize> TryFrom<Vec<(K, V)>>
    for BoundedMap<K, V, MIN, MAX>
{
    type Error = CollectionError;

    fn try_from(entries: Vec<(K, V)>) -> Result<Self, Self::Error> {
        Self::new(entries)
    }
}

impl<K, V, const MIN: usize, const MAX: usize> From<BoundedMap<K, V, MIN, MAX>> for Vec<(K, V)> {
    fn from(value: BoundedMap<K, V, MIN, MAX>) -> Self {
        value.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedMap;
    use crate::CollectionError;

    fn map_3() -> BoundedMap<&'static str, i32, 0, 3> {
        BoundedMap::new(alloc::vec![("a", 1), ("b", 2)]).unwrap()
    }

    #[test]
    fn accepts_valid_entries() {
        let m = map_3();
        assert_eq!(m.len(), 2);
        assert!(!m.is_empty());
        assert_eq!(m.get(&"a"), Some(&1));
        assert_eq!(m.get(&"b"), Some(&2));
        assert_eq!(m.get(&"z"), None);
    }

    #[test]
    fn rejects_duplicate_keys() {
        assert_eq!(
            BoundedMap::<&str, i32, 0, 5>::new(alloc::vec![("a", 1), ("a", 2)]).unwrap_err(),
            CollectionError::Duplicate
        );
    }

    #[test]
    fn rejects_too_few() {
        assert_eq!(
            BoundedMap::<&str, i32, 2, 5>::new(alloc::vec![("a", 1)]).unwrap_err(),
            CollectionError::TooFew { min: 2, actual: 1 }
        );
    }

    #[test]
    fn rejects_too_many() {
        assert_eq!(
            BoundedMap::<&str, i32, 0, 1>::new(alloc::vec![("a", 1), ("b", 2)]).unwrap_err(),
            CollectionError::TooMany { max: 1, actual: 2 }
        );
    }

    #[test]
    fn rejects_invalid_bounds() {
        assert_eq!(
            BoundedMap::<&str, i32, 5, 3>::new(alloc::vec![]).unwrap_err(),
            CollectionError::InvalidBounds { min: 5, max: 3 }
        );
    }

    #[test]
    fn insert_new_key_appends() {
        let mut m = map_3();
        assert_eq!(m.insert("c", 3).unwrap(), None);
        assert_eq!(m.len(), 3);
        assert_eq!(m.get(&"c"), Some(&3));
    }

    #[test]
    fn insert_existing_key_replaces_value() {
        let mut m = map_3();
        assert_eq!(m.insert("a", 99).unwrap(), Some(1));
        assert_eq!(m.len(), 2); // count unchanged
        assert_eq!(m.get(&"a"), Some(&99));
    }

    #[test]
    fn insert_at_capacity_new_key_errors() {
        let mut m = BoundedMap::<&str, i32, 0, 2>::new(alloc::vec![("a", 1), ("b", 2)]).unwrap();
        assert_eq!(
            m.insert("c", 3).unwrap_err(),
            CollectionError::TooMany { max: 2, actual: 3 }
        );
        // Replacing an existing key still works at capacity.
        assert_eq!(m.insert("a", 10).unwrap(), Some(1));
    }

    #[test]
    fn remove_existing_key_returns_value() {
        let mut m = map_3();
        assert_eq!(m.remove(&"a").unwrap(), Some(1));
        assert_eq!(m.len(), 1);
        assert_eq!(m.get(&"a"), None);
    }

    #[test]
    fn remove_absent_key_is_noop() {
        let mut m = map_3();
        assert_eq!(m.remove(&"z").unwrap(), None);
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn remove_below_minimum_errors() {
        let mut m = BoundedMap::<&str, i32, 2, 5>::new(alloc::vec![("a", 1), ("b", 2)]).unwrap();
        assert_eq!(
            m.remove(&"a").unwrap_err(),
            CollectionError::TooFew { min: 2, actual: 1 }
        );
        assert_eq!(m.len(), 2); // unchanged
    }

    #[test]
    fn get_mut_updates_in_place() {
        let mut m = map_3();
        *m.get_mut(&"b").unwrap() += 10;
        assert_eq!(m.get(&"b"), Some(&12));
        assert!(m.get_mut(&"z").is_none());
    }

    #[test]
    fn contains_key_works() {
        let m = map_3();
        assert!(m.contains_key(&"a"));
        assert!(!m.contains_key(&"z"));
    }

    #[test]
    fn keys_values_iter_in_insertion_order() {
        let m = map_3();
        let keys: alloc::vec::Vec<&&str> = m.keys().collect();
        assert_eq!(keys, alloc::vec![&"a", &"b"]);
        let values: alloc::vec::Vec<&i32> = m.values().collect();
        assert_eq!(values, alloc::vec![&1, &2]);
        let entries: alloc::vec::Vec<&(&str, i32)> = m.iter().collect();
        assert_eq!(entries, alloc::vec![&("a", 1), &("b", 2)]);
    }

    #[test]
    fn into_inner_and_from() {
        let m = map_3();
        let inner: alloc::vec::Vec<(&str, i32)> = m.into_inner();
        assert_eq!(inner, alloc::vec![("a", 1), ("b", 2)]);
    }

    #[test]
    fn try_from_vec() {
        assert!(BoundedMap::<&str, i32, 1, 3>::try_from(alloc::vec![("a", 1)]).is_ok());
        assert_eq!(
            BoundedMap::<&str, i32, 1, 3>::try_from(alloc::vec![("a", 1), ("a", 2)]).unwrap_err(),
            CollectionError::Duplicate
        );
    }

    #[test]
    fn min_equals_max_exact_size() {
        assert!(BoundedMap::<&str, i32, 2, 2>::new(alloc::vec![("a", 1), ("b", 2)]).is_ok());
        assert!(BoundedMap::<&str, i32, 2, 2>::new(alloc::vec![("a", 1)]).is_err());
    }

    #[test]
    fn equality_is_order_sensitive() {
        let a = BoundedMap::<&str, i32, 0, 3>::new(alloc::vec![("a", 1), ("b", 2)]).unwrap();
        let b = BoundedMap::<&str, i32, 0, 3>::new(alloc::vec![("b", 2), ("a", 1)]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn min_max_len_constants() {
        assert_eq!(BoundedMap::<&str, i32, 2, 8>::min_len(), 2);
        assert_eq!(BoundedMap::<&str, i32, 2, 8>::max_len(), 8);
    }

    #[cfg(test)]
    mod map_retain_tests {
        use super::*;

        // ---- Happy path -------------------------------------------------------

        #[test]
        fn map_retain_succeeds_above_min() {
            let mut m: BoundedMap<&str, i32, 2, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2), ("c", 3), ("d", 4)]).unwrap();
            m.retain(|_k, v| *v >= 2).unwrap();
            assert_eq!(m.as_slice(), &[("b", 2), ("c", 3), ("d", 4)]);
        }

        #[test]
        fn map_retain_succeeds_at_min_boundary() {
            // Boundary: predicate keeps exactly MIN entries.
            let mut m: BoundedMap<&str, i32, 2, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2), ("c", 3)]).unwrap();
            m.retain(|_k, v| *v >= 2).unwrap();
            assert_eq!(m.as_slice(), &[("b", 2), ("c", 3)]);
        }

        #[test]
        fn map_retain_can_filter_by_key() {
            // Predicate uses only the key — proves key is accessible.
            let mut m: BoundedMap<&str, i32, 1, 10> =
                BoundedMap::new(vec![("apple", 1), ("banana", 2), ("apricot", 3)]).unwrap();
            m.retain(|k, _v| k.starts_with('a')).unwrap();
            assert_eq!(m.as_slice(), &[("apple", 1), ("apricot", 3)]);
        }

        #[test]
        fn map_retain_can_filter_by_both_key_and_value() {
            // Predicate uses both — proves both are accessible.
            let mut m: BoundedMap<&str, i32, 1, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2), ("c", 3), ("d", 4)]).unwrap();
            m.retain(|k, v| k != &"b" && *v >= 2).unwrap();
            assert_eq!(m.as_slice(), &[("c", 3), ("d", 4)]);
        }

        #[test]
        fn map_retain_with_always_true_predicate_is_noop() {
            let mut m: BoundedMap<&str, i32, 1, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2)]).unwrap();
            m.retain(|_, _| true).unwrap();
            assert_eq!(m.as_slice(), &[("a", 1), ("b", 2)]);
        }

        // ---- Failure path -----------------------------------------------------

        #[test]
        fn map_retain_fails_below_min() {
            let mut m: BoundedMap<&str, i32, 2, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2), ("c", 3)]).unwrap();
            let result = m.retain(|_, v| *v == 1);
            assert!(matches!(
                result,
                Err(CollectionError::TooFew { min: 2, actual: 1 })
            ));
            assert_eq!(
                m.as_slice(),
                &[("a", 1), ("b", 2), ("c", 3)],
                "map unchanged after failed retain"
            );
        }

        #[test]
        fn map_retain_fails_when_predicate_keeps_nothing() {
            let mut m: BoundedMap<&str, i32, 1, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2)]).unwrap();
            let result = m.retain(|_, _| false);
            assert!(matches!(
                result,
                Err(CollectionError::TooFew { min: 1, actual: 0 })
            ));
            assert_eq!(m.as_slice(), &[("a", 1), ("b", 2)]);
        }
    }

    #[cfg(test)]
    mod map_drain_tests {
        use super::*;

        #[test]
        fn map_drain_returns_all_entries_and_empties() {
            let mut m: BoundedMap<&str, i32, 0, 10> =
                BoundedMap::new(vec![("a", 1), ("b", 2), ("c", 3)]).unwrap();
            let drained = m.drain();
            assert_eq!(drained, vec![("a", 1), ("b", 2), ("c", 3)]);
            assert!(m.is_empty());
        }

        #[test]
        fn map_drain_preserves_insertion_order() {
            // Insertion order should be respected in the drained Vec.
            let mut m: BoundedMap<&str, i32, 0, 10> = BoundedMap::new(vec![]).unwrap();
            m.insert("z", 1).unwrap();
            m.insert("a", 2).unwrap();
            m.insert("m", 3).unwrap();

            let drained = m.drain();
            assert_eq!(drained, vec![("z", 1), ("a", 2), ("m", 3)]);
        }

        #[test]
        fn map_drain_on_empty_map_returns_empty_vec() {
            let mut m: BoundedMap<&str, i32, 0, 10> = BoundedMap::new(vec![]).unwrap();
            let drained = m.drain();
            assert!(drained.is_empty());
            assert!(m.is_empty());
        }

        #[test]
        fn map_drain_can_be_called_multiple_times() {
            // After drain, the map is reusable — can be filled and drained again.
            let mut m: BoundedMap<&str, i32, 0, 10> = BoundedMap::new(vec![("a", 1)]).unwrap();

            let first = m.drain();
            assert_eq!(first, vec![("a", 1)]);
            assert!(m.is_empty());

            m.insert("b", 2).unwrap();
            let second = m.drain();
            assert_eq!(second, vec![("b", 2)]);
            assert!(m.is_empty());
        }
    }
}

//! A bounded, drop-oldest ring buffer.

use alloc::collections::VecDeque;

use crate::error::{CollectionError, CollectionResult};

/// A fixed-capacity circular buffer that overwrites the oldest element when
/// full.
///
/// Unlike [`BoundedVec`](crate::BoundedVec), pushing onto a full `RingBuffer`
/// never fails: it evicts and returns the oldest element instead. That makes it
/// a good fit for rolling windows such as recent log lines, the last *N*
/// samples, or event histories, where keeping the newest data and bounding
/// memory matters more than keeping everything.
///
/// Capacity is fixed at construction and must be greater than zero. Order is
/// preserved oldest-to-newest, which is the order [`iter`](Self::iter) yields.
///
/// # Examples
///
/// ```
/// use reliakit_collections::RingBuffer;
///
/// let mut last3 = RingBuffer::new(3).unwrap();
/// assert_eq!(last3.push(1), None);
/// assert_eq!(last3.push(2), None);
/// assert_eq!(last3.push(3), None);
/// assert!(last3.is_full());
///
/// // Pushing onto a full buffer evicts the oldest element.
/// assert_eq!(last3.push(4), Some(1));
/// assert_eq!(last3.iter().copied().collect::<Vec<_>>(), [2, 3, 4]);
/// ```
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    inner: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    /// Creates an empty ring buffer with the given `capacity`.
    ///
    /// Returns [`CollectionError::ZeroCapacity`] if `capacity` is `0`.
    pub fn new(capacity: usize) -> CollectionResult<Self> {
        if capacity == 0 {
            return Err(CollectionError::ZeroCapacity);
        }
        Ok(Self {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        })
    }

    /// Appends `item` as the newest element.
    ///
    /// If the buffer is full, the oldest element is evicted and returned;
    /// otherwise returns `None`.
    pub fn push(&mut self, item: T) -> Option<T> {
        let evicted = if self.inner.len() == self.capacity {
            self.inner.pop_front()
        } else {
            None
        };
        self.inner.push_back(item);
        evicted
    }

    /// Removes and returns the oldest element, or `None` if empty.
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    /// Returns a reference to the oldest element, or `None` if empty.
    pub fn oldest(&self) -> Option<&T> {
        self.inner.front()
    }

    /// Returns a reference to the newest element, or `None` if empty.
    pub fn newest(&self) -> Option<&T> {
        self.inner.back()
    }

    /// The fixed capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The number of elements currently held.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the buffer holds no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if the buffer is at capacity.
    pub fn is_full(&self) -> bool {
        self.inner.len() == self.capacity
    }

    /// Removes all elements, keeping the capacity.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterates over the elements from oldest to newest.
    pub fn iter(&self) -> alloc::collections::vec_deque::Iter<'_, T> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    fn snapshot<T: Clone>(rb: &RingBuffer<T>) -> Vec<T> {
        rb.iter().cloned().collect()
    }

    #[test]
    fn rejects_zero_capacity() {
        assert_eq!(
            RingBuffer::<i32>::new(0).unwrap_err(),
            CollectionError::ZeroCapacity
        );
    }

    #[test]
    fn fills_then_evicts_oldest() {
        let mut rb = RingBuffer::new(3).unwrap();
        assert!(rb.is_empty());
        assert_eq!(rb.push(1), None);
        assert_eq!(rb.push(2), None);
        assert_eq!(rb.push(3), None);
        assert!(rb.is_full());
        assert_eq!(rb.len(), 3);
        assert_eq!(snapshot(&rb), [1, 2, 3]);

        assert_eq!(rb.push(4), Some(1));
        assert_eq!(rb.push(5), Some(2));
        assert_eq!(snapshot(&rb), [3, 4, 5]);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.capacity(), 3);
    }

    #[test]
    fn oldest_newest_pop_and_clear() {
        let mut rb = RingBuffer::new(2).unwrap();
        assert_eq!(rb.oldest(), None);
        assert_eq!(rb.newest(), None);
        rb.push(10);
        rb.push(20);
        assert_eq!(rb.oldest(), Some(&10));
        assert_eq!(rb.newest(), Some(&20));
        assert_eq!(rb.pop(), Some(10)); // oldest out
        assert_eq!(rb.oldest(), Some(&20));
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), 2); // capacity preserved
    }

    #[test]
    fn capacity_one_keeps_only_newest() {
        let mut rb = RingBuffer::new(1).unwrap();
        assert_eq!(rb.push("a"), None);
        assert_eq!(rb.push("b"), Some("a"));
        assert_eq!(rb.push("c"), Some("b"));
        assert_eq!(snapshot(&rb), ["c"]);
    }
}

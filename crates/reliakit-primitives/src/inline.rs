use crate::{PrimitiveError, PrimitiveResult};
use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    ops::Deref,
    str::FromStr,
};

/// A UTF-8 string of `MIN..=MAX` bytes stored inline, with no heap allocation.
///
/// Unlike [`BoundedStr`](crate::BoundedStr), which lives on the heap and counts
/// length in Unicode scalar values (`char`s), `InlineStr` keeps its bytes in a
/// `[u8; MAX]` buffer owned by the value and bounds the **byte** length, since
/// that is what a fixed buffer holds. A multi-byte character therefore spends
/// more than one unit of the bound: `"é"` is two bytes, so it needs `MAX >= 2`.
///
/// Because the bytes live in the value itself, `InlineStr` is `Copy` and needs
/// no allocator, so it works in `no_std` builds without the `alloc` feature.
///
/// Reads go through [`as_str`](Self::as_str), which revalidates the bytes as
/// UTF-8 on each call: the crate forbids unsafe code, so it cannot use
/// `from_utf8_unchecked`. The check is over at most `MAX` bytes.
///
/// # Examples
///
/// ```
/// use reliakit_primitives::InlineStr;
///
/// type Code = InlineStr<1, 8>;
///
/// let code = Code::new("AB12").unwrap();
/// assert_eq!(code.as_str(), "AB12");
/// assert_eq!(code.len(), 4);
///
/// // Nine bytes do not fit the eight-byte budget.
/// assert!(Code::new("123456789").is_err());
/// ```
#[derive(Clone, Copy)]
pub struct InlineStr<const MIN: usize, const MAX: usize> {
    buf: [u8; MAX],
    len: usize,
}

impl<const MIN: usize, const MAX: usize> InlineStr<MIN, MAX> {
    /// Creates a new inline string.
    ///
    /// Length is measured in UTF-8 bytes, not characters. If `MIN > MAX`,
    /// construction returns `OutOfRange`. When `MIN > 0`, an input that is empty
    /// or contains only whitespace is rejected with `Empty`, even if its byte
    /// length would otherwise satisfy `MIN`.
    pub fn new(value: &str) -> PrimitiveResult<Self> {
        let bytes = value.as_bytes();
        let actual = bytes.len();

        if MIN > MAX {
            return Err(PrimitiveError::OutOfRange {
                min: MIN as u128,
                max: MAX as u128,
                actual: actual as u128,
            });
        }
        if actual < MIN {
            return Err(PrimitiveError::TooShort { min: MIN, actual });
        }
        if actual > MAX {
            return Err(PrimitiveError::TooLong { max: MAX, actual });
        }
        if MIN > 0 && value.trim().is_empty() {
            return Err(PrimitiveError::Empty);
        }

        let mut buf = [0u8; MAX];
        buf[..actual].copy_from_slice(bytes);
        Ok(Self { buf, len: actual })
    }

    /// Returns the underlying string slice.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.as_bytes())
            .expect("InlineStr always holds valid UTF-8 by construction")
    }

    /// Returns the string's bytes, without the unused buffer tail.
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    /// Returns the byte length of the string.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the string is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the minimum allowed byte length.
    pub fn min_len(&self) -> usize {
        MIN
    }

    /// Returns the maximum byte length, which is the inline capacity.
    pub fn max_len(&self) -> usize {
        MAX
    }
}

impl<const MIN: usize, const MAX: usize> fmt::Display for InlineStr<MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<const MIN: usize, const MAX: usize> fmt::Debug for InlineStr<MIN, MAX> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const MIN: usize, const MAX: usize> AsRef<str> for InlineStr<MIN, MAX> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const MIN: usize, const MAX: usize> Deref for InlineStr<MIN, MAX> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

// Equality, hashing, and ordering compare the bytes directly: UTF-8 byte order
// matches code-point order, so this agrees with comparing the `&str`s, and it
// skips the UTF-8 revalidation that `as_str` would do.
impl<const MIN: usize, const MAX: usize> PartialEq for InlineStr<MIN, MAX> {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const MIN: usize, const MAX: usize> Eq for InlineStr<MIN, MAX> {}

impl<const MIN: usize, const MAX: usize> Hash for InlineStr<MIN, MAX> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl<const MIN: usize, const MAX: usize> PartialOrd for InlineStr<MIN, MAX> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const MIN: usize, const MAX: usize> Ord for InlineStr<MIN, MAX> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl<const MIN: usize, const MAX: usize> PartialEq<str> for InlineStr<MIN, MAX> {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const MIN: usize, const MAX: usize> PartialEq<&str> for InlineStr<MIN, MAX> {
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const MIN: usize, const MAX: usize> TryFrom<&str> for InlineStr<MIN, MAX> {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<const MIN: usize, const MAX: usize> FromStr for InlineStr<MIN, MAX> {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::InlineStr;
    use crate::PrimitiveError;

    #[test]
    fn accepts_valid_length() {
        let value = InlineStr::<3, 12>::new("service").unwrap();
        assert_eq!(value.as_str(), "service");
        assert_eq!(value.len(), 7);
        assert_eq!(value.min_len(), 3);
        assert_eq!(value.max_len(), 12);
        assert!(!value.is_empty());
    }

    #[test]
    fn rejects_too_short() {
        assert_eq!(
            InlineStr::<3, 12>::new("ab").unwrap_err(),
            PrimitiveError::TooShort { min: 3, actual: 2 }
        );
    }

    #[test]
    fn rejects_too_long() {
        assert_eq!(
            InlineStr::<1, 5>::new("service").unwrap_err(),
            PrimitiveError::TooLong { max: 5, actual: 7 }
        );
    }

    #[test]
    fn counts_bytes_not_chars() {
        // "éå" is two characters but four UTF-8 bytes.
        let value = InlineStr::<4, 4>::new("éå").unwrap();
        assert_eq!(value.len(), 4);
        assert_eq!(value.as_str(), "éå");
        // A single two-byte character does not fit a one-byte budget.
        assert_eq!(
            InlineStr::<1, 1>::new("é").unwrap_err(),
            PrimitiveError::TooLong { max: 1, actual: 2 }
        );
    }

    #[test]
    fn rejects_whitespace_only_when_min_positive() {
        assert_eq!(
            InlineStr::<1, 5>::new("  ").unwrap_err(),
            PrimitiveError::Empty
        );
    }

    #[test]
    fn allows_zero_min_whitespace_only() {
        let value = InlineStr::<0, 5>::new("   ").unwrap();
        assert_eq!(value.as_str(), "   ");
        assert_eq!(value.len(), 3);
    }

    #[test]
    fn allows_empty_when_min_zero() {
        let value = InlineStr::<0, 4>::new("").unwrap();
        assert!(value.is_empty());
        assert_eq!(value.as_str(), "");
    }

    #[test]
    fn zero_capacity_only_holds_empty() {
        assert!(InlineStr::<0, 0>::new("").unwrap().is_empty());
        assert_eq!(
            InlineStr::<0, 0>::new("x").unwrap_err(),
            PrimitiveError::TooLong { max: 0, actual: 1 }
        );
    }

    #[test]
    fn handles_invalid_bounds() {
        assert_eq!(
            InlineStr::<5, 3>::new("abcd").unwrap_err(),
            PrimitiveError::OutOfRange {
                min: 5,
                max: 3,
                actual: 4
            }
        );
    }

    #[test]
    fn exact_min_and_max_fit() {
        assert_eq!(InlineStr::<3, 3>::new("abc").unwrap().len(), 3);
        assert!(InlineStr::<3, 3>::new("ab").is_err());
        assert!(InlineStr::<3, 3>::new("abcd").is_err());
    }

    #[test]
    fn is_copy() {
        let a = InlineStr::<1, 8>::new("copy").unwrap();
        let b = a;
        // `a` is still usable because the value is `Copy`, not moved.
        assert_eq!(a.as_str(), "copy");
        assert_eq!(b.as_str(), "copy");
    }

    #[test]
    fn as_bytes_excludes_unused_tail() {
        let value = InlineStr::<1, 16>::new("hi").unwrap();
        assert_eq!(value.as_bytes(), b"hi");
    }

    #[test]
    fn deref_and_as_ref_to_str() {
        let value = InlineStr::<1, 8>::new("api").unwrap();
        let s: &str = value.as_ref();
        assert_eq!(s, "api");
        // `starts_with` is a `str` method reached through `Deref`.
        assert!(value.starts_with("ap"));
    }

    #[test]
    fn equality_and_ordering() {
        let a = InlineStr::<1, 8>::new("alpha").unwrap();
        let b = InlineStr::<1, 8>::new("beta").unwrap();
        assert_eq!(a, InlineStr::<1, 8>::new("alpha").unwrap());
        assert_ne!(a, b);
        assert!(a < b);
        assert_eq!(a, "alpha");
    }

    #[test]
    fn from_str_and_try_from() {
        let parsed: InlineStr<1, 8> = "node".parse().unwrap();
        assert_eq!(parsed.as_str(), "node");
        let converted = InlineStr::<1, 8>::try_from("node").unwrap();
        assert_eq!(parsed, converted);
        assert!("toolongforbudget".parse::<InlineStr<1, 8>>().is_err());
    }

    #[test]
    fn display_and_debug() {
        use alloc::string::ToString;
        let value = InlineStr::<3, 12>::new("service").unwrap();
        assert_eq!(value.to_string(), "service");
        assert_eq!(alloc::format!("{value:?}"), "\"service\"");
    }

    #[test]
    fn hash_distinguishes_contents() {
        use core::hash::{Hash, Hasher};

        // A small FNV-1a hasher keeps this pure-core and dependency-free; it only
        // needs to tell different contents apart.
        fn fnv(value: &InlineStr<1, 8>) -> u64 {
            struct Fnv(u64);
            impl Hasher for Fnv {
                fn finish(&self) -> u64 {
                    self.0
                }
                fn write(&mut self, bytes: &[u8]) {
                    for &b in bytes {
                        self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
                    }
                }
            }
            let mut hasher = Fnv(0xcbf2_9ce4_8422_2325);
            value.hash(&mut hasher);
            hasher.finish()
        }

        let a = InlineStr::<1, 8>::new("alpha").unwrap();
        assert_eq!(fnv(&a), fnv(&InlineStr::<1, 8>::new("alpha").unwrap()));
        // Different contents hash differently, so a no-op hash impl is caught.
        assert_ne!(fnv(&a), fnv(&InlineStr::<1, 8>::new("beta").unwrap()));
    }

    #[test]
    fn str_equality_both_impls() {
        let a = InlineStr::<1, 8>::new("alpha").unwrap();
        // `PartialEq<&str>`: both equal and not equal.
        assert!(a == "alpha");
        assert!(a != "other");
        // `PartialEq<str>` (the unsized form) is selected by a generic bound.
        fn eq_str<T: PartialEq<str> + Copy>(value: T, s: &str) -> bool {
            value == *s
        }
        assert!(eq_str(a, "alpha"));
        assert!(!eq_str(a, "other"));
    }
}

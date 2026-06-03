//! Convenience helpers for common encode and decode operations.

use crate::{CanonicalDecode, CodecError, SliceReader};

#[cfg(feature = "alloc")]
use crate::CanonicalEncode;

/// Encodes a value into an owned byte vector.
#[cfg(feature = "alloc")]
pub fn encode_to_vec<T: CanonicalEncode + ?Sized>(
    value: &T,
) -> Result<alloc::vec::Vec<u8>, CodecError> {
    let mut out = alloc::vec::Vec::new();
    value.encode(&mut out)?;
    Ok(out)
}

/// Decodes a value from a byte slice and returns the value plus unread byte count.
pub fn decode_from_slice<T: CanonicalDecode>(bytes: &[u8]) -> Result<(T, usize), CodecError> {
    let mut reader = SliceReader::new(bytes);
    let value = T::decode(&mut reader)?;
    Ok((value, reader.remaining()))
}

/// Decodes a value from a byte slice and rejects trailing bytes.
pub fn decode_from_slice_exact<T: CanonicalDecode>(bytes: &[u8]) -> Result<T, CodecError> {
    let (value, remaining) = decode_from_slice(bytes)?;
    if remaining != 0 {
        return Err(CodecError::trailing_bytes());
    }
    Ok(value)
}

//! Decoding traits and sources.

use crate::CodecError;

/// Source for canonical encoded bytes.
pub trait DecodeSource {
    /// Reads exactly enough bytes to fill `out` or returns an error.
    fn read_exact(&mut self, out: &mut [u8]) -> Result<(), CodecError>;

    /// Returns the unread byte count when the source can know it cheaply.
    ///
    /// Streaming sources may return `None`. Slice-backed sources return
    /// `Some(_)`, allowing decoders to reject impossible length prefixes before
    /// allocating.
    fn remaining_len(&self) -> Option<usize> {
        None
    }
}

/// Trait for strict canonical binary decoding.
pub trait CanonicalDecode: Sized {
    /// Decodes `Self` from `reader` using the crate's canonical binary format.
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError>;
}

/// Decode source backed by an immutable byte slice.
#[derive(Debug, Clone)]
pub struct SliceReader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> SliceReader<'a> {
    /// Creates a new reader over `input`.
    pub const fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    /// Returns the number of unread bytes.
    pub const fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    /// Returns `true` if all bytes have been consumed.
    pub const fn is_empty(&self) -> bool {
        self.remaining() == 0
    }
}

impl DecodeSource for SliceReader<'_> {
    fn read_exact(&mut self, out: &mut [u8]) -> Result<(), CodecError> {
        let end = self
            .offset
            .checked_add(out.len())
            .ok_or_else(|| CodecError::length_overflow("read offset overflow"))?;
        let bytes = self
            .input
            .get(self.offset..end)
            .ok_or_else(CodecError::unexpected_eof)?;
        out.copy_from_slice(bytes);
        self.offset = end;
        Ok(())
    }

    fn remaining_len(&self) -> Option<usize> {
        Some(self.remaining())
    }
}

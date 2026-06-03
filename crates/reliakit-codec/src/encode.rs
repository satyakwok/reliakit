//! Encoding traits and sinks.

use crate::CodecError;

/// Destination for canonical encoded bytes.
pub trait EncodeSink {
    /// Writes all bytes to the sink or returns an error.
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), CodecError>;
}

/// Trait for deterministic canonical binary encoding.
pub trait CanonicalEncode {
    /// Encodes `self` into `writer` using the crate's canonical binary format.
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError>;
}

#[cfg(feature = "alloc")]
impl EncodeSink for alloc::vec::Vec<u8> {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), CodecError> {
        self.extend_from_slice(bytes);
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T: std::io::Write> EncodeSink for std::io::BufWriter<T> {
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), CodecError> {
        std::io::Write::write_all(self, bytes).map_err(|_| CodecError::write_failed())
    }
}

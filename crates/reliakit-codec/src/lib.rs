//! Deterministic canonical binary encoding and decoding.
//!
//! `reliakit-codec` provides small traits and strict primitive implementations
//! for one canonical binary representation per supported type. It is intended
//! for simple protocols, fixtures, cache keys, and reliability-oriented library
//! boundaries where handwritten implementations are preferable to schema or
//! derive machinery.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "alloc")]
//! # {
//! use reliakit_codec::{decode_from_slice_exact, encode_to_vec, CanonicalDecode, CanonicalEncode};
//!
//! #[derive(Debug, PartialEq)]
//! struct Point {
//!     x: u16,
//!     y: u16,
//! }
//!
//! impl CanonicalEncode for Point {
//!     fn encode<W: reliakit_codec::EncodeSink + ?Sized>(
//!         &self,
//!         writer: &mut W,
//!     ) -> Result<(), reliakit_codec::CodecError> {
//!         self.x.encode(writer)?;
//!         self.y.encode(writer)
//!     }
//! }
//!
//! impl CanonicalDecode for Point {
//!     fn decode<R: reliakit_codec::DecodeSource + ?Sized>(
//!         reader: &mut R,
//!     ) -> Result<Self, reliakit_codec::CodecError> {
//!         Ok(Self {
//!             x: u16::decode(reader)?,
//!             y: u16::decode(reader)?,
//!         })
//!     }
//! }
//!
//! let encoded = encode_to_vec(&Point { x: 10, y: 20 })?;
//! assert_eq!(encoded, [10, 0, 20, 0]);
//! assert_eq!(decode_from_slice_exact::<Point>(&encoded)?, Point { x: 10, y: 20 });
//! # Ok::<(), reliakit_codec::CodecError>(())
//! # }
//! # #[cfg(not(feature = "alloc"))]
//! # Ok::<(), reliakit_codec::CodecError>(())
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Decoding traits and byte-slice readers.
pub mod decode;
/// Encoding traits and byte sinks.
pub mod encode;
/// Error types.
pub mod error;
/// Wire format constants and documentation.
pub mod format;
/// Convenience helpers.
pub mod helpers;
mod impls;
/// Optional `reliakit-primitives` integrations.
#[cfg(feature = "primitives")]
pub mod primitives;

pub use decode::{CanonicalDecode, DecodeSource, SliceReader};
pub use encode::{CanonicalEncode, EncodeSink};
pub use error::{CodecError, CodecErrorKind};
pub use helpers::{decode_from_slice, decode_from_slice_exact};

#[cfg(feature = "alloc")]
pub use helpers::encode_to_vec;

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    #[derive(Debug, PartialEq, Eq)]
    struct Message {
        id: u32,
        body: String,
        urgent: bool,
    }

    impl CanonicalEncode for Message {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.id.encode(writer)?;
            self.body.encode(writer)?;
            self.urgent.encode(writer)
        }
    }

    impl CanonicalDecode for Message {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            Ok(Self {
                id: u32::decode(reader)?,
                body: String::decode(reader)?,
                urgent: bool::decode(reader)?,
            })
        }
    }

    #[test]
    fn primitive_roundtrips_are_little_endian() {
        assert_eq!(encode_to_vec(&0x1234u16).unwrap(), vec![0x34, 0x12]);
        assert_eq!(
            decode_from_slice_exact::<u16>(&[0x34, 0x12]).unwrap(),
            0x1234
        );
        assert_eq!(encode_to_vec(&-2i16).unwrap(), (-2i16).to_le_bytes());
    }

    #[test]
    fn bool_decode_is_strict() {
        assert!(!decode_from_slice_exact::<bool>(&[0x00]).unwrap());
        assert!(decode_from_slice_exact::<bool>(&[0x01]).unwrap());
        let err = decode_from_slice_exact::<bool>(&[0x02]).unwrap_err();
        assert_eq!(err.kind(), CodecErrorKind::InvalidValue);
    }

    #[test]
    fn string_rejects_invalid_utf8() {
        let err = decode_from_slice_exact::<String>(&[1, 0, 0, 0, 0xff]).unwrap_err();
        assert_eq!(err.kind(), CodecErrorKind::InvalidValue);
    }

    #[test]
    fn length_prefix_controls_string_bytes() {
        assert_eq!(
            encode_to_vec("abc").unwrap(),
            vec![3, 0, 0, 0, b'a', b'b', b'c']
        );
        assert_eq!(
            decode_from_slice_exact::<String>(&[3, 0, 0, 0, b'a', b'b', b'c']).unwrap(),
            "abc"
        );
    }

    #[test]
    fn exact_decode_rejects_trailing_bytes() {
        let err = decode_from_slice_exact::<u8>(&[1, 2]).unwrap_err();
        assert_eq!(err.kind(), CodecErrorKind::TrailingBytes);
        assert_eq!(decode_from_slice::<u8>(&[1, 2]).unwrap(), (1, 1));
    }

    #[test]
    fn manual_struct_roundtrip() {
        let message = Message {
            id: 7,
            body: String::from("ready"),
            urgent: true,
        };
        let encoded = encode_to_vec(&message).unwrap();
        assert_eq!(
            decode_from_slice_exact::<Message>(&encoded).unwrap(),
            message
        );
    }

    #[test]
    fn invalid_tags_fail() {
        assert_eq!(
            decode_from_slice_exact::<Option<u8>>(&[3])
                .unwrap_err()
                .kind(),
            CodecErrorKind::InvalidValue
        );
        assert_eq!(
            decode_from_slice_exact::<Result<u8, u8>>(&[3])
                .unwrap_err()
                .kind(),
            CodecErrorKind::InvalidValue
        );
    }

    #[test]
    fn vec_and_array_roundtrip() {
        let values = vec![1u16, 2, 3];
        let encoded = encode_to_vec(&values).unwrap();
        assert_eq!(
            decode_from_slice_exact::<Vec<u16>>(&encoded).unwrap(),
            values
        );

        let array = [1u8, 2, 3, 4];
        let encoded = encode_to_vec(&array).unwrap();
        assert_eq!(decode_from_slice_exact::<[u8; 4]>(&encoded).unwrap(), array);
    }
}

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
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;
    use core::fmt::Write as _;

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
    fn integer_widths_roundtrip() {
        macro_rules! assert_integer {
            ($ty:ty, $value:expr) => {
                let value = $value as $ty;
                let encoded = encode_to_vec(&value).unwrap();
                assert_eq!(encoded, value.to_le_bytes());
                assert_eq!(decode_from_slice_exact::<$ty>(&encoded).unwrap(), value);
            };
        }

        assert_integer!(u8, 200);
        assert_integer!(i8, -12);
        assert_integer!(u16, 0x1234);
        assert_integer!(i16, -1234);
        assert_integer!(u32, 0x1234_5678);
        assert_integer!(i32, -123_456);
        assert_integer!(u64, 0x1234_5678_9abc_def0);
        assert_integer!(i64, -123_456_789);
        assert_integer!(u128, 0x1234_5678_9abc_def0_1111_2222_3333_4444);
        assert_integer!(i128, -123_456_789_123_456_789);
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
    fn string_rejects_impossible_length_before_allocating() {
        let err = decode_from_slice_exact::<String>(&u32::MAX.to_le_bytes()).unwrap_err();
        assert_eq!(err.kind(), CodecErrorKind::UnexpectedEof);
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
    fn slice_reader_reports_remaining_and_eof() {
        let mut reader = SliceReader::new(&[1, 2]);
        assert!(!reader.is_empty());

        let mut one = [0u8; 1];
        reader.read_exact(&mut one).unwrap();
        assert_eq!(one, [1]);
        assert_eq!(reader.remaining(), 1);

        let mut two = [0u8; 2];
        let err = reader.read_exact(&mut two).unwrap_err();
        assert_eq!(err.kind(), CodecErrorKind::UnexpectedEof);
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
    fn option_and_result_encode_all_branches() {
        let none: Option<u16> = None;
        assert_eq!(encode_to_vec(&none).unwrap(), vec![0]);
        assert_eq!(decode_from_slice_exact::<Option<u16>>(&[0]).unwrap(), None);

        let some = Some(0x1234u16);
        assert_eq!(encode_to_vec(&some).unwrap(), vec![1, 0x34, 0x12]);
        assert_eq!(
            decode_from_slice_exact::<Option<u16>>(&[1, 0x34, 0x12]).unwrap(),
            some
        );

        let ok: Result<u8, u16> = Ok(9);
        assert_eq!(encode_to_vec(&ok).unwrap(), vec![0, 9]);
        assert_eq!(
            decode_from_slice_exact::<Result<u8, u16>>(&[0, 9]).unwrap(),
            ok
        );

        let err: Result<u8, u16> = Err(0x1234);
        assert_eq!(encode_to_vec(&err).unwrap(), vec![1, 0x34, 0x12]);
        assert_eq!(
            decode_from_slice_exact::<Result<u8, u16>>(&[1, 0x34, 0x12]).unwrap(),
            err
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
        // Fixed arrays carry no length prefix: items are written in order.
        assert_eq!(encoded, vec![1, 2, 3, 4]);
        assert_eq!(decode_from_slice_exact::<[u8; 4]>(&encoded).unwrap(), array);
    }

    #[test]
    fn vec_of_zero_sized_elements_roundtrips() {
        // A `Vec` of zero-sized elements encodes to just its `u32` item-count
        // prefix; decoding reconstructs the same count. This pins the documented
        // behavior that the prefix is an item count, not a byte length.
        let values: Vec<[u8; 0]> = vec![[], [], []];
        let encoded = encode_to_vec(&values).unwrap();
        assert_eq!(encoded, vec![3, 0, 0, 0]);
        assert_eq!(
            decode_from_slice_exact::<Vec<[u8; 0]>>(&encoded).unwrap(),
            values
        );
    }

    #[test]
    fn tuples_roundtrip_by_field_order() {
        assert_eq!(
            decode_from_slice_exact::<(u8,)>(&encode_to_vec(&(1u8,)).unwrap()).unwrap(),
            (1,)
        );
        assert_eq!(
            decode_from_slice_exact::<(u8, u16)>(&encode_to_vec(&(1u8, 0x0203u16)).unwrap())
                .unwrap(),
            (1, 0x0203)
        );
        assert_eq!(
            decode_from_slice_exact::<(u8, u16, bool)>(
                &encode_to_vec(&(1u8, 0x0203u16, true)).unwrap()
            )
            .unwrap(),
            (1, 0x0203, true)
        );
        assert_eq!(
            decode_from_slice_exact::<(u8, u16, bool, i8)>(
                &encode_to_vec(&(1u8, 0x0203u16, true, -4i8)).unwrap()
            )
            .unwrap(),
            (1, 0x0203, true, -4)
        );
    }

    #[test]
    fn codec_error_accessors_and_display_are_actionable() {
        let error = CodecError::new(CodecErrorKind::ReadFailed, "reader failed");
        assert_eq!(error.kind(), CodecErrorKind::ReadFailed);
        assert_eq!(error.message(), "reader failed");
        assert_eq!(error.to_string(), "reader failed");

        assert_eq!(CodecError::read_failed().kind(), CodecErrorKind::ReadFailed);
        assert_eq!(
            CodecError::write_failed().kind(),
            CodecErrorKind::WriteFailed
        );
        assert_eq!(
            CodecError::length_overflow("length too large").message(),
            "length too large"
        );

        let mut rendered = String::new();
        write!(&mut rendered, "{}", CodecError::trailing_bytes()).unwrap();
        assert_eq!(rendered, "decode completed but trailing bytes remain");
    }

    #[test]
    #[cfg(feature = "std")]
    fn std_buf_writer_maps_write_failures() {
        struct FailingWriter;

        impl std::io::Write for FailingWriter {
            fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("fail"))
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let mut writer = std::io::BufWriter::with_capacity(0, FailingWriter);
        let err = 1u8.encode(&mut writer).unwrap_err();
        assert_eq!(err.kind(), CodecErrorKind::WriteFailed);
    }
}

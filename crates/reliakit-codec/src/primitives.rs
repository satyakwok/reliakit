//! Optional integrations for `reliakit-primitives`.
//!
//! These implementations are available with the `primitives` feature. Decoding
//! always uses public constructors or parsers so primitive invariants are
//! preserved.

#[cfg(feature = "primitives")]
mod impls {
    use crate::{CanonicalDecode, CanonicalEncode, CodecError, DecodeSource, EncodeSink};
    use alloc::string::{String, ToString};
    use reliakit_primitives::{
        BoundedStr, ByteSize, Email, HexString, HttpUrl, HumanDuration, NonEmptyStr, NonEmptyVec,
        Percent, Port, PositiveInt, SemVer, Slug, Uuid,
    };

    fn invalid_primitive() -> CodecError {
        CodecError::invalid_value("decoded value failed reliakit-primitives validation")
    }

    macro_rules! impl_string_primitive {
        ($ty:ty, $ctor:expr) => {
            impl CanonicalEncode for $ty {
                fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
                    self.as_str().encode(writer)
                }
            }

            impl CanonicalDecode for $ty {
                fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
                    let value = String::decode(reader)?;
                    $ctor(value).map_err(|_| invalid_primitive())
                }
            }
        };
    }

    impl_string_primitive!(NonEmptyStr, NonEmptyStr::new);
    impl_string_primitive!(Email, Email::new);
    impl_string_primitive!(HttpUrl, HttpUrl::new);
    impl_string_primitive!(Slug, Slug::new);
    impl_string_primitive!(HexString, HexString::new);

    impl<const MIN: usize, const MAX: usize> CanonicalEncode for BoundedStr<MIN, MAX> {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.as_str().encode(writer)
        }
    }

    impl<const MIN: usize, const MAX: usize> CanonicalDecode for BoundedStr<MIN, MAX> {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            let value = String::decode(reader)?;
            Self::new(value).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for Port {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.get().encode(writer)
        }
    }

    impl CanonicalDecode for Port {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            Self::new(u16::decode(reader)?).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for Percent {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.get().encode(writer)
        }
    }

    impl CanonicalDecode for Percent {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            Self::new(u8::decode(reader)?).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for PositiveInt {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.get().encode(writer)
        }
    }

    impl CanonicalDecode for PositiveInt {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            Self::new(u64::decode(reader)?).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for ByteSize {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.as_bytes().encode(writer)
        }
    }

    impl CanonicalDecode for ByteSize {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            Ok(Self::from_bytes(u64::decode(reader)?))
        }
    }

    impl<T: CanonicalEncode> CanonicalEncode for NonEmptyVec<T> {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            let len = u32::try_from(self.len()).map_err(|_| {
                CodecError::length_overflow("non-empty vector length exceeds u32::MAX items")
            })?;
            len.encode(writer)?;
            for item in self.iter() {
                item.encode(writer)?;
            }
            Ok(())
        }
    }

    impl<T: CanonicalDecode> CanonicalDecode for NonEmptyVec<T> {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            Self::new(alloc::vec::Vec::<T>::decode(reader)?).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for Uuid {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.as_bytes().encode(writer)
        }
    }

    impl CanonicalDecode for Uuid {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            let bytes = <[u8; 16]>::decode(reader)?;
            let text = format_uuid(bytes);
            Self::parse(&text).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for SemVer {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.to_string().encode(writer)
        }
    }

    impl CanonicalDecode for SemVer {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            let value = String::decode(reader)?;
            Self::parse(&value).map_err(|_| invalid_primitive())
        }
    }

    impl CanonicalEncode for HumanDuration {
        fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
            self.to_string().encode(writer)
        }
    }

    impl CanonicalDecode for HumanDuration {
        fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
            let value = String::decode(reader)?;
            Self::parse(&value).map_err(|_| invalid_primitive())
        }
    }

    fn format_uuid(bytes: [u8; 16]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(36);
        for (idx, byte) in bytes.iter().copied().enumerate() {
            if matches!(idx, 4 | 6 | 8 | 10) {
                out.push('-');
            }
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }

    // Float-backed primitives are intentionally not implemented in v0.1 because
    // the codec format does not define float encoding.
}

#[cfg(all(test, feature = "primitives"))]
mod tests {
    use crate::{decode_from_slice_exact, encode_to_vec, CodecErrorKind};
    use alloc::vec;
    use reliakit_primitives::{Email, NonEmptyStr, NonEmptyVec, Percent, Port, Uuid};

    #[test]
    fn string_primitives_roundtrip_through_validation() {
        let name = NonEmptyStr::new("api").unwrap();
        let encoded = encode_to_vec(&name).unwrap();
        assert_eq!(
            decode_from_slice_exact::<NonEmptyStr>(&encoded).unwrap(),
            name
        );

        let email = Email::new("ops@example.com").unwrap();
        let encoded = encode_to_vec(&email).unwrap();
        assert_eq!(decode_from_slice_exact::<Email>(&encoded).unwrap(), email);
    }

    #[test]
    fn numeric_primitives_reject_invalid_decoded_values() {
        assert_eq!(
            decode_from_slice_exact::<Port>(&0u16.to_le_bytes())
                .unwrap_err()
                .kind(),
            CodecErrorKind::InvalidValue
        );
        assert_eq!(
            decode_from_slice_exact::<Percent>(&[101])
                .unwrap_err()
                .kind(),
            CodecErrorKind::InvalidValue
        );
    }

    #[test]
    fn uuid_encodes_raw_bytes_canonically() {
        let uuid = Uuid::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let encoded = encode_to_vec(&uuid).unwrap();
        assert_eq!(encoded, uuid.as_bytes());
        assert_eq!(decode_from_slice_exact::<Uuid>(&encoded).unwrap(), uuid);
    }

    #[test]
    fn non_empty_vec_decode_validates_non_empty() {
        let values = NonEmptyVec::new(vec![1u8, 2, 3]).unwrap();
        let encoded = encode_to_vec(&values).unwrap();
        assert_eq!(
            decode_from_slice_exact::<NonEmptyVec<u8>>(&encoded).unwrap(),
            values
        );

        assert_eq!(
            decode_from_slice_exact::<NonEmptyVec<u8>>(&0u32.to_le_bytes())
                .unwrap_err()
                .kind(),
            CodecErrorKind::InvalidValue
        );
    }
}

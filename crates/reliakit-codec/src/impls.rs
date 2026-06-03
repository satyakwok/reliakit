//! Canonical implementations for Rust primitive and standard library types.

use crate::format::{BOOL_FALSE, BOOL_TRUE, OPTION_NONE, OPTION_SOME, RESULT_ERR, RESULT_OK};
use crate::{CanonicalDecode, CanonicalEncode, CodecError, DecodeSource, EncodeSink};

macro_rules! impl_int {
    ($ty:ty) => {
        impl CanonicalEncode for $ty {
            fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
                writer.write_all(&self.to_le_bytes())
            }
        }

        impl CanonicalDecode for $ty {
            fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
                let mut bytes = [0u8; core::mem::size_of::<$ty>()];
                reader.read_exact(&mut bytes)?;
                Ok(<$ty>::from_le_bytes(bytes))
            }
        }
    };
}

impl_int!(u16);
impl_int!(i16);
impl_int!(u32);
impl_int!(i32);
impl_int!(u64);
impl_int!(i64);
impl_int!(u128);
impl_int!(i128);

impl CanonicalEncode for u8 {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        writer.write_all(&[*self])
    }
}

impl CanonicalDecode for u8 {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        Ok(byte[0])
    }
}

impl CanonicalEncode for i8 {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        writer.write_all(&self.to_le_bytes())
    }
}

impl CanonicalDecode for i8 {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        let byte = u8::decode(reader)?;
        Ok(Self::from_le_bytes([byte]))
    }
}

impl CanonicalEncode for bool {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        writer.write_all(&[if *self { BOOL_TRUE } else { BOOL_FALSE }])
    }
}

impl CanonicalDecode for bool {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        match u8::decode(reader)? {
            BOOL_FALSE => Ok(false),
            BOOL_TRUE => Ok(true),
            _ => Err(CodecError::invalid_value(
                "invalid bool byte: expected 0x00 or 0x01",
            )),
        }
    }
}

impl CanonicalEncode for str {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        let bytes = self.as_bytes();
        let len = u32::try_from(bytes.len())
            .map_err(|_| CodecError::length_overflow("string length exceeds u32::MAX bytes"))?;
        len.encode(writer)?;
        writer.write_all(bytes)
    }
}

#[cfg(feature = "alloc")]
impl CanonicalEncode for alloc::string::String {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        self.as_str().encode(writer)
    }
}

#[cfg(feature = "alloc")]
impl CanonicalDecode for alloc::string::String {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        let len = u32::decode(reader)?;
        let len = usize::try_from(len)
            .map_err(|_| CodecError::length_overflow("string length does not fit usize"))?;
        let mut bytes = alloc::vec![0u8; len];
        reader.read_exact(&mut bytes)?;
        Self::from_utf8(bytes)
            .map_err(|_| CodecError::invalid_value("invalid UTF-8 string payload"))
    }
}

#[cfg(feature = "alloc")]
impl<T: CanonicalEncode> CanonicalEncode for alloc::vec::Vec<T> {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        let len = u32::try_from(self.len())
            .map_err(|_| CodecError::length_overflow("vector length exceeds u32::MAX items"))?;
        len.encode(writer)?;
        for item in self {
            item.encode(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T: CanonicalDecode> CanonicalDecode for alloc::vec::Vec<T> {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        let len = u32::decode(reader)?;
        let len = usize::try_from(len)
            .map_err(|_| CodecError::length_overflow("vector length does not fit usize"))?;
        let mut items = alloc::vec::Vec::with_capacity(len);
        for _ in 0..len {
            items.push(T::decode(reader)?);
        }
        Ok(items)
    }
}

impl<T: CanonicalEncode> CanonicalEncode for Option<T> {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        match self {
            None => OPTION_NONE.encode(writer),
            Some(value) => {
                OPTION_SOME.encode(writer)?;
                value.encode(writer)
            }
        }
    }
}

impl<T: CanonicalDecode> CanonicalDecode for Option<T> {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        match u8::decode(reader)? {
            OPTION_NONE => Ok(None),
            OPTION_SOME => T::decode(reader).map(Some),
            _ => Err(CodecError::invalid_value(
                "invalid Option tag: expected 0x00 for None or 0x01 for Some",
            )),
        }
    }
}

impl<T: CanonicalEncode, E: CanonicalEncode> CanonicalEncode for Result<T, E> {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        match self {
            Ok(value) => {
                RESULT_OK.encode(writer)?;
                value.encode(writer)
            }
            Err(error) => {
                RESULT_ERR.encode(writer)?;
                error.encode(writer)
            }
        }
    }
}

impl<T: CanonicalDecode, E: CanonicalDecode> CanonicalDecode for Result<T, E> {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        match u8::decode(reader)? {
            RESULT_OK => T::decode(reader).map(Ok),
            RESULT_ERR => E::decode(reader).map(Err),
            _ => Err(CodecError::invalid_value(
                "invalid Result tag: expected 0x00 for Ok or 0x01 for Err",
            )),
        }
    }
}

impl<T: CanonicalEncode, const N: usize> CanonicalEncode for [T; N] {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        for item in self {
            item.encode(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T: CanonicalDecode, const N: usize> CanonicalDecode for [T; N] {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        let mut items = alloc::vec::Vec::with_capacity(N);
        for _ in 0..N {
            items.push(T::decode(reader)?);
        }
        match items.try_into() {
            Ok(array) => Ok(array),
            Err(_) => Err(CodecError::invalid_value("decoded array length mismatch")),
        }
    }
}

macro_rules! impl_tuple {
    ($($name:ident),+) => {
        impl<$($name: CanonicalEncode),+> CanonicalEncode for ($($name,)+) {
            #[allow(non_snake_case)]
            fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
                let ($($name,)+) = self;
                $($name.encode(writer)?;)+
                Ok(())
            }
        }

        impl<$($name: CanonicalDecode),+> CanonicalDecode for ($($name,)+) {
            fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
                Ok(($($name::decode(reader)?,)+))
            }
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);

//! Encoding and decoding of individual CSV fields.

use alloc::string::{String, ToString};

use crate::error::CsvDecodeError;

/// A scalar value that maps to and from a single CSV field.
///
/// Encoding never fails — every supported value has a text form. Decoding is
/// strict: the field text must parse exactly into the target type.
///
/// Implemented for the integer types, `bool` (`"true"`/`"false"`), `char`, `String`,
/// and `Option<T>` (an empty field decodes to `None`).
pub trait CsvField: Sized {
    /// Encodes `self` into a field value.
    fn encode_field(&self) -> String;

    /// Decodes a field value into `Self`, or returns a [`CsvDecodeError`].
    fn decode_field(input: &str) -> Result<Self, CsvDecodeError>;
}

macro_rules! impl_int {
    ($($t:ty),* $(,)?) => {$(
        impl CsvField for $t {
            fn encode_field(&self) -> String {
                self.to_string()
            }
            fn decode_field(input: &str) -> Result<Self, CsvDecodeError> {
                input.parse::<$t>().map_err(|_| {
                    CsvDecodeError::field("field is not an integer that fits the target type")
                })
            }
        }
    )*};
}
impl_int!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl CsvField for bool {
    fn encode_field(&self) -> String {
        if *self { "true" } else { "false" }.to_string()
    }
    fn decode_field(input: &str) -> Result<Self, CsvDecodeError> {
        match input {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(CsvDecodeError::field("field is not `true` or `false`")),
        }
    }
}

impl CsvField for char {
    fn encode_field(&self) -> String {
        self.to_string()
    }
    fn decode_field(input: &str) -> Result<Self, CsvDecodeError> {
        let mut chars = input.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            (None, _) => Err(CsvDecodeError::field("field is empty")),
            (Some(_), Some(_)) => Err(CsvDecodeError::field("field is not a single char")),
        }
    }
}

impl CsvField for String {
    fn encode_field(&self) -> String {
        self.clone()
    }
    fn decode_field(input: &str) -> Result<Self, CsvDecodeError> {
        Ok(input.to_string())
    }
}

impl<T: CsvField> CsvField for Option<T> {
    fn encode_field(&self) -> String {
        match self {
            Some(value) => value.encode_field(),
            None => String::new(),
        }
    }
    fn decode_field(input: &str) -> Result<Self, CsvDecodeError> {
        if input.is_empty() {
            Ok(None)
        } else {
            T::decode_field(input).map(Some)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_round_trip_and_reject() {
        assert_eq!(255u8.encode_field(), "255");
        assert_eq!(u8::decode_field("255").unwrap(), 255);
        assert!(u8::decode_field("256").is_err());
        assert!(u8::decode_field("").is_err());
        assert_eq!(i32::decode_field("-5").unwrap(), -5);
    }

    #[test]
    fn bool_is_strict() {
        assert_eq!(true.encode_field(), "true");
        assert_eq!(false.encode_field(), "false");
        assert!(bool::decode_field("true").unwrap());
        assert!(!bool::decode_field("false").unwrap());
        assert!(bool::decode_field("True").is_err());
        assert!(bool::decode_field("1").is_err());
    }

    #[test]
    fn char_is_strict() {
        assert_eq!('a'.encode_field(), "a");
        assert_eq!(char::decode_field("a").unwrap(), 'a');
        let crab = '🦀';
        assert_eq!(char::decode_field(&crab.encode_field()).unwrap(), crab);
        assert!(char::decode_field("").is_err());
        assert!(char::decode_field("abc").is_err());
    }

    #[test]
    fn string_encode_and_decode() {
        assert_eq!(String::from("hi").encode_field(), "hi");
        assert_eq!(String::decode_field("hi").unwrap(), "hi");
        assert_eq!(String::decode_field("").unwrap(), "");
    }

    #[test]
    fn option_uses_empty_for_none() {
        assert_eq!(Option::<u8>::None.encode_field(), "");
        assert_eq!(Some(7u8).encode_field(), "7");
        assert_eq!(Option::<u8>::decode_field("").unwrap(), None);
        assert_eq!(Option::<u8>::decode_field("7").unwrap(), Some(7));
        assert!(Option::<u8>::decode_field("x").is_err());
    }
}

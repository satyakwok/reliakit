//! Typed JSON decoding: build a Rust value from a [`JsonValue`].
//!
//! [`JsonDecode`] is the decode half of typed JSON serialization. Decoding is
//! strict: the JSON type must match the target, required object fields must be
//! present, and numbers must fit the target type. Unknown object fields are
//! ignored. Use [`from_json_str`] to parse and decode in one step.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{JsonDecodeError, JsonFromStrError};
use crate::parse::parse_str;
use crate::value::JsonValue;

/// A type that can be decoded from a [`JsonValue`].
///
/// The derive in `reliakit-derive` generates implementations of this trait.
pub trait JsonDecode: Sized {
    /// Decodes `Self` from a [`JsonValue`], or returns a [`JsonDecodeError`].
    fn from_json_value(value: &JsonValue) -> Result<Self, JsonDecodeError>;
}

/// Parses JSON text and decodes it into `T` in one step.
pub fn from_json_str<T: JsonDecode>(input: &str) -> Result<T, JsonFromStrError> {
    let value = parse_str(input)?;
    let decoded = T::from_json_value(&value)?;
    Ok(decoded)
}

macro_rules! impl_int_decode {
    ($($t:ty),* $(,)?) => {$(
        impl JsonDecode for $t {
            fn from_json_value(value: &JsonValue) -> Result<Self, JsonDecodeError> {
                let number = value
                    .as_number()
                    .ok_or_else(|| JsonDecodeError::unexpected_type("expected a JSON number"))?;
                // Strict: the number's exact text must be a plain integer that
                // fits the target type (no fraction, exponent, or overflow).
                number.as_str().parse::<$t>().map_err(|_| {
                    JsonDecodeError::number(
                        "number is not a plain integer that fits the target type",
                    )
                })
            }
        }
    )*};
}
impl_int_decode!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl JsonDecode for bool {
    fn from_json_value(value: &JsonValue) -> Result<Self, JsonDecodeError> {
        value
            .as_bool()
            .ok_or_else(|| JsonDecodeError::unexpected_type("expected a JSON boolean"))
    }
}

impl JsonDecode for String {
    fn from_json_value(value: &JsonValue) -> Result<Self, JsonDecodeError> {
        value
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| JsonDecodeError::unexpected_type("expected a JSON string"))
    }
}

impl<T: JsonDecode> JsonDecode for Option<T> {
    fn from_json_value(value: &JsonValue) -> Result<Self, JsonDecodeError> {
        if value.is_null() {
            Ok(None)
        } else {
            T::from_json_value(value).map(Some)
        }
    }
}

impl<T: JsonDecode> JsonDecode for Vec<T> {
    fn from_json_value(value: &JsonValue) -> Result<Self, JsonDecodeError> {
        let array = value
            .as_array()
            .ok_or_else(|| JsonDecodeError::unexpected_type("expected a JSON array"))?;
        let mut out = Vec::with_capacity(array.len());
        for item in array {
            out.push(T::from_json_value(item)?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::{to_json_string, JsonEncode};
    use crate::error::JsonDecodeErrorKind;

    fn decode<T: JsonDecode>(input: &str) -> T {
        from_json_str(input).expect("should decode")
    }

    fn roundtrip<T: JsonEncode + JsonDecode + PartialEq + core::fmt::Debug>(value: T) {
        let text = to_json_string(&value);
        let back: T = from_json_str(&text).expect("round-trip should decode");
        assert_eq!(back, value, "round-trip mismatch for {text}");
    }

    #[test]
    fn decodes_scalars() {
        assert_eq!(decode::<u8>("255"), 255);
        assert_eq!(decode::<i32>("-5"), -5);
        assert_eq!(
            decode::<u128>("340282366920938463463374607431768211455"),
            u128::MAX
        );
        assert!(decode::<bool>("true"));
        assert_eq!(decode::<String>("\"hi\""), "hi");
    }

    #[test]
    fn decodes_option_and_sequences() {
        assert_eq!(decode::<Option<u8>>("null"), None);
        assert_eq!(decode::<Option<u8>>("7"), Some(7));
        assert_eq!(decode::<Vec<u8>>("[1,2,3]"), vec![1, 2, 3]);
        assert_eq!(decode::<Vec<u8>>("[]"), Vec::<u8>::new());
    }

    #[test]
    fn round_trips() {
        roundtrip(255u8);
        roundtrip(-12345i32);
        roundtrip(u128::MAX);
        roundtrip(true);
        roundtrip(String::from("hello"));
        roundtrip(Some(9u16));
        roundtrip(Option::<u16>::None);
        roundtrip(vec![1u8, 2, 3]);
    }

    #[test]
    fn wrong_type_is_rejected() {
        let err = from_json_str::<u8>("\"x\"").unwrap_err();
        match err {
            JsonFromStrError::Decode(e) => {
                assert_eq!(e.kind(), JsonDecodeErrorKind::UnexpectedType)
            }
            other => panic!("expected decode error, got {other:?}"),
        }
    }

    #[test]
    fn out_of_range_number_is_rejected() {
        let err = from_json_str::<u8>("256").unwrap_err();
        match err {
            JsonFromStrError::Decode(e) => assert_eq!(e.kind(), JsonDecodeErrorKind::Number),
            other => panic!("expected decode error, got {other:?}"),
        }
    }

    #[test]
    fn non_integer_number_is_rejected() {
        // `25.0` is numerically 25 but not a plain integer literal; strict.
        let err = from_json_str::<u8>("25.0").unwrap_err();
        assert!(matches!(err, JsonFromStrError::Decode(_)));
    }

    #[test]
    fn invalid_json_is_a_parse_error() {
        let err = from_json_str::<u8>("nope").unwrap_err();
        assert!(matches!(err, JsonFromStrError::Parse(_)));
    }
}

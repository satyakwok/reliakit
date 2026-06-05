//! Typed JSON encoding: turn a Rust value into a [`JsonValue`].
//!
//! [`JsonEncode`] is the encode half of typed JSON serialization. It is
//! deliberately small: a value produces a [`JsonValue`], and the existing
//! [`to_compact_string`](crate::to_compact_string) writer turns that into
//! deterministic, compact JSON text. Encoding never fails — every supported
//! value has a JSON representation.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::number::JsonNumber;
use crate::value::JsonValue;
use crate::write::{to_compact_string, to_compact_vec};

/// A type that can be encoded into a [`JsonValue`].
///
/// Pair with [`to_json_string`] for the canonical compact text, or with
/// [`JsonDecode`](crate::JsonDecode) to round-trip. The derive in
/// `reliakit-derive` generates implementations of this trait.
pub trait JsonEncode {
    /// Encodes `self` into a [`JsonValue`].
    fn to_json_value(&self) -> JsonValue;
}

/// Encodes a value to compact, deterministic JSON text.
pub fn to_json_string<T: JsonEncode + ?Sized>(value: &T) -> String {
    to_compact_string(&value.to_json_value())
}

/// Encodes a value to compact, deterministic JSON bytes.
pub fn to_json_vec<T: JsonEncode + ?Sized>(value: &T) -> Vec<u8> {
    to_compact_vec(&value.to_json_value())
}

macro_rules! impl_int {
    ($($t:ty),* $(,)?) => {$(
        impl JsonEncode for $t {
            fn to_json_value(&self) -> JsonValue {
                // A decimal integer literal is always a valid JSON number.
                JsonValue::Number(JsonNumber::from_validated(self.to_string()))
            }
        }
    )*};
}
impl_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl JsonEncode for bool {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::Bool(*self)
    }
}

impl JsonEncode for str {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::String(self.into())
    }
}

impl JsonEncode for String {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::String(self.clone())
    }
}

impl<T: JsonEncode> JsonEncode for Option<T> {
    fn to_json_value(&self) -> JsonValue {
        match self {
            Some(value) => value.to_json_value(),
            None => JsonValue::Null,
        }
    }
}

impl<T: JsonEncode> JsonEncode for Vec<T> {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::Array(self.iter().map(JsonEncode::to_json_value).collect())
    }
}

impl<T: JsonEncode> JsonEncode for [T] {
    fn to_json_value(&self) -> JsonValue {
        JsonValue::Array(self.iter().map(JsonEncode::to_json_value).collect())
    }
}

impl<T: JsonEncode + ?Sized> JsonEncode for &T {
    fn to_json_value(&self) -> JsonValue {
        (**self).to_json_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::JsonObject;

    #[test]
    fn scalars_encode_to_exact_text() {
        assert_eq!(to_json_string(&255u8), "255");
        assert_eq!(to_json_string(&(-5i32)), "-5");
        assert_eq!(
            to_json_string(&u128::MAX),
            "340282366920938463463374607431768211455"
        );
        assert_eq!(to_json_string(&true), "true");
        assert_eq!(to_json_string(&false), "false");
        assert_eq!(to_json_string("hi"), "\"hi\"");
        assert_eq!(to_json_string(&String::from("hi")), "\"hi\"");
    }

    #[test]
    fn option_encodes_none_as_null() {
        assert_eq!(to_json_string(&Option::<u8>::None), "null");
        assert_eq!(to_json_string(&Some(7u8)), "7");
    }

    #[test]
    fn sequences_encode_to_arrays() {
        assert_eq!(to_json_string(&vec![1u8, 2, 3]), "[1,2,3]");
        assert_eq!(to_json_string(&Vec::<u8>::new()), "[]");
        let slice: &[u8] = &[9, 8];
        assert_eq!(to_json_string(slice), "[9,8]");
    }

    #[test]
    fn object_member_order_is_preserved() {
        // The writer keeps insertion order; encoders rely on this for stable
        // field ordering.
        let mut obj = JsonObject::new();
        obj.insert("b".into(), 2u8.to_json_value());
        obj.insert("a".into(), 1u8.to_json_value());
        assert_eq!(
            to_compact_string(&JsonValue::Object(obj)),
            r#"{"b":2,"a":1}"#
        );
    }
}

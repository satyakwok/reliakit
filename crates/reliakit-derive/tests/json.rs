//! Tests for the `reliakit-json` derives.

use reliakit_derive::{JsonDecode, JsonEncode};
use reliakit_json::{JsonDecodeErrorKind, JsonFromStrError, from_json_str, to_json_string};

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Point {
    x: u16,
    y: u16,
}

#[test]
fn named_struct_exact_bytes_and_roundtrip() {
    let point = Point { x: 10, y: 20 };
    // Object fields in declaration order.
    assert_eq!(to_json_string(&point), r#"{"x":10,"y":20}"#);
    assert_eq!(from_json_str::<Point>(r#"{"x":10,"y":20}"#).unwrap(), point);
}

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Pair(u8, String);

#[test]
fn tuple_struct_is_a_json_array() {
    let pair = Pair(7, "hi".to_string());
    assert_eq!(to_json_string(&pair), r#"[7,"hi"]"#);
    assert_eq!(from_json_str::<Pair>(r#"[7,"hi"]"#).unwrap(), pair);
}

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Marker;

#[test]
fn unit_struct_is_json_null() {
    assert_eq!(to_json_string(&Marker), "null");
    assert_eq!(from_json_str::<Marker>("null").unwrap(), Marker);
}

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Inner {
    a: u8,
}

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Outer {
    inner: Inner,
    tags: Vec<String>,
    note: Option<String>,
}

#[test]
fn nested_and_composite_roundtrip() {
    let outer = Outer {
        inner: Inner { a: 1 },
        tags: vec!["x".to_string(), "y".to_string()],
        note: None,
    };
    assert_eq!(
        to_json_string(&outer),
        r#"{"inner":{"a":1},"tags":["x","y"],"note":null}"#
    );
    let text = to_json_string(&outer);
    assert_eq!(from_json_str::<Outer>(&text).unwrap(), outer);

    let with_note = Outer {
        inner: Inner { a: 9 },
        tags: vec![],
        note: Some("hi".to_string()),
    };
    assert_eq!(
        from_json_str::<Outer>(&to_json_string(&with_note)).unwrap(),
        with_note
    );
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Keyword {
    r#type: u8,
    r#struct: bool,
}

#[test]
fn raw_identifier_fields_use_plain_keys() {
    let value = Keyword {
        r#type: 5,
        r#struct: true,
    };
    // The `r#` prefix is dropped for the JSON key.
    assert_eq!(to_json_string(&value), r#"{"type":5,"struct":true}"#);
    assert_eq!(
        from_json_str::<Keyword>(r#"{"type":5,"struct":true}"#).unwrap(),
        value
    );
}

#[test]
fn missing_field_is_a_decode_error() {
    let err = from_json_str::<Point>(r#"{"x":1}"#).unwrap_err();
    match err {
        JsonFromStrError::Decode(error) => {
            assert_eq!(error.kind(), JsonDecodeErrorKind::MissingField)
        }
        other => panic!("expected a decode error, got {other:?}"),
    }
}

#[test]
fn unknown_fields_are_ignored() {
    assert_eq!(
        from_json_str::<Point>(r#"{"x":1,"y":2,"extra":99}"#).unwrap(),
        Point { x: 1, y: 2 }
    );
}

#[test]
fn wrong_shape_is_a_decode_error() {
    // A Point is an object; an array must be rejected.
    let err = from_json_str::<Point>("[1,2]").unwrap_err();
    assert!(matches!(err, JsonFromStrError::Decode(_)));
    // A tuple struct is an array; an object must be rejected.
    let err = from_json_str::<Pair>(r#"{"0":1}"#).unwrap_err();
    assert!(matches!(err, JsonFromStrError::Decode(_)));
}

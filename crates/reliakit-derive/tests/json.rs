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

#[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
struct Attributed {
    // The trailing comma exercises the attribute parser's empty-item handling.
    #[reliakit(rename = "id")]
    identifier: u32,
    #[reliakit(skip)]
    cached: u8,
    name: String,
}

#[test]
fn rename_changes_the_object_key_and_skip_omits_the_field() {
    let value = Attributed {
        identifier: 7,
        cached: 99,
        name: "x".into(),
    };
    // `identifier` is written as `id`; `cached` is omitted entirely.
    assert_eq!(to_json_string(&value), r#"{"id":7,"name":"x"}"#);
}

#[test]
fn skip_defaults_on_decode_and_rename_round_trips() {
    // The input has no `cached` key; it decodes to `u8::default()` (0).
    let parsed: Attributed = from_json_str(r#"{"id":7,"name":"x"}"#).unwrap();
    assert_eq!(
        parsed,
        Attributed {
            identifier: 7,
            cached: 0,
            name: "x".into(),
        }
    );

    // A full round-trip: the skipped value is lost (re-defaults to 0), the rest
    // survives under the renamed key.
    let value = Attributed {
        identifier: 3,
        cached: 42,
        name: "y".into(),
    };
    let restored: Attributed = from_json_str(&to_json_string(&value)).unwrap();
    assert_eq!(
        restored,
        Attributed {
            identifier: 3,
            cached: 0,
            name: "y".into(),
        }
    );
}

#[test]
fn an_unknown_renamed_key_is_a_missing_field_error() {
    // The wire key is `id`; the original field name `identifier` is not accepted.
    let err = from_json_str::<Attributed>(r#"{"identifier":7,"name":"x"}"#).unwrap_err();
    match err {
        JsonFromStrError::Decode(error) => {
            assert_eq!(error.kind(), JsonDecodeErrorKind::MissingField)
        }
        other => panic!("expected a missing-field decode error, got {other:?}"),
    }
}

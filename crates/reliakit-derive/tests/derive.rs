//! Round-trip tests for the `reliakit-codec` derives.

use reliakit_codec::{CodecErrorKind, decode_from_slice_exact, encode_to_vec};
use reliakit_derive::{CanonicalDecode, CanonicalEncode};

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Point {
    x: u16,
    y: u16,
}

#[test]
fn named_struct_roundtrip() {
    let point = Point { x: 10, y: 20 };
    let bytes = encode_to_vec(&point).unwrap();
    // Same canonical bytes a handwritten impl would produce: little-endian u16s.
    assert_eq!(bytes, [10, 0, 20, 0]);
    assert_eq!(decode_from_slice_exact::<Point>(&bytes).unwrap(), point);
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Header {
    tag: u8,
    flag: bool,
    label: String,
}

#[test]
fn named_struct_mixed_fields_roundtrip() {
    let header = Header {
        tag: 7,
        flag: true,
        label: "ok".to_string(),
    };
    let bytes = encode_to_vec(&header).unwrap();
    assert_eq!(decode_from_slice_exact::<Header>(&bytes).unwrap(), header);
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Pair(u8, u16);

#[test]
fn tuple_struct_roundtrip() {
    let pair = Pair(1, 2);
    let bytes = encode_to_vec(&pair).unwrap();
    assert_eq!(bytes, [1, 2, 0]);
    assert_eq!(decode_from_slice_exact::<Pair>(&bytes).unwrap(), pair);
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Marker;

#[test]
fn unit_struct_roundtrip() {
    let bytes = encode_to_vec(&Marker).unwrap();
    assert!(bytes.is_empty());
    assert_eq!(decode_from_slice_exact::<Marker>(&bytes).unwrap(), Marker);
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Empty {}

#[test]
fn empty_named_struct_roundtrip() {
    let bytes = encode_to_vec(&Empty {}).unwrap();
    assert!(bytes.is_empty());
    assert_eq!(decode_from_slice_exact::<Empty>(&bytes).unwrap(), Empty {});
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Single {
    only: u64,
}

#[test]
fn single_field_struct_roundtrip() {
    let value = Single { only: 42 };
    let bytes = encode_to_vec(&value).unwrap();
    assert_eq!(decode_from_slice_exact::<Single>(&bytes).unwrap(), value);
}

/// A documented, public struct with attributes, a `pub` field, and a raw
/// identifier field. This exercises the parser's skipping of outer attributes,
/// visibility, and field attributes, and its handling of raw identifiers.
#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
pub struct Documented {
    /// The leading tag byte.
    pub tag: u8,
    #[allow(dead_code)]
    pub value: u32,
    pub r#type: u8,
}

#[test]
fn attributed_public_raw_ident_struct_roundtrip() {
    let value = Documented {
        tag: 1,
        value: 2,
        r#type: 3,
    };
    let bytes = encode_to_vec(&value).unwrap();
    assert_eq!(
        decode_from_slice_exact::<Documented>(&bytes).unwrap(),
        value
    );
}

/// A public tuple struct with a `pub` field, exercising tuple field counting in
/// the presence of visibility tokens.
#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
pub struct Wrapped(pub u8, pub u16);

#[test]
fn public_tuple_struct_roundtrip() {
    let value = Wrapped(5, 6);
    let bytes = encode_to_vec(&value).unwrap();
    assert_eq!(bytes, [5, 6, 0]);
    assert_eq!(decode_from_slice_exact::<Wrapped>(&bytes).unwrap(), value);
}

// ----------------------------------------------------------------------------
// Enums
// ----------------------------------------------------------------------------

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Signal {
    Ping,
    Pong,
}

#[test]
fn unit_enum_encode_exact_bytes() {
    // Variant tag is the zero-based declaration index as a little-endian u32.
    assert_eq!(encode_to_vec(&Signal::Ping).unwrap(), [0, 0, 0, 0]);
    assert_eq!(encode_to_vec(&Signal::Pong).unwrap(), [1, 0, 0, 0]);
}

#[test]
fn unit_enum_decode() {
    assert_eq!(
        decode_from_slice_exact::<Signal>(&[0, 0, 0, 0]).unwrap(),
        Signal::Ping
    );
    assert_eq!(
        decode_from_slice_exact::<Signal>(&[1, 0, 0, 0]).unwrap(),
        Signal::Pong
    );
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Command {
    SetPort(u16),
    SetName(String),
}

#[test]
fn tuple_enum_encode_exact_bytes() {
    // tag 0 (u32 LE) then u16 8080 little-endian.
    assert_eq!(
        encode_to_vec(&Command::SetPort(8080)).unwrap(),
        [0, 0, 0, 0, 0x90, 0x1f]
    );
    // tag 1 (u32 LE) then string ("hi": u32 length 2, then bytes).
    assert_eq!(
        encode_to_vec(&Command::SetName("hi".to_string())).unwrap(),
        [1, 0, 0, 0, 2, 0, 0, 0, b'h', b'i']
    );
}

#[test]
fn tuple_enum_decode() {
    assert_eq!(
        decode_from_slice_exact::<Command>(&[0, 0, 0, 0, 0x90, 0x1f]).unwrap(),
        Command::SetPort(8080)
    );
    assert_eq!(
        decode_from_slice_exact::<Command>(&[1, 0, 0, 0, 2, 0, 0, 0, b'h', b'i']).unwrap(),
        Command::SetName("hi".to_string())
    );
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Event {
    UserCreated { id: u64, name: String },
    UserDeleted { id: u64 },
}

#[test]
fn struct_enum_encode_exact_bytes() {
    // tag 0, then u64 id (LE), then string name.
    assert_eq!(
        encode_to_vec(&Event::UserCreated {
            id: 1,
            name: "a".to_string(),
        })
        .unwrap(),
        [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, b'a']
    );
    // tag 1, then u64 id (LE).
    assert_eq!(
        encode_to_vec(&Event::UserDeleted { id: 5 }).unwrap(),
        [1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn struct_enum_decode() {
    assert_eq!(
        decode_from_slice_exact::<Event>(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, b'a'])
            .unwrap(),
        Event::UserCreated {
            id: 1,
            name: "a".to_string(),
        }
    );
    assert_eq!(
        decode_from_slice_exact::<Event>(&[1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0]).unwrap(),
        Event::UserDeleted { id: 5 }
    );
}

#[test]
fn unknown_tag_rejected() {
    // Signal only has tags 0 and 1; tag 9 must be rejected.
    let err = decode_from_slice_exact::<Signal>(&[9, 0, 0, 0]).unwrap_err();
    assert_eq!(err.kind(), CodecErrorKind::InvalidValue);
}

#[test]
fn truncated_tag_rejected() {
    // Fewer than four bytes cannot hold the u32 tag.
    let err = decode_from_slice_exact::<Signal>(&[0, 0]).unwrap_err();
    assert_eq!(err.kind(), CodecErrorKind::UnexpectedEof);
}

#[test]
fn truncated_tuple_payload_rejected() {
    // tag 0 (SetPort) present, but only one of the u16's two bytes follows.
    let err = decode_from_slice_exact::<Command>(&[0, 0, 0, 0, 0x90]).unwrap_err();
    assert_eq!(err.kind(), CodecErrorKind::UnexpectedEof);
}

#[test]
fn truncated_struct_payload_rejected() {
    // tag 1 (UserDeleted) present, but only three of the u64's eight bytes follow.
    let err = decode_from_slice_exact::<Event>(&[1, 0, 0, 0, 5, 0, 0]).unwrap_err();
    assert_eq!(err.kind(), CodecErrorKind::UnexpectedEof);
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Many {
    A,
    B,
    C,
    D,
}

#[test]
fn declaration_order_tags_preserved() {
    assert_eq!(encode_to_vec(&Many::A).unwrap(), [0, 0, 0, 0]);
    assert_eq!(encode_to_vec(&Many::B).unwrap(), [1, 0, 0, 0]);
    assert_eq!(encode_to_vec(&Many::C).unwrap(), [2, 0, 0, 0]);
    assert_eq!(encode_to_vec(&Many::D).unwrap(), [3, 0, 0, 0]);
}

#[test]
fn decode_returns_correct_variant() {
    assert_eq!(
        decode_from_slice_exact::<Many>(&[2, 0, 0, 0]).unwrap(),
        Many::C
    );
    assert_eq!(
        decode_from_slice_exact::<Many>(&[3, 0, 0, 0]).unwrap(),
        Many::D
    );
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
pub enum Mixed {
    Empty,
    One(u8),
    Two(u16, u8),
    Record { a: u8, b: u16 },
}

#[test]
fn mixed_enum_roundtrip() {
    for value in [
        Mixed::Empty,
        Mixed::One(7),
        Mixed::Two(1000, 9),
        Mixed::Record { a: 3, b: 600 },
    ] {
        let bytes = encode_to_vec(&value).unwrap();
        assert_eq!(decode_from_slice_exact::<Mixed>(&bytes).unwrap(), value);
    }
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Inner {
    a: u8,
    b: u16,
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Nested {
    Wrap(Inner),
    Pair { left: Inner, right: u8 },
    List(Vec<u8>),
    Maybe(Option<u16>),
}

#[test]
fn nested_and_composite_roundtrip() {
    // A derived struct used as a field, and codec's own composite types
    // (`Vec`, `Option`), compose through the derived enum.
    for value in [
        Nested::Wrap(Inner { a: 1, b: 2 }),
        Nested::Pair {
            left: Inner { a: 3, b: 4 },
            right: 5,
        },
        Nested::List(vec![10, 20, 30]),
        Nested::Maybe(Some(700)),
        Nested::Maybe(None),
    ] {
        let bytes = encode_to_vec(&value).unwrap();
        assert_eq!(decode_from_slice_exact::<Nested>(&bytes).unwrap(), value);
    }
}

// Raw-identifier variants are necessarily keywords, hence lower case.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Keywords {
    r#type,
    r#match(u8),
    Pair { r#fn: u8, r#struct: u16 },
}

#[test]
fn raw_identifier_variants_and_fields_roundtrip() {
    for value in [
        Keywords::r#type,
        Keywords::r#match(9),
        Keywords::Pair {
            r#fn: 1,
            r#struct: 2,
        },
    ] {
        let bytes = encode_to_vec(&value).unwrap();
        assert_eq!(decode_from_slice_exact::<Keywords>(&bytes).unwrap(), value);
    }
}

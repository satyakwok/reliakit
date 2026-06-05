//! Round-trip tests for the `reliakit-codec` derives.

use reliakit_codec::{decode_from_slice_exact, encode_to_vec};
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

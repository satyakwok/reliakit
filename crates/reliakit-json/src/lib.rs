//! Strict, bounded, and deterministic JSON for reliability-sensitive Rust.
//!
//! `reliakit-json` is built for systems that process **untrusted** JSON or need
//! **predictable** output: it parses a strict subset of [RFC 8259], rejects
//! duplicate object keys, enforces explicit [resource limits](JsonLimits),
//! preserves number precision, reports errors with location and path, and
//! serializes deterministically. It has no external dependencies, forbids
//! unsafe code, and supports `no_std` (with `alloc`).
//!
//! It deliberately does **not** provide derive macros, schema validation,
//! JSON5, comments, trailing commas, lenient parsing, or SIMD throughput.
//!
//! # Example
//!
//! ```
//! use reliakit_json::{parse_str, to_compact_string};
//!
//! let value = parse_str(r#"{"name":"reliakit","ok":true}"#).unwrap();
//! assert_eq!(value.as_object().unwrap().get("name").unwrap().as_str(), Some("reliakit"));
//!
//! // Serialization is deterministic and preserves member order.
//! assert_eq!(to_compact_string(&value), r#"{"name":"reliakit","ok":true}"#);
//!
//! // Strict by default: duplicate keys are rejected, not silently resolved.
//! assert!(parse_str(r#"{"a":1,"a":2}"#).is_err());
//! ```
//!
//! # Limits
//!
//! [`parse`] applies conservative [`JsonLimits`] by default. Use
//! [`parse_with_limits`] to choose a profile or tune individual limits:
//!
//! ```
//! use reliakit_json::{parse_with_limits, JsonLimits};
//!
//! let limits = JsonLimits::conservative().with_max_depth(8);
//! assert!(parse_with_limits(b"[[[[[[[[[[1]]]]]]]]]]", limits).is_err());
//! ```
//!
//! [RFC 8259]: https://www.rfc-editor.org/rfc/rfc8259

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

mod error;
mod limits;
mod number;
mod parse;
mod value;
mod write;

pub use error::{
    JsonError, JsonErrorKind, JsonLimitKind, JsonNumberError, JsonPath, JsonPathSegment,
};
pub use limits::JsonLimits;
pub use number::JsonNumber;
pub use parse::{parse, parse_str, parse_with_limits};
pub use value::{JsonMember, JsonObject, JsonValue};
pub use write::{to_compact_string, to_compact_vec};

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    fn parse_ok(input: &str) -> JsonValue {
        parse_str(input).expect("should parse")
    }

    fn kind(input: &str) -> JsonErrorKind {
        parse_str(input).expect_err("should fail").kind().clone()
    }

    // ---- scalars ----------------------------------------------------------

    #[test]
    fn parses_scalars() {
        assert_eq!(parse_ok("null"), JsonValue::Null);
        assert_eq!(parse_ok("true"), JsonValue::Bool(true));
        assert_eq!(parse_ok("false"), JsonValue::Bool(false));
        assert_eq!(parse_ok("\"hi\"").as_str(), Some("hi"));
        assert_eq!(parse_ok("42").as_number().unwrap().to_i64().unwrap(), 42);
    }

    #[test]
    fn whitespace_is_allowed_around_values() {
        assert_eq!(parse_ok("  \t\r\n 7 \n").as_number().unwrap().as_str(), "7");
    }

    #[test]
    fn only_json_whitespace_is_accepted() {
        // A vertical tab (U+000B) is not JSON whitespace.
        assert_eq!(kind("\u{0B}1"), JsonErrorKind::UnexpectedByte);
    }

    // ---- structure --------------------------------------------------------

    #[test]
    fn parses_object_and_array() {
        let value = parse_ok(r#"{"a":[1,2,3],"b":{"c":null}}"#);
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj.get("a").unwrap().as_array().unwrap().len(), 3);
        assert!(obj
            .get("b")
            .unwrap()
            .as_object()
            .unwrap()
            .get("c")
            .unwrap()
            .is_null());
    }

    #[test]
    fn empty_containers() {
        assert_eq!(parse_ok("[]").as_array().unwrap().len(), 0);
        assert_eq!(parse_ok("{}").as_object().unwrap().len(), 0);
    }

    // ---- required rejections ---------------------------------------------

    #[test]
    fn rejects_trailing_data() {
        assert_eq!(kind("1 2"), JsonErrorKind::TrailingData);
        assert_eq!(kind("{} x"), JsonErrorKind::TrailingData);
    }

    #[test]
    fn rejects_comments_and_trailing_commas() {
        assert_eq!(kind("1 // c"), JsonErrorKind::TrailingData);
        assert_eq!(kind("[1,]"), JsonErrorKind::UnexpectedByte);
        assert_eq!(kind(r#"{"a":1,}"#), JsonErrorKind::UnexpectedByte);
    }

    #[test]
    fn rejects_bad_numbers() {
        for bad in ["01", "1.", "-", "1e", "1e+", "00", "1.2.3"] {
            assert_eq!(kind(bad), JsonErrorKind::InvalidNumber, "input {bad:?}");
        }
        // Also rejected, with their own correct kinds (no valid value starts
        // with '.' or '+'; "0x1" parses "0" then chokes on the trailing "x1").
        assert_eq!(kind(".5"), JsonErrorKind::UnexpectedByte);
        assert_eq!(kind("+1"), JsonErrorKind::UnexpectedByte);
        assert_eq!(kind("0x1"), JsonErrorKind::TrailingData);
    }

    #[test]
    fn rejects_nan_and_infinity() {
        assert_eq!(kind("NaN"), JsonErrorKind::UnexpectedByte);
        assert_eq!(kind("Infinity"), JsonErrorKind::UnexpectedByte);
        assert_eq!(kind("-Infinity"), JsonErrorKind::InvalidNumber);
    }

    #[test]
    fn rejects_unescaped_control_and_bad_escapes() {
        assert_eq!(kind("\"\u{01}\""), JsonErrorKind::UnescapedControlCharacter);
        assert_eq!(kind(r#""\x""#), JsonErrorKind::InvalidEscape);
        assert_eq!(kind(r#""\u00""#), JsonErrorKind::InvalidUnicodeEscape);
    }

    #[test]
    fn rejects_lone_surrogates() {
        assert_eq!(kind(r#""\uD800""#), JsonErrorKind::LoneSurrogate);
        assert_eq!(kind(r#""\uDC00""#), JsonErrorKind::LoneSurrogate);
        assert_eq!(kind(r#""\uD800a""#), JsonErrorKind::LoneSurrogate);
    }

    #[test]
    fn accepts_valid_surrogate_pair() {
        assert_eq!(parse_ok(r#""𝄞""#).as_str(), Some("\u{1D11E}"));
    }

    #[test]
    fn rejects_invalid_utf8_and_bom() {
        assert_eq!(
            parse(&[0xff]).unwrap_err().kind().clone(),
            JsonErrorKind::InvalidUtf8
        );
        assert_eq!(
            parse(&[0xEF, 0xBB, 0xBF, b'1']).unwrap_err().kind().clone(),
            JsonErrorKind::InvalidUtf8
        );
    }

    // ---- string semantics -------------------------------------------------

    #[test]
    fn escape_and_literal_decode_equally() {
        assert_eq!(parse_ok(r#""a""#), parse_ok(r#""a""#));
    }

    #[test]
    fn decodes_named_escapes() {
        assert_eq!(
            parse_ok(r#""\n\t\r\b\f\"\\\/""#).as_str(),
            Some("\n\t\r\u{08}\u{0C}\"\\/")
        );
    }

    // ---- duplicate keys ---------------------------------------------------

    #[test]
    fn rejects_duplicate_keys() {
        assert_eq!(kind(r#"{"a":1,"a":2}"#), JsonErrorKind::DuplicateKey);
    }

    #[test]
    fn duplicate_detection_is_after_escape_decoding() {
        assert_eq!(
            kind(r#"{"role":"user","role":"admin"}"#),
            JsonErrorKind::DuplicateKey
        );
    }

    // ---- limits -----------------------------------------------------------

    #[test]
    fn enforces_depth_limit() {
        let limits = JsonLimits::new().with_max_depth(3);
        assert!(parse_with_limits(b"[[[1]]]", limits).is_ok());
        assert_eq!(
            parse_with_limits(b"[[[[1]]]]", limits)
                .unwrap_err()
                .kind()
                .clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::Depth)
        );
    }

    #[test]
    fn enforces_count_limits() {
        let limits = JsonLimits::new();
        let limits = JsonLimits {
            max_array_items: 2,
            max_object_members: 2,
            max_total_nodes: 100,
            ..limits
        };
        assert_eq!(
            parse_with_limits(b"[1,2,3]", limits)
                .unwrap_err()
                .kind()
                .clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::ArrayItems)
        );
        assert_eq!(
            parse_with_limits(br#"{"a":1,"b":2,"c":3}"#, limits)
                .unwrap_err()
                .kind()
                .clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::ObjectMembers)
        );
    }

    #[test]
    fn enforces_total_nodes_and_input_bytes() {
        let nodes = JsonLimits::new().with_max_total_nodes(2);
        assert_eq!(
            parse_with_limits(b"[1,2]", nodes)
                .unwrap_err()
                .kind()
                .clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::TotalNodes)
        );
        let bytes = JsonLimits::new().with_max_input_bytes(2);
        assert_eq!(
            parse_with_limits(b"[1]", bytes).unwrap_err().kind().clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::InputBytes)
        );
    }

    #[test]
    fn enforces_string_and_number_byte_limits() {
        let s = JsonLimits {
            max_string_bytes: 3,
            ..JsonLimits::new()
        };
        assert_eq!(
            parse_with_limits(br#""abcd""#, s)
                .unwrap_err()
                .kind()
                .clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::StringBytes)
        );
        let n = JsonLimits {
            max_number_bytes: 2,
            ..JsonLimits::new()
        };
        assert_eq!(
            parse_with_limits(b"12345", n).unwrap_err().kind().clone(),
            JsonErrorKind::LimitExceeded(JsonLimitKind::NumberBytes)
        );
    }

    // ---- numbers ----------------------------------------------------------

    #[test]
    fn number_conversions() {
        assert_eq!(parse_ok("-7").as_number().unwrap().to_i64().unwrap(), -7);
        assert_eq!(parse_ok("7").as_number().unwrap().to_u64().unwrap(), 7);
        assert!((parse_ok("1.5").as_number().unwrap().to_f64().unwrap() - 1.5).abs() < 1e-12);
        assert_eq!(
            parse_ok("1.5").as_number().unwrap().to_i64(),
            Err(JsonNumberError::NotAnInteger)
        );
        assert_eq!(
            parse_ok("99999999999999999999999")
                .as_number()
                .unwrap()
                .to_i64(),
            Err(JsonNumberError::OutOfRange)
        );
        assert_eq!(
            parse_ok("1e400").as_number().unwrap().to_f64(),
            Err(JsonNumberError::NotFinite)
        );
    }

    #[test]
    fn number_preserves_representation() {
        assert_eq!(parse_ok("1.0").as_number().unwrap().as_str(), "1.0");
        assert_ne!(parse_ok("1.0"), parse_ok("1")); // structural equality
    }

    #[test]
    fn json_number_from_f64() {
        assert_eq!(JsonNumber::try_from_f64(1.5).unwrap().as_str(), "1.5");
        assert_eq!(
            JsonNumber::try_from_f64(f64::NAN),
            Err(JsonNumberError::NotFinite)
        );
        assert_eq!(
            JsonNumber::try_from_f64(f64::INFINITY),
            Err(JsonNumberError::NotFinite)
        );
        assert_eq!(JsonNumber::new("01"), Err(JsonNumberError::InvalidNumber));
    }

    // ---- errors -----------------------------------------------------------

    #[test]
    fn error_reports_location_and_path() {
        let err = parse_str("  @").unwrap_err();
        assert_eq!(err.kind().clone(), JsonErrorKind::UnexpectedByte);
        assert_eq!(err.offset(), 2);
        assert_eq!(err.line(), 1);
        assert_eq!(err.column(), 3);

        let err = parse_str(r#"{"users":[{"name":1},{"name":}]}"#).unwrap_err();
        let path = err.path().unwrap().to_string();
        assert_eq!(path, "$.users[1].name");
    }

    // ---- serialization ----------------------------------------------------

    #[test]
    fn compact_roundtrip_and_golden_bytes() {
        let value = parse_ok(r#"{"a":1,"b":true,"c":[null,"x"]}"#);
        assert_eq!(
            to_compact_vec(&value),
            br#"{"a":1,"b":true,"c":[null,"x"]}"#
        );
        // Roundtrip: serialize, reparse, equal value.
        let again = parse_str(&to_compact_string(&value)).unwrap();
        assert_eq!(value, again);
    }

    #[test]
    fn writer_escapes_control_and_special_characters() {
        let mut object = JsonObject::new();
        object.insert(
            String::from("k"),
            JsonValue::String(String::from("a\nb\"c\\\u{01}")),
        );
        let value = JsonValue::Object(object);
        assert_eq!(to_compact_string(&value), r#"{"k":"a\nb\"c\\\u0001"}"#);
    }

    #[test]
    fn object_insert_replaces_in_place() {
        let mut object = JsonObject::new();
        assert!(object
            .insert(String::from("a"), JsonValue::Bool(true))
            .is_none());
        let old = object.insert(String::from("a"), JsonValue::Bool(false));
        assert_eq!(old, Some(JsonValue::Bool(true)));
        assert_eq!(object.len(), 1);
    }

    #[test]
    fn deeply_nested_within_limits_does_not_overflow() {
        // Build input nested to the default limit and confirm bounded handling.
        let depth = 64;
        let mut s = String::new();
        for _ in 0..depth {
            s.push('[');
        }
        s.push('1');
        for _ in 0..depth {
            s.push(']');
        }
        // Default max_depth is 64, so depth 64 is at the edge; depth 65 fails.
        let _ = parse_str(&s); // must not panic regardless of accept/reject
        assert!(parse_with_limits(s.as_bytes(), JsonLimits::new().with_max_depth(64)).is_ok());
    }

    #[test]
    fn arbitrary_bytes_never_panic() {
        // Smoke test: a spread of odd inputs must each return Ok or Err, never panic.
        for input in [
            &b""[..],
            b"   ",
            b"{",
            b"[",
            b"\"",
            b"\"\\",
            b"\"\\u",
            b"tru",
            b"-",
            b"[,]",
            b"{,}",
            b"\xff\xfe",
            b"[[[",
            b"}}}",
            b"\"\\uD800\"",
            b"1e",
            b"{\"a\"}",
        ] {
            let _ = parse(input);
        }
    }
}

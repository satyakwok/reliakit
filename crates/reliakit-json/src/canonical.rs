//! Canonical JSON serialization following RFC 8785 (JSON Canonicalization
//! Scheme, JCS).
//!
//! **Experimental.** Enabled by the `canonical` feature. The output has not yet
//! been validated against the full RFC 8785 conformance vectors and the API may
//! change before it is declared stable.
//!
//! JCS produces a single, deterministic byte sequence for a JSON value so it can
//! be hashed or signed: object members are sorted by the UTF-16 code units of
//! their names, insignificant whitespace is removed, strings use minimal
//! escaping, and numbers are emitted in the shortest round-tripping form defined
//! by ECMAScript `Number.prototype.toString`.
//!
//! Numbers are treated as IEEE-754 doubles, exactly as RFC 8785 specifies. A
//! value carrying more precision than an `f64` can hold (for example an integer
//! larger than 2^53) is canonicalized as the nearest double — this loss of
//! precision is part of the scheme, not a bug. A number whose magnitude
//! overflows `f64` to infinity cannot be represented and returns
//! [`JsonErrorKind::NonFiniteNumber`](crate::JsonErrorKind::NonFiniteNumber).
//!
//! ```
//! use reliakit_json::{parse_str, to_canonical_string};
//!
//! let value = parse_str(r#"{ "b": 1, "a": 1.0 }"#).unwrap();
//! // Keys sorted, no whitespace, 1.0 normalized to 1.
//! assert_eq!(to_canonical_string(&value).unwrap(), r#"{"a":1,"b":1}"#);
//! ```

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{JsonError, JsonErrorKind};
use crate::value::{JsonMember, JsonValue};
use crate::write::write_escaped;

/// Serializes a value to its RFC 8785 (JCS) canonical form as a `String`.
///
/// Returns [`JsonErrorKind::NonFiniteNumber`](crate::JsonErrorKind::NonFiniteNumber)
/// if a number cannot be represented as a finite IEEE-754 `f64`.
///
/// **Experimental** (`canonical` feature); see the [module docs](self).
pub fn to_canonical_string(value: &JsonValue) -> Result<String, JsonError> {
    let mut out = String::new();
    write_canonical(&mut out, value)?;
    Ok(out)
}

/// Serializes a value to its RFC 8785 (JCS) canonical form as UTF-8 bytes.
///
/// See [`to_canonical_string`].
pub fn to_canonical_vec(value: &JsonValue) -> Result<Vec<u8>, JsonError> {
    Ok(to_canonical_string(value)?.into_bytes())
}

fn write_canonical(out: &mut String, value: &JsonValue) -> Result<(), JsonError> {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Number(number) => {
            let x = match number.as_str().parse::<f64>() {
                Ok(value) if value.is_finite() => value,
                _ => return Err(JsonError::serialization(JsonErrorKind::NonFiniteNumber)),
            };
            out.push_str(&ecmascript_number_to_string(x));
        }
        JsonValue::String(string) => write_escaped(out, string),
        JsonValue::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical(out, item)?;
            }
            out.push(']');
        }
        JsonValue::Object(object) => {
            out.push('{');
            // RFC 8785: sort members by the UTF-16 code units of their names.
            let mut members: Vec<&JsonMember> = object.iter().collect();
            members.sort_by(|a, b| a.key().encode_utf16().cmp(b.key().encode_utf16()));
            for (index, member) in members.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_escaped(out, member.key());
                out.push(':');
                write_canonical(out, member.value())?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// Formats a finite `f64` exactly as ECMAScript `Number.prototype.toString`
/// (RFC 8785 §3.2.2.3): the shortest decimal that round-trips, laid out with the
/// spec's thresholds for fixed vs. exponential notation.
fn ecmascript_number_to_string(x: f64) -> String {
    // Covers both +0.0 and -0.0; RFC 8785 emits "0" for negative zero.
    if x == 0.0 {
        return String::from("0");
    }

    let negative = x < 0.0;
    let abs = if negative { -x } else { x };

    // Rust's shortest scientific form gives the minimal significant digits and a
    // base-10 exponent: "d[.ddd]e[-]EXP" (lowercase e, no '+', no leading zeros).
    let sci = alloc::format!("{abs:e}");
    let (mantissa, exp_str) = match sci.split_once('e') {
        Some(parts) => parts,
        None => return sci, // unreachable for a finite, non-zero value
    };
    let exp: i32 = exp_str.parse().unwrap_or(0);

    // `digits` are the significant digits (no '.'), `k` their count. The value is
    // `digits * 10^(n - k)`, so from "d.rest e EXP" we have `n = EXP + 1`.
    let mut digits = String::with_capacity(mantissa.len());
    for c in mantissa.chars() {
        if c != '.' {
            digits.push(c);
        }
    }
    let k = digits.len() as i32;
    let n = exp + 1;

    let mut out = String::new();
    if negative {
        out.push('-');
    }

    if k <= n && n <= 21 {
        // Integer, padded with trailing zeros.
        out.push_str(&digits);
        for _ in 0..(n - k) {
            out.push('0');
        }
    } else if 0 < n && n <= 21 {
        // Decimal point inside the digit run (here n < k).
        out.push_str(&digits[..n as usize]);
        out.push('.');
        out.push_str(&digits[n as usize..]);
    } else if -6 < n && n <= 0 {
        // 0.00…digits
        out.push_str("0.");
        for _ in 0..(-n) {
            out.push('0');
        }
        out.push_str(&digits);
    } else {
        // Exponential form.
        out.push_str(&digits[..1]);
        if k > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('e');
        let e = n - 1;
        out.push(if e >= 0 { '+' } else { '-' });
        let magnitude = if e >= 0 { e } else { -e };
        out.push_str(&alloc::format!("{magnitude}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_str;

    #[test]
    fn number_formatting_matches_ecmascript() {
        let cases: &[(f64, &str)] = &[
            (0.0, "0"),
            (-0.0, "0"),
            (1.0, "1"),
            (-1.0, "-1"),
            (1.5, "1.5"),
            (-1.5, "-1.5"),
            (10.0, "10"),
            (100.0, "100"),
            (1234.5, "1234.5"),
            (0.1, "0.1"),
            (0.0001, "0.0001"),
            (1e-6, "0.000001"),
            (1e-7, "1e-7"),
            (1e20, "100000000000000000000"),
            (1e21, "1e+21"),
            (1e22, "1e+22"),
            (9007199254740992.0, "9007199254740992"),
            (5e-324, "5e-324"),
        ];
        for &(input, expected) in cases {
            assert_eq!(
                ecmascript_number_to_string(input),
                expected,
                "for {input:?}"
            );
        }
    }

    #[test]
    fn canonicalizes_numbers_from_json() {
        let value = parse_str("[1.0, 1.50, 100, 0.1, 1e2]").unwrap();
        assert_eq!(to_canonical_string(&value).unwrap(), "[1,1.5,100,0.1,100]");
    }

    #[test]
    fn sorts_object_keys_by_utf16_code_units() {
        // "ﬀ" is U+FB00, "😀" is U+1F600. By scalar value FB00 < 1F600, but in
        // UTF-16 "😀" starts with surrogate 0xD83D < 0xFB00, so it sorts first.
        let value = parse_str("{\"\u{FB00}\":1,\"\u{1F600}\":2,\"a\":3}").unwrap();
        let canonical = to_canonical_string(&value).unwrap();
        // ASCII "a" (0x61) first, then 😀, then ﬀ.
        assert_eq!(canonical, "{\"a\":3,\"\u{1F600}\":2,\"\u{FB00}\":1}");
    }

    #[test]
    fn strips_whitespace_and_sorts_nested() {
        let value = parse_str(r#"{ "b" : [ 3 , 2 ] , "a" : { "y" : 1 , "x" : 2 } }"#).unwrap();
        assert_eq!(
            to_canonical_string(&value).unwrap(),
            r#"{"a":{"x":2,"y":1},"b":[3,2]}"#
        );
    }

    #[test]
    fn string_escaping_is_minimal() {
        // Tab and newline use short escapes; non-ASCII stays raw UTF-8.
        let value = parse_str("{\"k\":\"a\\tb\\n€\"}").unwrap();
        assert_eq!(
            to_canonical_string(&value).unwrap(),
            "{\"k\":\"a\\tb\\n€\"}"
        );
    }

    #[test]
    fn is_idempotent() {
        let value = parse_str(r#"{"b":1,"a":1.0,"c":[2.50,3]}"#).unwrap();
        let once = to_canonical_string(&value).unwrap();
        let reparsed = parse_str(&once).unwrap();
        assert_eq!(to_canonical_string(&reparsed).unwrap(), once);
        assert_eq!(once, r#"{"a":1,"b":1,"c":[2.5,3]}"#);
    }

    #[test]
    fn to_vec_matches_string() {
        let value = parse_str(r#"{"a":1}"#).unwrap();
        let s = to_canonical_string(&value).unwrap();
        assert_eq!(to_canonical_vec(&value).unwrap(), s.into_bytes());
    }

    #[test]
    fn rejects_numbers_that_overflow_f64() {
        let value = parse_str("1e400").unwrap(); // valid JSON token, infinite as f64
        let err = to_canonical_string(&value).unwrap_err();
        assert_eq!(err.kind(), &JsonErrorKind::NonFiniteNumber);
    }
}

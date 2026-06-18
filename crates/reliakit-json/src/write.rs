//! Compact, deterministic serialization.

use alloc::string::String;
use alloc::vec::Vec;

use crate::value::JsonValue;

/// Serializes a value to a compact JSON string (no insignificant whitespace).
///
/// Output is deterministic for a given value: object members are emitted in
/// stored (insertion) order and numbers keep their exact representation. This
/// is *not* the canonical (RFC 8785) form: it does not sort keys or reformat
/// numbers. In-memory serialization cannot fail, so this is infallible.
pub fn to_compact_string(value: &JsonValue) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

/// Serializes a value to compact JSON bytes. See [`to_compact_string`].
pub fn to_compact_vec(value: &JsonValue) -> Vec<u8> {
    to_compact_string(value).into_bytes()
}

fn write_value(out: &mut String, value: &JsonValue) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Number(number) => out.push_str(number.as_str()),
        JsonValue::String(string) => write_escaped(out, string),
        JsonValue::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(out, item);
            }
            out.push(']');
        }
        JsonValue::Object(object) => {
            out.push('{');
            for (index, member) in object.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_escaped(out, member.key());
                out.push(':');
                write_value(out, member.value());
            }
            out.push('}');
        }
    }
}

pub(crate) fn write_escaped(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str("\\u");
                let code = c as u32;
                for shift in [12u32, 8, 4, 0] {
                    let nibble = ((code >> shift) & 0xF) as u8;
                    let hex = if nibble < 10 {
                        b'0' + nibble
                    } else {
                        b'a' + nibble - 10
                    };
                    out.push(hex as char);
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

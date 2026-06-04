//! Precision-preserving JSON numbers.

use alloc::boxed::Box;
use alloc::string::{String, ToString};

use crate::error::JsonNumberError;

/// A JSON number that preserves its exact, validated source representation.
///
/// Parsing never silently rounds or truncates: the original token is kept
/// verbatim and conversions to `i64`/`u64`/`f64` are explicit and fallible.
/// Equality is **structural** over the representation — `1.0`, `1`, and `1e0`
/// are distinct `JsonNumber`s. Compare numerically by converting first.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsonNumber {
    repr: Box<str>,
}

impl JsonNumber {
    /// Creates a number from a string, validating it against the strict JSON
    /// number grammar (no leading `+`, no leading zeros, no `NaN`/`Infinity`).
    pub fn new(input: &str) -> Result<Self, JsonNumberError> {
        if is_valid_json_number(input) {
            Ok(Self { repr: input.into() })
        } else {
            Err(JsonNumberError::InvalidNumber)
        }
    }

    /// Creates a number from a token already validated by the parser.
    pub(crate) fn from_validated(repr: String) -> Self {
        Self {
            repr: repr.into_boxed_str(),
        }
    }

    /// Returns the exact source representation.
    pub fn as_str(&self) -> &str {
        &self.repr
    }

    /// Returns `true` if the representation has no fraction or exponent.
    pub fn is_integer(&self) -> bool {
        !self
            .repr
            .bytes()
            .any(|b| b == b'.' || b == b'e' || b == b'E')
    }

    /// Converts to `i64`, or fails if the value is not an integer or is out of
    /// range.
    pub fn to_i64(&self) -> Result<i64, JsonNumberError> {
        if !self.is_integer() {
            return Err(JsonNumberError::NotAnInteger);
        }
        self.repr
            .parse::<i64>()
            .map_err(|_| JsonNumberError::OutOfRange)
    }

    /// Converts to `u64`, or fails if the value is not a non-negative integer or
    /// is out of range.
    pub fn to_u64(&self) -> Result<u64, JsonNumberError> {
        if !self.is_integer() {
            return Err(JsonNumberError::NotAnInteger);
        }
        self.repr
            .parse::<u64>()
            .map_err(|_| JsonNumberError::OutOfRange)
    }

    /// Converts to `f64`. Fails only if the magnitude overflows `f64` to
    /// infinity; ordinary rounding of the decimal value is not an error.
    pub fn to_f64(&self) -> Result<f64, JsonNumberError> {
        match self.repr.parse::<f64>() {
            Ok(value) if value.is_finite() => Ok(value),
            _ => Err(JsonNumberError::NotFinite),
        }
    }

    /// Builds a number from an `f64`. Fails if the value is `NaN` or infinite.
    pub fn try_from_f64(value: f64) -> Result<Self, JsonNumberError> {
        if !value.is_finite() {
            return Err(JsonNumberError::NotFinite);
        }
        let repr = value.to_string();
        // `f64::to_string` produces a valid JSON number for every finite value,
        // but validate defensively so the invariant always holds.
        if is_valid_json_number(&repr) {
            Ok(Self {
                repr: repr.into_boxed_str(),
            })
        } else {
            Err(JsonNumberError::InvalidNumber)
        }
    }
}

/// Validates a string against the RFC 8259 number grammar:
/// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`.
pub(crate) fn is_valid_json_number(s: &str) -> bool {
    let b = s.as_bytes();
    let n = b.len();
    let mut i = 0;

    if i < n && b[i] == b'-' {
        i += 1;
    }

    // Integer part: a single `0`, or a non-zero digit followed by digits.
    match b.get(i) {
        Some(b'0') => i += 1,
        Some(d) if d.is_ascii_digit() => {
            i += 1;
            while i < n && b[i].is_ascii_digit() {
                i += 1;
            }
        }
        _ => return false,
    }

    // Optional fraction.
    if i < n && b[i] == b'.' {
        i += 1;
        let start = i;
        while i < n && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return false;
        }
    }

    // Optional exponent.
    if i < n && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < n && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        let start = i;
        while i < n && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return false;
        }
    }

    i == n
}

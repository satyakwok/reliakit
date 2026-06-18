//! Error types for parsing and serialization.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// A resource limit that was exceeded while parsing.
///
/// `#[non_exhaustive]`: new limit kinds may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsonLimitKind {
    /// `max_input_bytes` exceeded.
    InputBytes,
    /// `max_depth` exceeded.
    Depth,
    /// `max_string_bytes` exceeded for a single string value.
    StringBytes,
    /// `max_key_bytes` exceeded for a single object key.
    KeyBytes,
    /// `max_number_bytes` exceeded for a single number token.
    NumberBytes,
    /// `max_array_items` exceeded.
    ArrayItems,
    /// `max_object_members` exceeded.
    ObjectMembers,
    /// `max_total_nodes` exceeded.
    TotalNodes,
    /// `max_total_decoded_string_bytes` exceeded across the document.
    TotalDecodedStringBytes,
}

impl JsonLimitKind {
    /// A short, stable description of the limit.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputBytes => "input bytes",
            Self::Depth => "nesting depth",
            Self::StringBytes => "string bytes",
            Self::KeyBytes => "key bytes",
            Self::NumberBytes => "number bytes",
            Self::ArrayItems => "array items",
            Self::ObjectMembers => "object members",
            Self::TotalNodes => "total nodes",
            Self::TotalDecodedStringBytes => "total decoded string bytes",
        }
    }
}

/// The category of a parse or serialization failure.
///
/// This is a stable, machine-readable classification: match on it for
/// programmatic handling rather than on [`Display`](fmt::Display) text.
///
/// `#[non_exhaustive]`: new kinds may be added in a future release, so match
/// with a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonErrorKind {
    /// Input ended before a complete value was parsed.
    UnexpectedEof,
    /// A byte was found that is not valid at this position.
    UnexpectedByte,
    /// The input was not valid UTF-8 (including a leading byte-order mark).
    InvalidUtf8,
    /// An invalid string escape such as `\x`.
    InvalidEscape,
    /// A malformed `\uXXXX` escape.
    InvalidUnicodeEscape,
    /// An unpaired UTF-16 surrogate escape.
    LoneSurrogate,
    /// A raw control character (`U+0000..=U+001F`) inside a string.
    UnescapedControlCharacter,
    /// A number that does not match the strict JSON grammar.
    InvalidNumber,
    /// A duplicate object member name.
    DuplicateKey,
    /// Non-whitespace bytes after the top-level value.
    TrailingData,
    /// A configured [`JsonLimits`](crate::JsonLimits) value was exceeded.
    LimitExceeded(JsonLimitKind),
    /// The output sink failed to accept bytes (serialization).
    WriteFailure,
    /// A number could not be represented as a finite IEEE-754 `f64` during
    /// canonical serialization (e.g. a magnitude that overflows to infinity).
    NonFiniteNumber,
}

/// One segment of a [`JsonPath`].
///
/// A JSON location is always either an object member or an array element, so
/// this enum is intentionally exhaustive: you can `match` it without a wildcard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonPathSegment {
    /// An object member name.
    Key(String),
    /// An array index.
    Index(usize),
}

/// The location of an error within the JSON document, as a path of object keys
/// and array indices from the document root (`$`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JsonPath {
    segments: Vec<JsonPathSegment>,
}

impl JsonPath {
    pub(crate) fn from_segments(segments: Vec<JsonPathSegment>) -> Self {
        Self { segments }
    }

    /// Returns the path segments from the root.
    pub fn segments(&self) -> &[JsonPathSegment] {
        &self.segments
    }
}

impl fmt::Display for JsonPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("$")?;
        for segment in &self.segments {
            match segment {
                JsonPathSegment::Key(key) => write!(f, ".{key}")?,
                JsonPathSegment::Index(index) => write!(f, "[{index}]")?,
            }
        }
        Ok(())
    }
}

/// An error produced while parsing or serializing JSON.
///
/// Carries a stable [`kind`](Self::kind), the byte `offset`, 1-based `line` and
/// `column`, and the [`JsonPath`] being processed when known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    kind: JsonErrorKind,
    offset: usize,
    line: usize,
    column: usize,
    path: Option<JsonPath>,
}

impl JsonError {
    pub(crate) fn new(kind: JsonErrorKind, offset: usize, line: usize, column: usize) -> Self {
        Self {
            kind,
            offset,
            line,
            column,
            path: None,
        }
    }

    pub(crate) fn with_path(mut self, path: JsonPath) -> Self {
        self.path = Some(path);
        self
    }

    /// Builds an error that occurred during serialization, where there is no
    /// source position to report. `offset`, `line`, and `column` are `0`.
    #[cfg(feature = "canonical")]
    pub(crate) fn serialization(kind: JsonErrorKind) -> Self {
        Self {
            kind,
            offset: 0,
            line: 0,
            column: 0,
            path: None,
        }
    }

    /// The stable error category.
    pub fn kind(&self) -> &JsonErrorKind {
        &self.kind
    }

    /// The byte offset of the error in the input.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The 1-based line of the error.
    pub fn line(&self) -> usize {
        self.line
    }

    /// The 1-based column of the error.
    pub fn column(&self) -> usize {
        self.column
    }

    /// The JSON path being processed when the error occurred, if known.
    pub fn path(&self) -> Option<&JsonPath> {
        self.path.as_ref()
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match &self.kind {
            JsonErrorKind::UnexpectedEof => "unexpected end of input",
            JsonErrorKind::UnexpectedByte => "unexpected byte",
            JsonErrorKind::InvalidUtf8 => "invalid UTF-8",
            JsonErrorKind::InvalidEscape => "invalid escape sequence",
            JsonErrorKind::InvalidUnicodeEscape => "invalid unicode escape",
            JsonErrorKind::LoneSurrogate => "unpaired UTF-16 surrogate",
            JsonErrorKind::UnescapedControlCharacter => "unescaped control character in string",
            JsonErrorKind::InvalidNumber => "invalid number",
            JsonErrorKind::DuplicateKey => "duplicate object key",
            JsonErrorKind::TrailingData => "trailing data after JSON value",
            JsonErrorKind::LimitExceeded(limit) => {
                write!(
                    f,
                    "limit exceeded: {} at byte {}, line {}, column {}",
                    limit.as_str(),
                    self.offset,
                    self.line,
                    self.column
                )?;
                if let Some(path) = &self.path {
                    write!(f, ", path: {path}")?;
                }
                return Ok(());
            }
            // Serialization-side errors have no source position to report.
            JsonErrorKind::WriteFailure => return f.write_str("failed to write output"),
            JsonErrorKind::NonFiniteNumber => {
                return f.write_str("number is not representable as a finite f64");
            }
        };
        write!(
            f,
            "{message} at byte {}, line {}, column {}",
            self.offset, self.line, self.column
        )?;
        if let Some(path) = &self.path {
            write!(f, ", path: {path}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl std::error::Error for JsonError {}

/// An error from a [`JsonNumber`](crate::JsonNumber) conversion.
///
/// `#[non_exhaustive]`: new kinds may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonNumberError {
    /// The value does not fit the target integer type.
    OutOfRange,
    /// The value is not an integer (it has a fraction or exponent).
    NotAnInteger,
    /// The value is not a finite number (e.g. an exponent that overflows `f64`).
    NotFinite,
    /// The string is not a valid JSON number.
    InvalidNumber,
}

impl fmt::Display for JsonNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::OutOfRange => "number out of range for target type",
            Self::NotAnInteger => "number is not an integer",
            Self::NotFinite => "number is not finite",
            Self::InvalidNumber => "not a valid JSON number",
        })
    }
}

#[cfg(feature = "std")]
impl std::error::Error for JsonNumberError {}

/// The kind of a typed-JSON decoding error.
///
/// `#[non_exhaustive]`: new kinds may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonDecodeErrorKind {
    /// A value had a different JSON type than the target expected.
    UnexpectedType,
    /// A required object field was missing.
    MissingField,
    /// A number could not be represented by the target type.
    Number,
}

/// An error from decoding a [`JsonValue`](crate::JsonValue) into a typed value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonDecodeError {
    kind: JsonDecodeErrorKind,
    message: &'static str,
}

impl JsonDecodeError {
    /// Creates a decode error with a stable kind and an actionable message.
    pub const fn new(kind: JsonDecodeErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }

    /// Returns the stable error category.
    pub const fn kind(&self) -> JsonDecodeErrorKind {
        self.kind
    }

    /// Returns a human-readable message.
    pub const fn message(&self) -> &'static str {
        self.message
    }

    /// A value had a different JSON type than expected.
    pub const fn unexpected_type(message: &'static str) -> Self {
        Self::new(JsonDecodeErrorKind::UnexpectedType, message)
    }

    /// A required object field was missing.
    pub const fn missing_field(message: &'static str) -> Self {
        Self::new(JsonDecodeErrorKind::MissingField, message)
    }

    /// A number could not be represented by the target type.
    pub const fn number(message: &'static str) -> Self {
        Self::new(JsonDecodeErrorKind::Number, message)
    }
}

impl fmt::Display for JsonDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for JsonDecodeError {}

/// The error type of [`from_json_str`](crate::from_json_str): either the input
/// was not valid JSON, or the parsed value did not match the target type.
///
/// `#[non_exhaustive]`: new variants may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonFromStrError {
    /// The input was not valid JSON.
    Parse(JsonError),
    /// The JSON parsed but did not match the target type.
    Decode(JsonDecodeError),
}

impl fmt::Display for JsonFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "invalid JSON: {error}"),
            Self::Decode(error) => write!(f, "JSON did not match the target type: {error}"),
        }
    }
}

impl From<JsonError> for JsonFromStrError {
    fn from(error: JsonError) -> Self {
        Self::Parse(error)
    }
}

impl From<JsonDecodeError> for JsonFromStrError {
    fn from(error: JsonDecodeError) -> Self {
        Self::Decode(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for JsonFromStrError {}

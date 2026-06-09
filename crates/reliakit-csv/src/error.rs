//! Error types for reading, writing, and typed decoding of CSV.

use core::fmt;

/// A resource limit that was exceeded while reading.
///
/// `#[non_exhaustive]`: new limit kinds may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CsvLimitKind {
    /// `max_input_bytes` exceeded.
    InputBytes,
    /// `max_records` exceeded.
    Records,
    /// `max_fields_per_record` exceeded for a single record.
    FieldsPerRecord,
    /// `max_field_bytes` exceeded for a single field.
    FieldBytes,
}

impl CsvLimitKind {
    /// A short, stable description of the limit.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InputBytes => "input bytes",
            Self::Records => "records",
            Self::FieldsPerRecord => "fields per record",
            Self::FieldBytes => "field bytes",
        }
    }
}

/// The category of a CSV reading failure.
///
/// This is a stable, machine-readable classification: match on it for
/// programmatic handling rather than on [`Display`](fmt::Display) text.
///
/// `#[non_exhaustive]`: new kinds may be added in a future release, so match
/// with a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvErrorKind {
    /// A carriage return (`\r`) that was not immediately followed by a line
    /// feed (`\n`). Only `\n` and `\r\n` terminate a record.
    BareCarriageReturn,
    /// A double quote (`"`) appeared inside an unquoted field.
    QuoteInUnquotedField,
    /// A quoted field began with `"` but the closing quote was never found.
    UnterminatedQuotedField,
    /// Characters other than a delimiter or record terminator followed the
    /// closing quote of a quoted field.
    TextAfterQuotedField,
    /// A record had a different number of fields than the first record. CSV
    /// read by this crate is rectangular.
    FieldCountMismatch {
        /// The field count established by the first record.
        expected: usize,
        /// The field count of the offending record.
        found: usize,
    },
    /// A configured [`CsvLimits`](crate::CsvLimits) value was exceeded.
    LimitExceeded(CsvLimitKind),
}

/// An error produced while reading CSV.
///
/// Carries a stable [`kind`](Self::kind), the byte `offset`, 1-based `line` and
/// `column`, and the 0-based `record` and `field` index being read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvError {
    kind: CsvErrorKind,
    offset: usize,
    line: usize,
    column: usize,
    record: usize,
    field: usize,
}

impl CsvError {
    pub(crate) const fn new(
        kind: CsvErrorKind,
        offset: usize,
        line: usize,
        column: usize,
        record: usize,
        field: usize,
    ) -> Self {
        Self {
            kind,
            offset,
            line,
            column,
            record,
            field,
        }
    }

    /// The stable error category.
    pub const fn kind(&self) -> CsvErrorKind {
        self.kind
    }

    /// The byte offset of the error in the input.
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// The 1-based line of the error.
    pub const fn line(&self) -> usize {
        self.line
    }

    /// The 1-based column of the error.
    pub const fn column(&self) -> usize {
        self.column
    }

    /// The 0-based index of the record being read.
    pub const fn record(&self) -> usize {
        self.record
    }

    /// The 0-based index of the field within the record being read.
    pub const fn field(&self) -> usize {
        self.field
    }
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            CsvErrorKind::BareCarriageReturn => {
                f.write_str("carriage return not followed by line feed")?
            }
            CsvErrorKind::QuoteInUnquotedField => f.write_str("quote inside an unquoted field")?,
            CsvErrorKind::UnterminatedQuotedField => f.write_str("unterminated quoted field")?,
            CsvErrorKind::TextAfterQuotedField => {
                f.write_str("unexpected text after a quoted field")?
            }
            CsvErrorKind::FieldCountMismatch { expected, found } => write!(
                f,
                "record has {found} field(s) but {expected} were expected"
            )?,
            CsvErrorKind::LimitExceeded(limit) => write!(f, "limit exceeded: {}", limit.as_str())?,
        }
        write!(
            f,
            " at byte {}, line {}, column {} (record {}, field {})",
            self.offset, self.line, self.column, self.record, self.field
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CsvError {}

/// The kind of a typed-CSV decoding error.
///
/// `#[non_exhaustive]`: new kinds may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvDecodeErrorKind {
    /// A record had a different number of fields than the target expected.
    FieldCount,
    /// A field could not be parsed into the target type.
    Field,
    /// The header row did not match the target's [`header`](crate::CsvEncode::header).
    HeaderMismatch,
}

/// An error from decoding a CSV record into a typed value.
///
/// Carries a stable [`kind`](Self::kind), a human-readable message, and the
/// 0-based `record`/`field` indices when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CsvDecodeError {
    kind: CsvDecodeErrorKind,
    message: &'static str,
    record: Option<usize>,
    field: Option<usize>,
}

impl CsvDecodeError {
    /// Creates a decode error with a stable kind and an actionable message.
    pub const fn new(kind: CsvDecodeErrorKind, message: &'static str) -> Self {
        Self {
            kind,
            message,
            record: None,
            field: None,
        }
    }

    /// A record had the wrong number of fields for the target type. The
    /// offending location is reported through the record/field indices rather
    /// than the message, which stays a stable `&'static str`.
    pub const fn field_count() -> Self {
        Self::new(
            CsvDecodeErrorKind::FieldCount,
            "record has the wrong number of fields for the target type",
        )
    }

    /// A field could not be parsed into the target type.
    pub const fn field(message: &'static str) -> Self {
        Self::new(CsvDecodeErrorKind::Field, message)
    }

    /// The header row did not match the target's expected header.
    pub const fn header_mismatch() -> Self {
        Self::new(
            CsvDecodeErrorKind::HeaderMismatch,
            "header row does not match the target type's header",
        )
    }

    /// Returns the stable error category.
    pub const fn kind(&self) -> CsvDecodeErrorKind {
        self.kind
    }

    /// Returns a human-readable message.
    pub const fn message(&self) -> &'static str {
        self.message
    }

    /// The 0-based record index where the error occurred, if known.
    pub const fn record(&self) -> Option<usize> {
        self.record
    }

    /// The 0-based field index where the error occurred, if known.
    pub const fn field_index(&self) -> Option<usize> {
        self.field
    }

    /// Attaches the 0-based record index to this error.
    pub const fn at_record(mut self, record: usize) -> Self {
        self.record = Some(record);
        self
    }

    /// Attaches the 0-based field index to this error.
    pub const fn at_field(mut self, field: usize) -> Self {
        self.field = Some(field);
        self
    }
}

impl fmt::Display for CsvDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)?;
        match (self.record, self.field) {
            (Some(record), Some(field)) => write!(f, " (record {record}, field {field})"),
            (Some(record), None) => write!(f, " (record {record})"),
            (None, Some(field)) => write!(f, " (field {field})"),
            (None, None) => Ok(()),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CsvDecodeError {}

/// The error type of [`from_csv_str`](crate::from_csv_str): either the input
/// was not valid CSV, or a record did not match the target type.
///
/// `#[non_exhaustive]`: new variants may be added in a future release.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CsvFromStrError {
    /// The input was not valid CSV.
    Parse(CsvError),
    /// The CSV parsed but a record did not match the target type.
    Decode(CsvDecodeError),
}

impl fmt::Display for CsvFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "invalid CSV: {error}"),
            Self::Decode(error) => write!(f, "CSV did not match the target type: {error}"),
        }
    }
}

impl From<CsvError> for CsvFromStrError {
    fn from(error: CsvError) -> Self {
        Self::Parse(error)
    }
}

impl From<CsvDecodeError> for CsvFromStrError {
    fn from(error: CsvDecodeError) -> Self {
        Self::Decode(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CsvFromStrError {}

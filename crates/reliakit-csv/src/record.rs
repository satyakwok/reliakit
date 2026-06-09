//! Typed encoding and decoding of whole records.

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{CsvDecodeError, CsvFromStrError};
use crate::limits::CsvLimits;
use crate::reader::read_str_with_limits;
use crate::writer::CsvWriter;

/// A record type that can be written as a CSV row.
///
/// Implementors describe a fixed column [`header`](Self::header) and append
/// their fields, in the same order, in [`encode_fields`](Self::encode_fields).
/// The companion [`CsvDecode`] reads the row back.
pub trait CsvEncode {
    /// The column headers, in field order.
    fn header() -> Vec<&'static str>;

    /// Appends this record's fields, in column order, to `out`.
    fn encode_fields(&self, out: &mut Vec<String>);
}

/// A record type that can be read from a CSV row.
pub trait CsvDecode: Sized {
    /// Decodes a record from its fields, or returns a [`CsvDecodeError`].
    ///
    /// Implementors should reject a wrong field count with
    /// [`CsvDecodeError::field_count`].
    fn decode_fields(fields: &[&str]) -> Result<Self, CsvDecodeError>;
}

/// Encodes a slice of records to CSV text, with a leading header row.
///
/// The header comes from [`CsvEncode::header`]; each record follows in order.
/// Output is deterministic (see [`CsvWriter`]).
pub fn to_csv_string<T: CsvEncode>(records: &[T]) -> String {
    let mut writer = CsvWriter::new();
    writer.write_record(T::header());
    let mut fields: Vec<String> = Vec::new();
    for record in records {
        fields.clear();
        record.encode_fields(&mut fields);
        writer.write_record(&fields);
    }
    writer.into_string()
}

/// Encodes a slice of records to CSV text with no header row.
pub fn to_csv_string_headerless<T: CsvEncode>(records: &[T]) -> String {
    let mut writer = CsvWriter::new();
    let mut fields: Vec<String> = Vec::new();
    for record in records {
        fields.clear();
        record.encode_fields(&mut fields);
        writer.write_record(&fields);
    }
    writer.into_string()
}

/// Reads CSV text into typed records, validating the header row.
///
/// The first record must equal [`CsvEncode::header`] exactly (same fields, same
/// order); the rest are decoded with [`CsvDecode::decode_fields`]. Uses
/// conservative [`CsvLimits`].
///
/// An empty input yields no records. An input whose only record is the header
/// also yields no records.
pub fn from_csv_str<T: CsvEncode + CsvDecode>(input: &str) -> Result<Vec<T>, CsvFromStrError> {
    from_csv_str_with_limits(input, &CsvLimits::conservative())
}

/// Reads CSV text into typed records, validating the header, with explicit limits.
pub fn from_csv_str_with_limits<T: CsvEncode + CsvDecode>(
    input: &str,
    limits: &CsvLimits,
) -> Result<Vec<T>, CsvFromStrError> {
    let records = read_str_with_limits(input, limits)?;
    let mut rows = records.into_iter();

    match rows.next() {
        None => Ok(Vec::new()),
        Some(header) => {
            let expected = T::header();
            if header.len() != expected.len()
                || header.iter().zip(expected.iter()).any(|(a, b)| a != b)
            {
                return Err(CsvDecodeError::header_mismatch().at_record(0).into());
            }
            decode_rows::<T>(rows, 1)
        }
    }
}

/// Reads headerless CSV text into typed records, decoding every record.
pub fn from_csv_str_headerless<T: CsvDecode>(input: &str) -> Result<Vec<T>, CsvFromStrError> {
    from_csv_str_headerless_with_limits(input, &CsvLimits::conservative())
}

/// Reads headerless CSV text into typed records, with explicit limits.
pub fn from_csv_str_headerless_with_limits<T: CsvDecode>(
    input: &str,
    limits: &CsvLimits,
) -> Result<Vec<T>, CsvFromStrError> {
    let records = read_str_with_limits(input, limits)?;
    decode_rows::<T>(records.into_iter(), 0)
}

/// Decodes an iterator of string records into typed values, numbering record
/// errors from `base_record`.
fn decode_rows<T: CsvDecode>(
    rows: impl Iterator<Item = Vec<String>>,
    base_record: usize,
) -> Result<Vec<T>, CsvFromStrError> {
    let mut out = Vec::new();
    for (offset, record) in rows.enumerate() {
        let refs: Vec<&str> = record.iter().map(String::as_str).collect();
        let value =
            T::decode_fields(&refs).map_err(|error| error.at_record(base_record + offset))?;
        out.push(value);
    }
    Ok(out)
}

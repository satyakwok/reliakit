//! Strict, bounded CSV reading.

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::{CsvError, CsvErrorKind, CsvLimitKind};
use crate::limits::CsvLimits;

/// Parses CSV text into records, using conservative [`CsvLimits`].
///
/// Each record is a `Vec<String>` of fields. The result is rectangular: every
/// record has the same number of fields as the first, or the read fails. See
/// the [crate] documentation for the exact accepted grammar.
///
/// ```
/// use reliakit_csv::read_str;
///
/// assert_eq!(read_str("a,b\n1,2\n").unwrap(), [["a", "b"], ["1", "2"]]);
/// assert_eq!(read_str("").unwrap(), Vec::<Vec<String>>::new());
/// ```
pub fn read_str(input: &str) -> Result<Vec<Vec<String>>, CsvError> {
    read_str_with_limits(input, &CsvLimits::conservative())
}

/// Parses CSV text into records with explicit [`CsvLimits`].
pub fn read_str_with_limits(input: &str, limits: &CsvLimits) -> Result<Vec<Vec<String>>, CsvError> {
    if input.len() > limits.max_input_bytes() {
        return Err(CsvError::new(
            CsvErrorKind::LimitExceeded(CsvLimitKind::InputBytes),
            0,
            1,
            1,
            0,
            0,
        ));
    }

    let chars: Vec<(usize, char)> = input.char_indices().collect();
    let mut parser = Parser {
        input,
        chars,
        pos: 0,
        limits,
    };

    let mut records: Vec<Vec<String>> = Vec::new();
    let mut expected_width: Option<usize> = None;

    while parser.pos < parser.chars.len() {
        let record_index = records.len();
        if record_index >= limits.max_records() {
            return parser.err(
                CsvErrorKind::LimitExceeded(CsvLimitKind::Records),
                record_index,
                0,
            );
        }

        let record = parser.parse_record(record_index)?;

        match expected_width {
            None => expected_width = Some(record.len()),
            Some(width) if record.len() != width => {
                // Report at the start of the offending record's terminator
                // position (current parser position), which is just past it.
                return parser.err(
                    CsvErrorKind::FieldCountMismatch {
                        expected: width,
                        found: record.len(),
                    },
                    record_index,
                    record.len().saturating_sub(1),
                );
            }
            Some(_) => {}
        }

        records.push(record);
    }

    Ok(records)
}

/// How a single field ended.
enum FieldEnd {
    /// A `,` delimiter; another field follows.
    Delimiter,
    /// A `\n` or `\r\n` record terminator, or end of input.
    Record,
}

struct Parser<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    limits: &'a CsvLimits,
}

impl Parser<'_> {
    /// Builds an error at the current parser position for the given location.
    fn err<T>(&self, kind: CsvErrorKind, record: usize, field: usize) -> Result<T, CsvError> {
        let offset = self.offset_at(self.pos);
        let (line, column) = self.line_col(offset);
        Err(CsvError::new(kind, offset, line, column, record, field))
    }

    /// The byte offset for a character index (or end of input if past the end).
    fn offset_at(&self, index: usize) -> usize {
        self.chars
            .get(index)
            .map(|(offset, _)| *offset)
            .unwrap_or(self.input.len())
    }

    /// 1-based line and column for a byte offset.
    fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut column = 1;
        for (byte_index, ch) in self.input.char_indices() {
            if byte_index >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        (line, column)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).map(|(_, c)| *c)
    }

    fn peek_at(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.pos + ahead).map(|(_, c)| *c)
    }

    /// Parses one record (one or more fields). Assumes `self.pos < len`.
    fn parse_record(&mut self, record_index: usize) -> Result<Vec<String>, CsvError> {
        let mut record: Vec<String> = Vec::new();
        loop {
            let field_index = record.len();
            if field_index >= self.limits.max_fields_per_record() {
                return self.err(
                    CsvErrorKind::LimitExceeded(CsvLimitKind::FieldsPerRecord),
                    record_index,
                    field_index,
                );
            }

            let (field, end) = self.parse_field(record_index, field_index)?;
            record.push(field);
            match end {
                FieldEnd::Delimiter => continue,
                FieldEnd::Record => break,
            }
        }
        Ok(record)
    }

    /// Parses one field and reports how it ended.
    fn parse_field(
        &mut self,
        record_index: usize,
        field_index: usize,
    ) -> Result<(String, FieldEnd), CsvError> {
        if self.peek() == Some('"') {
            self.parse_quoted_field(record_index, field_index)
        } else {
            self.parse_unquoted_field(record_index, field_index)
        }
    }

    fn parse_quoted_field(
        &mut self,
        record_index: usize,
        field_index: usize,
    ) -> Result<(String, FieldEnd), CsvError> {
        self.pos += 1; // consume the opening quote
        let mut buf = String::new();
        loop {
            let Some(c) = self.peek() else {
                return self.err(
                    CsvErrorKind::UnterminatedQuotedField,
                    record_index,
                    field_index,
                );
            };
            if c == '"' {
                if self.peek_at(1) == Some('"') {
                    // Escaped quote: `""` is one literal `"`.
                    self.push_field_byte(&mut buf, '"', record_index, field_index)?;
                    self.pos += 2;
                } else {
                    // Closing quote: only a delimiter, terminator, or EOF may follow.
                    self.pos += 1;
                    return self.finish_after_quote(record_index, field_index, buf);
                }
            } else {
                self.push_field_byte(&mut buf, c, record_index, field_index)?;
                self.pos += 1;
            }
        }
    }

    fn finish_after_quote(
        &mut self,
        record_index: usize,
        field_index: usize,
        buf: String,
    ) -> Result<(String, FieldEnd), CsvError> {
        match self.peek() {
            None => Ok((buf, FieldEnd::Record)),
            Some(',') => {
                self.pos += 1;
                Ok((buf, FieldEnd::Delimiter))
            }
            Some('\n') => {
                self.pos += 1;
                Ok((buf, FieldEnd::Record))
            }
            Some('\r') => {
                if self.peek_at(1) == Some('\n') {
                    self.pos += 2;
                    Ok((buf, FieldEnd::Record))
                } else {
                    self.err(CsvErrorKind::BareCarriageReturn, record_index, field_index)
                }
            }
            Some(_) => self.err(
                CsvErrorKind::TextAfterQuotedField,
                record_index,
                field_index,
            ),
        }
    }

    fn parse_unquoted_field(
        &mut self,
        record_index: usize,
        field_index: usize,
    ) -> Result<(String, FieldEnd), CsvError> {
        let mut buf = String::new();
        loop {
            match self.peek() {
                None => return Ok((buf, FieldEnd::Record)),
                Some(',') => {
                    self.pos += 1;
                    return Ok((buf, FieldEnd::Delimiter));
                }
                Some('\n') => {
                    self.pos += 1;
                    return Ok((buf, FieldEnd::Record));
                }
                Some('\r') => {
                    if self.peek_at(1) == Some('\n') {
                        self.pos += 2;
                        return Ok((buf, FieldEnd::Record));
                    }
                    return self.err(CsvErrorKind::BareCarriageReturn, record_index, field_index);
                }
                Some('"') => {
                    return self.err(
                        CsvErrorKind::QuoteInUnquotedField,
                        record_index,
                        field_index,
                    )
                }
                Some(c) => {
                    self.push_field_byte(&mut buf, c, record_index, field_index)?;
                    self.pos += 1;
                }
            }
        }
    }

    /// Appends a character to a field buffer, enforcing the field byte limit.
    fn push_field_byte(
        &self,
        buf: &mut String,
        c: char,
        record_index: usize,
        field_index: usize,
    ) -> Result<(), CsvError> {
        if buf.len() + c.len_utf8() > self.limits.max_field_bytes() {
            return self.err(
                CsvErrorKind::LimitExceeded(CsvLimitKind::FieldBytes),
                record_index,
                field_index,
            );
        }
        buf.push(c);
        Ok(())
    }
}

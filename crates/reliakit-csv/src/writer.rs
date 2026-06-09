//! Deterministic CSV writing.

use alloc::string::String;

/// Builds CSV text one record at a time.
///
/// Output is deterministic: a field is quoted only when it contains a delimiter
/// (`,`), a quote (`"`), or a line break (`\r`/`\n`); an embedded quote is
/// doubled; and every record is terminated with `\r\n` (per RFC 4180).
/// In-memory writing cannot fail.
///
/// ```
/// use reliakit_csv::CsvWriter;
///
/// let mut writer = CsvWriter::new();
/// writer.write_record(["id", "note"]);
/// writer.write_record(["1", "has \"quotes\""]);
/// assert_eq!(writer.into_string(), "id,note\r\n1,\"has \"\"quotes\"\"\"\r\n");
/// ```
#[derive(Debug, Clone, Default)]
pub struct CsvWriter {
    out: String,
}

impl CsvWriter {
    /// Creates an empty writer.
    pub const fn new() -> Self {
        Self { out: String::new() }
    }

    /// Writes one record from an iterator of field values, terminated by `\r\n`.
    pub fn write_record<I, S>(&mut self, fields: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut first = true;
        for field in fields {
            if !first {
                self.out.push(',');
            }
            first = false;
            write_field(&mut self.out, field.as_ref());
        }
        self.out.push_str("\r\n");
    }

    /// Returns the number of bytes written so far.
    pub fn len(&self) -> usize {
        self.out.len()
    }

    /// Returns `true` if no records have been written.
    pub fn is_empty(&self) -> bool {
        self.out.is_empty()
    }

    /// Returns a borrowed view of the CSV written so far.
    pub fn as_str(&self) -> &str {
        &self.out
    }

    /// Consumes the writer and returns the CSV text.
    pub fn into_string(self) -> String {
        self.out
    }
}

/// Appends a single field to `out`, quoting and escaping only as required.
fn write_field(out: &mut String, field: &str) {
    if needs_quoting(field) {
        out.push('"');
        for c in field.chars() {
            if c == '"' {
                out.push('"');
            }
            out.push(c);
        }
        out.push('"');
    } else {
        out.push_str(field);
    }
}

/// A field needs quoting if it contains a delimiter, a quote, or a line break.
fn needs_quoting(field: &str) -> bool {
    field
        .bytes()
        .any(|b| matches!(b, b',' | b'"' | b'\r' | b'\n'))
}

//! Strict, bounded, and deterministic CSV for reliability-sensitive Rust.
//!
//! `reliakit-csv` reads and writes a strict subset of [RFC 4180]. It is built
//! for systems that process **untrusted** CSV or need **predictable** output:
//! it rejects malformed quoting, enforces a rectangular shape (every record has
//! the same number of fields), applies explicit [resource limits](CsvLimits),
//! reports errors with a location, and serializes deterministically. It has no
//! external dependencies, forbids unsafe code, and supports `no_std` (with
//! `alloc`).
//!
//! It deliberately does **not** include type inference, configurable dialects,
//! a streaming reader/writer over `std::io`, schema validation, or lenient
//! recovery from malformed input.
//!
//! # Records
//!
//! At the lowest level a CSV document is a sequence of records, and each record
//! is a sequence of UTF-8 string fields. [`read_str`] parses text into
//! `Vec<Vec<String>>` and [`CsvWriter`] turns records back into text.
//!
//! ```
//! use reliakit_csv::{read_str, CsvWriter};
//!
//! let records = read_str("name,city\nAda,London\n").unwrap();
//! assert_eq!(records, [["name", "city"], ["Ada", "London"]]);
//!
//! // Serialization is deterministic: a field is quoted only when it must be,
//! // and every record ends with CRLF.
//! let mut writer = CsvWriter::new();
//! writer.write_record(["a", "b,c"]);
//! assert_eq!(writer.into_string(), "a,\"b,c\"\r\n");
//! ```
//!
//! # Strictness
//!
//! The reader rejects input that lenient parsers would accept, so a successful
//! parse is a strong guarantee:
//!
//! ```
//! use reliakit_csv::read_str;
//!
//! // A quote inside an unquoted field is rejected, not absorbed.
//! assert!(read_str("ab\"c\n").is_err());
//! // Records must be rectangular: a short row is an error, not a `None` cell.
//! assert!(read_str("a,b\nc\n").is_err());
//! // A quoted field that never closes is rejected.
//! assert!(read_str("\"oops\n").is_err());
//! ```
//!
//! # Typed encoding
//!
//! [`CsvEncode`] maps your record type to a row of fields (with a header), and
//! [`CsvDecode`] reads a row back, strictly. [`to_csv_string`] writes a header
//! row followed by one row per value; [`from_csv_str`] validates the header and
//! decodes the rest.
//!
//! ```
//! use reliakit_csv::{
//!     from_csv_str, to_csv_string, CsvDecode, CsvDecodeError, CsvEncode, CsvField,
//! };
//! use std::vec::Vec;
//!
//! #[derive(Debug, PartialEq)]
//! struct Row {
//!     id: u32,
//!     name: String,
//! }
//!
//! impl CsvEncode for Row {
//!     fn header() -> Vec<&'static str> {
//!         vec!["id", "name"]
//!     }
//!     fn encode_fields(&self, out: &mut Vec<String>) {
//!         out.push(self.id.encode_field());
//!         out.push(self.name.encode_field());
//!     }
//! }
//!
//! impl CsvDecode for Row {
//!     fn decode_fields(fields: &[&str]) -> Result<Self, CsvDecodeError> {
//!         if fields.len() != 2 {
//!             return Err(CsvDecodeError::field_count());
//!         }
//!         Ok(Row {
//!             id: u32::decode_field(fields[0]).map_err(|e| e.at_field(0))?,
//!             name: String::decode_field(fields[1]).map_err(|e| e.at_field(1))?,
//!         })
//!     }
//! }
//!
//! let rows = vec![Row { id: 1, name: "Ada".into() }];
//! let text = to_csv_string(&rows);
//! assert_eq!(text, "id,name\r\n1,Ada\r\n");
//! assert_eq!(from_csv_str::<Row>(&text).unwrap(), rows);
//! ```
//!
//! The per-field `encode_field`/`decode_field` calls above come from
//! [`CsvField`], which is implemented for the integer types, `bool`, `String`,
//! and `Option<T>`.
//!
//! # Limits
//!
//! [`read_str`] applies conservative [`CsvLimits`] by default. Use
//! [`read_str_with_limits`] to choose a profile or tune individual limits:
//!
//! ```
//! use reliakit_csv::{read_str_with_limits, CsvLimits};
//!
//! let limits = CsvLimits::conservative().with_max_fields_per_record(2);
//! assert!(read_str_with_limits("a,b,c\n", &limits).is_err());
//! ```
//!
//! # Feature flags
//!
//! - `std` (default) enables `std::error::Error` for the error types. The crate
//!   is otherwise `no_std` and always uses `alloc`.
//!
//! [RFC 4180]: https://www.rfc-editor.org/rfc/rfc4180

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

mod error;
mod field;
mod limits;
mod reader;
mod record;
mod writer;

pub use error::{
    CsvDecodeError, CsvDecodeErrorKind, CsvError, CsvErrorKind, CsvFromStrError, CsvLimitKind,
};
pub use field::CsvField;
pub use limits::CsvLimits;
pub use reader::{read_str, read_str_with_limits};
pub use record::{
    from_csv_str, from_csv_str_headerless, from_csv_str_headerless_with_limits,
    from_csv_str_with_limits, to_csv_string, to_csv_string_headerless, CsvDecode, CsvEncode,
};
pub use writer::CsvWriter;

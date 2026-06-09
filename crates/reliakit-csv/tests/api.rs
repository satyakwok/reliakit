//! Coverage of the public error, limits, and writer surface: accessors,
//! `Display`, and builder methods that the format tests do not exercise.

use reliakit_csv::{
    from_csv_str, read_str, read_str_with_limits, CsvDecode, CsvDecodeError, CsvDecodeErrorKind,
    CsvEncode, CsvError, CsvErrorKind, CsvField, CsvFromStrError, CsvLimitKind, CsvLimits,
    CsvWriter,
};

#[derive(Debug)]
struct Row {
    a: u8,
}

impl CsvEncode for Row {
    fn header() -> Vec<&'static str> {
        vec!["a"]
    }
    fn encode_fields(&self, out: &mut Vec<String>) {
        out.push(self.a.encode_field());
    }
}

impl CsvDecode for Row {
    fn decode_fields(fields: &[&str]) -> Result<Self, CsvDecodeError> {
        if fields.len() != 1 {
            return Err(CsvDecodeError::field_count());
        }
        Ok(Row {
            a: u8::decode_field(fields[0]).map_err(|e| e.at_field(0))?,
        })
    }
}

// ---- CsvError: kinds, accessors, Display ------------------------------------

fn read_err(input: &str) -> CsvError {
    read_str(input).unwrap_err()
}

#[test]
fn csv_error_accessors_and_display_cover_every_kind() {
    // A bare CR at line 2 gives a concrete location to check accessors against.
    let err = read_err("ok\na\rb\n");
    assert_eq!(err.kind(), CsvErrorKind::BareCarriageReturn);
    assert_eq!(err.line(), 2);
    assert!(err.column() >= 1);
    assert_eq!(err.record(), 1);
    assert_eq!(err.field(), 0);
    assert!(err.offset() > 0);
    assert!(format!("{err}").contains("carriage return"));

    let cases = [
        (
            read_err("ab\"c\n"),
            CsvErrorKind::QuoteInUnquotedField,
            "quote",
        ),
        (
            read_err("\"oops\n"),
            CsvErrorKind::UnterminatedQuotedField,
            "unterminated",
        ),
        (
            read_err("\"ab\"c\n"),
            CsvErrorKind::TextAfterQuotedField,
            "text",
        ),
    ];
    for (err, kind, needle) in cases {
        assert_eq!(err.kind(), kind);
        assert!(format!("{err}").contains(needle), "{err}");
    }

    // FieldCountMismatch carries the counts and renders them.
    let err = read_err("a,b\nc\n");
    assert_eq!(
        err.kind(),
        CsvErrorKind::FieldCountMismatch {
            expected: 2,
            found: 1
        }
    );
    let text = format!("{err}");
    assert!(text.contains('2') && text.contains('1'), "{text}");

    // LimitExceeded renders its limit description.
    let limits = CsvLimits::conservative().with_max_records(1);
    let err = read_str_with_limits("a\nb\n", &limits).unwrap_err();
    assert_eq!(
        err.kind(),
        CsvErrorKind::LimitExceeded(CsvLimitKind::Records)
    );
    assert!(format!("{err}").contains("limit exceeded"));
}

#[test]
fn csv_limit_kind_descriptions() {
    assert_eq!(CsvLimitKind::InputBytes.as_str(), "input bytes");
    assert_eq!(CsvLimitKind::Records.as_str(), "records");
    assert_eq!(CsvLimitKind::FieldsPerRecord.as_str(), "fields per record");
    assert_eq!(CsvLimitKind::FieldBytes.as_str(), "field bytes");
}

// ---- CsvLimits: builders, accessors, defaults -------------------------------

#[test]
fn csv_limits_builders_and_accessors() {
    let limits = CsvLimits::conservative()
        .with_max_input_bytes(10)
        .with_max_records(5)
        .with_max_fields_per_record(3)
        .with_max_field_bytes(4);
    assert_eq!(limits.max_input_bytes(), 10);
    assert_eq!(limits.max_records(), 5);
    assert_eq!(limits.max_fields_per_record(), 3);
    assert_eq!(limits.max_field_bytes(), 4);

    // The default profile is the conservative one.
    assert_eq!(CsvLimits::default(), CsvLimits::conservative());

    // Permissive is strictly looser on every axis.
    let (c, p) = (CsvLimits::conservative(), CsvLimits::permissive());
    assert!(p.max_input_bytes() > c.max_input_bytes());
    assert!(p.max_records() > c.max_records());
    assert!(p.max_fields_per_record() > c.max_fields_per_record());
    assert!(p.max_field_bytes() > c.max_field_bytes());

    // The per-field limit triggers with the expected kind.
    let limits = CsvLimits::conservative().with_max_field_bytes(2);
    let err = read_str_with_limits("abc\n", &limits).unwrap_err();
    assert_eq!(
        err.kind(),
        CsvErrorKind::LimitExceeded(CsvLimitKind::FieldBytes)
    );
}

// ---- CsvDecodeError: constructors, accessors, Display -----------------------

#[test]
fn csv_decode_error_surface() {
    let base = CsvDecodeError::field("bad value");
    assert_eq!(base.kind(), CsvDecodeErrorKind::Field);
    assert_eq!(base.message(), "bad value");
    assert_eq!(base.record(), None);
    assert_eq!(base.field_index(), None);
    assert_eq!(format!("{base}"), "bad value");

    let with_field = base.at_field(2);
    assert_eq!(with_field.field_index(), Some(2));
    assert!(format!("{with_field}").contains("field 2"));

    let with_both = CsvDecodeError::field_count().at_record(1).at_field(0);
    assert_eq!(with_both.kind(), CsvDecodeErrorKind::FieldCount);
    assert_eq!(with_both.record(), Some(1));
    assert_eq!(with_both.field_index(), Some(0));
    assert!(format!("{with_both}").contains("record 1"));

    let only_record = CsvDecodeError::field("x").at_record(3);
    assert!(format!("{only_record}").contains("record 3"));

    let header = CsvDecodeError::header_mismatch();
    assert_eq!(header.kind(), CsvDecodeErrorKind::HeaderMismatch);

    let manual = CsvDecodeError::new(CsvDecodeErrorKind::Field, "manual");
    assert_eq!(manual.message(), "manual");
}

#[test]
fn csv_from_str_error_display_and_from() {
    // Decode arm: a header mismatch surfaces through from_csv_str.
    let err = from_csv_str::<Row>("b\n").unwrap_err();
    assert!(matches!(err, CsvFromStrError::Decode(_)));
    assert!(format!("{err}").contains("did not match"));

    // Parse arm: malformed CSV surfaces as a parse error.
    let err = from_csv_str::<Row>("\"unterminated\n").unwrap_err();
    assert!(matches!(err, CsvFromStrError::Parse(_)));
    assert!(format!("{err}").contains("invalid CSV"));

    // The `From` conversions build each arm directly.
    let parse: CsvFromStrError = read_str("\"x\n").unwrap_err().into();
    assert!(matches!(parse, CsvFromStrError::Parse(_)));
    let decode: CsvFromStrError = CsvDecodeError::field_count().into();
    assert!(matches!(decode, CsvFromStrError::Decode(_)));
}

// ---- CsvWriter: len / is_empty / as_str / Default ---------------------------

#[test]
fn csv_writer_inspection_methods() {
    let mut writer = CsvWriter::default();
    assert!(writer.is_empty());
    assert_eq!(writer.len(), 0);

    writer.write_record(["x", "y"]);
    assert!(!writer.is_empty());
    assert_eq!(writer.as_str(), "x,y\r\n");
    assert_eq!(writer.len(), "x,y\r\n".len());

    // Clone + Debug are derived and usable.
    let cloned = writer.clone();
    assert_eq!(cloned.into_string(), writer.into_string());
}

//! Wire-format tests. These pin the exact text `reliakit-csv` reads and writes;
//! changing any assertion here is a breaking format change.

use reliakit_csv::{
    from_csv_str, read_str, read_str_with_limits, to_csv_string, to_csv_string_headerless,
    CsvDecode, CsvDecodeError, CsvDecodeErrorKind, CsvEncode, CsvErrorKind, CsvField, CsvLimitKind,
    CsvLimits, CsvWriter,
};

#[derive(Debug, PartialEq, Clone)]
struct Row {
    id: u32,
    label: String,
    active: Option<bool>,
}

impl CsvEncode for Row {
    fn header() -> Vec<&'static str> {
        vec!["id", "label", "active"]
    }
    fn encode_fields(&self, out: &mut Vec<String>) {
        out.push(self.id.encode_field());
        out.push(self.label.encode_field());
        out.push(self.active.encode_field());
    }
}

impl CsvDecode for Row {
    fn decode_fields(fields: &[&str]) -> Result<Self, CsvDecodeError> {
        if fields.len() != 3 {
            return Err(CsvDecodeError::field_count());
        }
        Ok(Row {
            id: u32::decode_field(fields[0]).map_err(|e| e.at_field(0))?,
            label: String::decode_field(fields[1]).map_err(|e| e.at_field(1))?,
            active: Option::<bool>::decode_field(fields[2]).map_err(|e| e.at_field(2))?,
        })
    }
}

// ---- Writer: exact bytes ----------------------------------------------------

#[test]
fn writer_quotes_only_when_required() {
    let mut w = CsvWriter::new();
    w.write_record(["plain", "", "a b"]);
    w.write_record(["a,b", "c\"d", "e\nf"]);
    w.write_record(["g\rh"]);
    assert_eq!(
        w.into_string(),
        "plain,,a b\r\n\"a,b\",\"c\"\"d\",\"e\nf\"\r\n\"g\rh\"\r\n"
    );
}

#[test]
fn empty_writer_is_empty() {
    assert_eq!(CsvWriter::new().into_string(), "");
}

#[test]
fn single_empty_field_round_trips_as_crlf() {
    let mut w = CsvWriter::new();
    w.write_record([""]);
    assert_eq!(w.into_string(), "\r\n");
    assert_eq!(read_str("\r\n").unwrap(), [[""]]);
}

// ---- Reader: acceptance -----------------------------------------------------

#[test]
fn reads_lf_and_crlf_and_trailing_terminator() {
    assert_eq!(read_str("a,b\n1,2").unwrap(), [["a", "b"], ["1", "2"]]);
    assert_eq!(
        read_str("a,b\r\n1,2\r\n").unwrap(),
        [["a", "b"], ["1", "2"]]
    );
    assert_eq!(read_str("").unwrap(), Vec::<Vec<String>>::new());
}

#[test]
fn reads_quoted_fields_with_embedded_specials() {
    assert_eq!(
        read_str("\"a,b\",\"c\"\"d\",\"e\nf\"\n").unwrap(),
        [["a,b", "c\"d", "e\nf"]]
    );
}

#[test]
fn blank_line_is_a_single_empty_field() {
    // One column throughout, so the blank middle line is a valid empty record.
    assert_eq!(read_str("a\n\nb\n").unwrap(), [["a"], [""], ["b"]]);
}

#[test]
fn quoted_field_boundaries() {
    // A quoted field can be the final field with no trailing terminator.
    assert_eq!(read_str("\"x\"").unwrap(), [["x"]]);
    assert_eq!(read_str("\"a\"\n\"b\"").unwrap(), [["a"], ["b"]]);

    // A quoted field may be terminated by CRLF, not only LF.
    assert_eq!(read_str("\"x\"\r\n").unwrap(), [["x"]]);
    assert_eq!(read_str("\"a,b\"\r\n\"c\"\r\n").unwrap(), [["a,b"], ["c"]]);

    // A bare CR after a closing quote is rejected, like a bare CR anywhere else.
    let err = read_str("\"x\"\ry").unwrap_err();
    assert_eq!(err.kind(), CsvErrorKind::BareCarriageReturn);
}

// ---- Reader: strict rejection ----------------------------------------------

#[test]
fn rejects_quote_in_unquoted_field() {
    let err = read_str("ab\"c\n").unwrap_err();
    assert_eq!(err.kind(), CsvErrorKind::QuoteInUnquotedField);
}

#[test]
fn rejects_text_after_quoted_field() {
    let err = read_str("\"ab\"c\n").unwrap_err();
    assert_eq!(err.kind(), CsvErrorKind::TextAfterQuotedField);
}

#[test]
fn rejects_unterminated_quoted_field() {
    let err = read_str("\"oops\n").unwrap_err();
    assert_eq!(err.kind(), CsvErrorKind::UnterminatedQuotedField);
}

#[test]
fn rejects_bare_carriage_return() {
    let err = read_str("a\rb\n").unwrap_err();
    assert_eq!(err.kind(), CsvErrorKind::BareCarriageReturn);
}

#[test]
fn rejects_ragged_records() {
    let err = read_str("a,b\nc\n").unwrap_err();
    assert_eq!(
        err.kind(),
        CsvErrorKind::FieldCountMismatch {
            expected: 2,
            found: 1
        }
    );
    assert_eq!(err.record(), 1);
}

#[test]
fn enforces_limits() {
    let limits = CsvLimits::conservative().with_max_fields_per_record(2);
    let err = read_str_with_limits("a,b,c\n", &limits).unwrap_err();
    assert_eq!(
        err.kind(),
        CsvErrorKind::LimitExceeded(CsvLimitKind::FieldsPerRecord)
    );

    let limits = CsvLimits::conservative().with_max_input_bytes(3);
    let err = read_str_with_limits("a,b,c", &limits).unwrap_err();
    assert_eq!(
        err.kind(),
        CsvErrorKind::LimitExceeded(CsvLimitKind::InputBytes)
    );
}

// ---- Typed layer ------------------------------------------------------------

#[test]
fn typed_round_trip_with_header() {
    let rows = vec![
        Row {
            id: 1,
            label: "ok".into(),
            active: Some(true),
        },
        Row {
            id: 2,
            label: "a,b".into(),
            active: None,
        },
    ];
    let text = to_csv_string(&rows);
    assert_eq!(text, "id,label,active\r\n1,ok,true\r\n2,\"a,b\",\r\n");
    assert_eq!(from_csv_str::<Row>(&text).unwrap(), rows);
}

#[test]
fn header_only_decodes_to_no_rows() {
    assert_eq!(
        from_csv_str::<Row>("id,label,active\r\n").unwrap(),
        Vec::new()
    );
    assert_eq!(from_csv_str::<Row>("").unwrap(), Vec::new());
}

#[test]
fn wrong_header_is_rejected() {
    let err = from_csv_str::<Row>("id,name,active\r\n").unwrap_err();
    match err {
        reliakit_csv::CsvFromStrError::Decode(e) => {
            assert_eq!(e.kind(), CsvDecodeErrorKind::HeaderMismatch);
        }
        other => panic!("expected decode error, got {other:?}"),
    }
}

#[test]
fn bad_field_reports_record_and_field() {
    let text = "id,label,active\r\nx,ok,true\r\n";
    let err = from_csv_str::<Row>(text).unwrap_err();
    match err {
        reliakit_csv::CsvFromStrError::Decode(e) => {
            assert_eq!(e.kind(), CsvDecodeErrorKind::Field);
            assert_eq!(e.record(), Some(1));
            assert_eq!(e.field_index(), Some(0));
        }
        other => panic!("expected decode error, got {other:?}"),
    }
}

#[test]
fn headerless_round_trip() {
    let rows = vec![Row {
        id: 7,
        label: "x".into(),
        active: Some(false),
    }];
    let text = to_csv_string_headerless(&rows);
    assert_eq!(text, "7,x,false\r\n");
    assert_eq!(
        reliakit_csv::from_csv_str_headerless::<Row>(&text).unwrap(),
        rows
    );
}

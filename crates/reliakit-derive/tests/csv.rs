//! Tests for the `reliakit-csv` derives.

use reliakit_csv::{CsvDecode, CsvDecodeErrorKind, CsvFromStrError, from_csv_str, to_csv_string};
use reliakit_derive::{CsvDecode, CsvEncode};

#[derive(Debug, PartialEq, CsvEncode, CsvDecode)]
struct Row {
    id: u32,
    name: String,
    active: Option<bool>,
}

#[test]
fn named_struct_exact_text_and_roundtrip() {
    let rows = vec![
        Row {
            id: 1,
            name: "ada".into(),
            active: Some(true),
        },
        Row {
            id: 2,
            name: "a,b".into(),
            active: None,
        },
    ];
    // Header from the field names, then one row each: a field with a comma is
    // quoted, and `None` is an empty field.
    assert_eq!(
        to_csv_string(&rows),
        "id,name,active\r\n1,ada,true\r\n2,\"a,b\",\r\n"
    );
    assert_eq!(from_csv_str::<Row>(&to_csv_string(&rows)).unwrap(), rows);
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, CsvEncode, CsvDecode)]
struct Keyword {
    r#type: u8,
    r#struct: bool,
}

#[test]
fn raw_identifier_fields_use_plain_columns() {
    let rows = vec![Keyword {
        r#type: 5,
        r#struct: true,
    }];
    // The `r#` prefix is dropped for the column name.
    assert_eq!(to_csv_string(&rows), "type,struct\r\n5,true\r\n");
    assert_eq!(
        from_csv_str::<Keyword>(&to_csv_string(&rows)).unwrap(),
        rows
    );
}

#[test]
fn decode_fields_rejects_wrong_count() {
    // The derived `decode_fields` rejects a row with the wrong number of fields.
    let err = Row::decode_fields(&["1", "ada"]).unwrap_err();
    assert_eq!(err.kind(), CsvDecodeErrorKind::FieldCount);
}

#[test]
fn bad_field_reports_its_index() {
    // `id` is a u32; a non-number fails to decode, located at field 0, record 1.
    let err = from_csv_str::<Row>("id,name,active\r\nx,ada,true\r\n").unwrap_err();
    match err {
        CsvFromStrError::Decode(e) => {
            assert_eq!(e.kind(), CsvDecodeErrorKind::Field);
            assert_eq!(e.record(), Some(1));
            assert_eq!(e.field_index(), Some(0));
        }
        other => panic!("expected a decode error, got {other:?}"),
    }
}

#[derive(Debug, PartialEq, CsvEncode, CsvDecode)]
struct Attributed {
    #[reliakit(rename = "ID")]
    id: u32,
    #[reliakit(skip)]
    note: String,
    name: String,
}

#[test]
fn csv_rename_changes_the_header_and_skip_drops_the_column() {
    let rows = vec![Attributed {
        id: 1,
        note: "ignored".into(),
        name: "a".into(),
    }];
    // Header uses `ID` (renamed) and omits the skipped `note` column entirely.
    assert_eq!(to_csv_string(&rows), "ID,name\r\n1,a\r\n");
}

#[test]
fn csv_skip_defaults_on_decode_and_round_trips() {
    let parsed: Vec<Attributed> = from_csv_str("ID,name\r\n1,a\r\n").unwrap();
    // `note` is not a column; it decodes to `String::default()`.
    assert_eq!(
        parsed,
        vec![Attributed {
            id: 1,
            note: String::new(),
            name: "a".into(),
        }]
    );

    // Round-trip: the skipped value is lost, the renamed column survives.
    let rows = vec![Attributed {
        id: 7,
        note: "lost".into(),
        name: "b".into(),
    }];
    let restored: Vec<Attributed> = from_csv_str(&to_csv_string(&rows)).unwrap();
    assert_eq!(
        restored,
        vec![Attributed {
            id: 7,
            note: String::new(),
            name: "b".into(),
        }]
    );
}

#[test]
fn csv_skip_changes_the_expected_column_count() {
    // Two columns now (ID, name); a three-column row no longer matches the layout.
    assert!(from_csv_str::<Attributed>("ID,name\r\n1,a,extra\r\n").is_err());
}

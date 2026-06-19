//! Gold test for the `#[reliakit(crate = "...")]` derive escape hatch.
//!
//! A downstream crate that depends ONLY on the umbrella `reliakit` (not the individual
//! `reliakit-csv`/`reliakit-json`/`reliakit-codec` crates) must be able to use the derives.
//! Before the fix the generated code referenced `::reliakit_csv` etc., which only resolve
//! when those crates are *direct* dependencies, so via the umbrella it failed to compile with
//! `error[E0433]: failed to resolve: use of undeclared crate or module reliakit_csv`. Pointing
//! the derive at the umbrella with `#[reliakit(crate = "reliakit")]` makes it emit
//! `::reliakit::csv` / `::reliakit::codec` / `::reliakit::json` instead.
#![cfg(all(
    feature = "derive",
    feature = "csv",
    feature = "json",
    feature = "codec"
))]

use reliakit::codec::{decode_from_slice_exact, encode_to_vec};
use reliakit::csv::{from_csv_str, to_csv_string};
use reliakit::derive::{
    CanonicalDecode, CanonicalEncode, CsvDecode, CsvEncode, JsonDecode, JsonEncode,
};
use reliakit::json::{from_json_str, to_json_string};

#[derive(
    Debug, PartialEq, CsvEncode, CsvDecode, JsonEncode, JsonDecode, CanonicalEncode, CanonicalDecode,
)]
#[reliakit(crate = "reliakit")]
struct Row {
    id: u32,
    name: String,
}

#[test]
fn umbrella_only_dependency_derives_roundtrip() {
    let rows = vec![
        Row {
            id: 1,
            name: "ada".into(),
        },
        Row {
            id: 2,
            name: "lin".into(),
        },
    ];

    // CSV via the umbrella path.
    let csv = to_csv_string(&rows);
    assert_eq!(from_csv_str::<Row>(&csv).unwrap(), rows);

    // JSON via the umbrella path.
    let json = to_json_string(&rows[0]);
    assert_eq!(from_json_str::<Row>(&json).unwrap(), rows[0]);

    // Canonical codec via the umbrella path.
    let encoded = encode_to_vec(&rows[0]).unwrap();
    assert_eq!(decode_from_slice_exact::<Row>(&encoded).unwrap(), rows[0]);
}

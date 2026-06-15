//! Round-trip typed CSV through the one `reliakit` name: derive the codec, write
//! rows with a header, then read them back into the same type.
//!
//! - [`reliakit::csv`] is strict, bounded, deterministic CSV (an RFC 4180
//!   subset): a field is quoted only when it has to be, and decoding rejects
//!   malformed rows instead of guessing.
//! - [`reliakit::derive`] generates the `CsvEncode`/`CsvDecode` impls, so the
//!   record type stays a plain struct.
//!
//! Run it:
//!
//! ```sh
//! cargo run -p reliakit --example typed_csv --features "csv derive"
//! ```

use reliakit::csv::{from_csv_str, to_csv_string};
use reliakit::derive::{CsvDecode, CsvEncode};

#[derive(Debug, PartialEq, CsvEncode, CsvDecode)]
struct Service {
    name: String,
    port: u16,
    enabled: bool,
}

fn main() {
    let services = vec![
        Service {
            name: "api".into(),
            port: 8080,
            enabled: true,
        },
        Service {
            name: "metrics".into(),
            port: 9090,
            enabled: false,
        },
        // The comma is data, not a separator: the writer quotes this field and
        // the reader puts it back together unchanged.
        Service {
            name: "db,primary".into(),
            port: 5432,
            enabled: true,
        },
    ];

    // Encode: a header row followed by one row per record.
    let csv = to_csv_string(&services);
    print!("written:\n{csv}");

    // Decode back into the same type. A missing column, a bad number, or the
    // wrong field count would be rejected here rather than silently accepted.
    let parsed: Vec<Service> = match from_csv_str(&csv) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("rejected: {err}");
            return;
        }
    };

    assert_eq!(parsed, services);
    println!("\nround-trip ok: {} records", parsed.len());
}

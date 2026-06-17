//! Read and write CSV with `reliakit-csv`, including a typed record.
//!
//! Run with: `cargo run -p reliakit-csv --example csv_basic`

use reliakit_csv::{
    CsvDecode, CsvDecodeError, CsvEncode, CsvField, CsvWriter, from_csv_str, read_str,
    to_csv_string,
};

#[derive(Debug, PartialEq)]
struct Service {
    name: String,
    port: u16,
    enabled: bool,
}

impl CsvEncode for Service {
    fn header() -> Vec<&'static str> {
        vec!["name", "port", "enabled"]
    }

    fn encode_fields(&self, out: &mut Vec<String>) {
        out.push(self.name.encode_field());
        out.push(self.port.encode_field());
        out.push(self.enabled.encode_field());
    }
}

impl CsvDecode for Service {
    fn decode_fields(fields: &[&str]) -> Result<Self, CsvDecodeError> {
        if fields.len() != 3 {
            return Err(CsvDecodeError::field_count());
        }
        Ok(Service {
            name: String::decode_field(fields[0]).map_err(|e| e.at_field(0))?,
            port: u16::decode_field(fields[1]).map_err(|e| e.at_field(1))?,
            enabled: bool::decode_field(fields[2]).map_err(|e| e.at_field(2))?,
        })
    }
}

fn main() {
    // Low-level: parse rows of strings.
    let rows = read_str("a,b\n1,2\n").unwrap();
    println!("rows: {rows:?}");

    // Low-level: write rows deterministically (a field is quoted only if needed).
    let mut writer = CsvWriter::new();
    writer.write_record(["plain", "needs,quote"]);
    print!("written: {}", writer.into_string());

    // Typed: round-trip a slice of records through a header + rows.
    let services = vec![
        Service {
            name: "api".into(),
            port: 8080,
            enabled: true,
        },
        Service {
            name: "worker, west".into(),
            port: 9000,
            enabled: false,
        },
    ];
    let text = to_csv_string(&services);
    print!("encoded:\n{text}");

    let decoded: Vec<Service> = from_csv_str(&text).unwrap();
    assert_eq!(decoded, services);
    println!("round-trip ok: {} record(s)", decoded.len());
}

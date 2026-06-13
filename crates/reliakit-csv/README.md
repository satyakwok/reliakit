<p align="center">
  <img src="https://raw.githubusercontent.com/satyakwok/reliakit/main/assets/reliakit-logo.png" alt="Reliakit" width="400">
</p>

# reliakit-csv

[![Crates.io](https://img.shields.io/crates/v/reliakit-csv.svg)](https://crates.io/crates/reliakit-csv)
[![Crates.io Downloads](https://img.shields.io/crates/d/reliakit-csv.svg)](https://crates.io/crates/reliakit-csv)
[![Docs.rs](https://docs.rs/reliakit-csv/badge.svg)](https://docs.rs/reliakit-csv)
[![CI](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml/badge.svg)](https://github.com/satyakwok/reliakit/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/satyakwok/reliakit/branch/main/graph/badge.svg?flag=reliakit-csv)](https://codecov.io/gh/satyakwok/reliakit/tree/main/crates/reliakit-csv)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/satyakwok/reliakit/blob/main/LICENSE)

Strict, bounded, and deterministic CSV for reliability-sensitive Rust.

`reliakit-csv` reads and writes a strict subset of [RFC 4180]. It is built for
systems that process **untrusted** CSV or need **predictable** output: it rejects
malformed quoting, enforces a rectangular shape (every record has the same number
of fields), applies explicit resource limits, reports errors with a location, and
serializes deterministically.

The crate has no dependencies, is `no_std`-friendly (with `alloc`), and forbids
unsafe code.

## What This Crate Does

- `read_str` / `read_str_with_limits` — parse text into `Vec<Vec<String>>`,
  strictly and within bounds.
- `CsvWriter` — build CSV text one record at a time; a field is quoted only when
  it must be, and every record ends with `\r\n`.
- `CsvField` — encode/decode a single field for the integer types, `bool`,
  `char`, `String`, `IpAddr` and `SocketAddr` types, and `Option<T>` (an empty field is `None`).
- `CsvEncode` / `CsvDecode` — map your record type to and from a row, with a
  header. `to_csv_string` / `from_csv_str` write and read a header row;
  `*_headerless` variants skip it.

## What This Crate Does Not Do

It does not infer column types, support configurable dialects (the delimiter is
`,` and the quote is `"`), stream over `std::io`, validate against a schema, or
recover leniently from malformed input. A successful read is a strong guarantee,
not a best effort.

## When To Use

- You parse CSV from an untrusted or semi-trusted source and want malformed input
  rejected, not silently repaired.
- You generate CSV that must be byte-for-byte reproducible (fixtures, exports,
  cache keys, diffs).
- You want a small, zero-dependency, `no_std`-friendly reader/writer.

## When Not To Use

- You need a configurable dialect (other delimiters, optional quoting modes) or
  lenient parsing of messy real-world files. Use a fuller-featured CSV crate.
- You need streaming over very large files without holding the input in memory.

## Installation

```toml
[dependencies]
reliakit-csv = "0.1"
```

For `no_std` with allocation:

```toml
[dependencies]
reliakit-csv = { version = "0.1", default-features = false, features = ["alloc"] }
```

## Example

```rust
use reliakit_csv::{read_str, CsvWriter};

// Read rows of strings, strictly.
let rows = read_str("name,city\nAda,London\n").unwrap();
assert_eq!(rows, [["name", "city"], ["Ada", "London"]]);

// Write deterministically: a field is quoted only when it must be.
let mut writer = CsvWriter::new();
writer.write_record(["plain", "needs,quote"]);
assert_eq!(writer.into_string(), "plain,\"needs,quote\"\r\n");
```

Typed records with a header:

```rust
use reliakit_csv::{from_csv_str, to_csv_string, CsvDecode, CsvDecodeError, CsvEncode, CsvField};

#[derive(Debug, PartialEq)]
struct Row {
    id: u32,
    name: String,
}

impl CsvEncode for Row {
    fn header() -> Vec<&'static str> {
        vec!["id", "name"]
    }
    fn encode_fields(&self, out: &mut Vec<String>) {
        out.push(self.id.encode_field());
        out.push(self.name.encode_field());
    }
}

impl CsvDecode for Row {
    fn decode_fields(fields: &[&str]) -> Result<Self, CsvDecodeError> {
        if fields.len() != 2 {
            return Err(CsvDecodeError::field_count());
        }
        Ok(Row {
            id: u32::decode_field(fields[0]).map_err(|e| e.at_field(0))?,
            name: String::decode_field(fields[1]).map_err(|e| e.at_field(1))?,
        })
    }
}

let rows = vec![Row { id: 1, name: "Ada".into() }];
let text = to_csv_string(&rows);
assert_eq!(text, "id,name\r\n1,Ada\r\n");
assert_eq!(from_csv_str::<Row>(&text).unwrap(), rows);
```

## Format

A strict subset of RFC 4180:

- UTF-8 text; the delimiter is `,` and the quote character is `"`.
- The writer quotes a field only if it contains `,`, `"`, `\r`, or `\n`, doubles
  an embedded `"`, and terminates every record with `\r\n`.
- The reader accepts `\n` and `\r\n` as record terminators and rejects a bare
  `\r`, a `"` inside an unquoted field, text after a closing quote, and an
  unterminated quoted field.
- Records are **rectangular**: every record must have the same number of fields
  as the first, or the read fails.
- The wire format is fixed and covered by exact-output tests; it will not change
  in a backward-incompatible way within `0.1`.

## Feature Flags

| Flag | Default | Description |
|---|---|---|
| `std` | yes | Enables the standard library (`std::error::Error`); implies `alloc` |
| `alloc` | no | The crate always needs `alloc` for owned strings and records |

## Safety

This crate is `#![forbid(unsafe_code)]`.

## Minimum Supported Rust Version

Rust `1.85` and newer. No nightly features are used.

## Status

Pre-1.0. The API is small and the wire format is fixed; the crate may receive
backward-compatible refinements before a `1.0` release.

## License

Licensed under the MIT License. See [`LICENSE`](https://github.com/satyakwok/reliakit/blob/main/LICENSE).

[RFC 4180]: https://www.rfc-editor.org/rfc/rfc4180

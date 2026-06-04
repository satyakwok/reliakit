#![no_main]

//! Any value that parses must survive a compact round-trip unchanged:
//! `parse -> to_compact_string -> parse` yields an equal value, and the compact
//! form is stable.

use libfuzzer_sys::fuzz_target;
use reliakit_json::{parse, parse_str, to_compact_string};

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = parse(data) {
        let compact = to_compact_string(&value);
        let reparsed = parse_str(&compact).expect("compact output must reparse");
        assert_eq!(reparsed, value);
        assert_eq!(to_compact_string(&reparsed), compact);
    }
});

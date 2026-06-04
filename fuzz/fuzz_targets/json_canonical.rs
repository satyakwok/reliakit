#![no_main]

//! RFC 8785 canonicalization must never panic and must be idempotent:
//! canonicalizing a value, reparsing the output, and canonicalizing again
//! yields identical bytes.

use libfuzzer_sys::fuzz_target;
use reliakit_json::{parse, parse_str, to_canonical_string};

fuzz_target!(|data: &[u8]| {
    if let Ok(value) = parse(data) {
        if let Ok(canonical) = to_canonical_string(&value) {
            let reparsed = parse_str(&canonical).expect("canonical output must reparse");
            let again = to_canonical_string(&reparsed).expect("canonical must re-canonicalize");
            assert_eq!(canonical, again, "canonicalization must be idempotent");
        }
    }
});

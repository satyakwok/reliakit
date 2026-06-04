#![no_main]

//! Parsing arbitrary bytes must never panic, hang, or use unbounded memory.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Default limits bound depth, sizes, and node counts on untrusted input.
    let _ = reliakit_json::parse(data);
});

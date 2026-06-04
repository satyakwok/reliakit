#![no_main]

//! Decoding arbitrary bytes into representative types must never panic; the
//! decoder validates length prefixes and tags before allocating.

use libfuzzer_sys::fuzz_target;
use reliakit_codec::decode_from_slice_exact;

fuzz_target!(|data: &[u8]| {
    let _ = decode_from_slice_exact::<Vec<String>>(data);
    let _ = decode_from_slice_exact::<Vec<i64>>(data);
    let _ = decode_from_slice_exact::<(u64, bool, String)>(data);
    let _ = decode_from_slice_exact::<Option<Vec<u8>>>(data);
});

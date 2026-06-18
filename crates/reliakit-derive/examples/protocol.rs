//! A real, runnable example: a small protocol built from a derived struct and a
//! derived enum with all three variant kinds (including a nested struct inside a
//! struct variant). It encodes each frame to canonical bytes, prints them,
//! decodes them back, and checks the round-trip, then shows that an unknown
//! variant tag is rejected.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p reliakit-derive --example protocol
//! ```

use reliakit_codec::{CodecErrorKind, decode_from_slice_exact, encode_to_vec};
use reliakit_derive::{CanonicalDecode, CanonicalEncode};

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
struct Header {
    version: u8,
    request_id: u32,
}

#[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
enum Frame {
    // tag 0: unit variant
    Ping,
    // tag 1: tuple variant
    SetName(String),
    // tag 2: struct variant, with a nested derived struct
    Connect { header: Header, port: u16 },
}

fn main() {
    let frames = [
        Frame::Ping,
        Frame::SetName("router-1".to_string()),
        Frame::Connect {
            header: Header {
                version: 1,
                request_id: 42,
            },
            port: 8080,
        },
    ];

    for frame in &frames {
        let bytes = encode_to_vec(frame).expect("encode");
        let decoded = decode_from_slice_exact::<Frame>(&bytes).expect("decode");
        println!("{frame:?}");
        println!("  encoded ({} bytes): {bytes:02x?}", bytes.len());
        println!("  decoded: {decoded:?}");
        assert_eq!(&decoded, frame, "round-trip must be lossless");
    }

    // The variant tag is the zero-based declaration index as a little-endian u32,
    // so `Ping` is `00 00 00 00` and `Connect` is `02 00 00 00`.
    assert_eq!(encode_to_vec(&Frame::Ping).unwrap(), [0, 0, 0, 0]);

    // An unknown tag is a clear decode error, not a panic or a wrong value.
    let err = decode_from_slice_exact::<Frame>(&[9, 0, 0, 0]).unwrap_err();
    assert_eq!(err.kind(), CodecErrorKind::InvalidValue);
    println!("unknown tag rejected: {}", err.message());

    println!("all frames round-tripped");
}

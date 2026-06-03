use reliakit_codec::{
    decode_from_slice, decode_from_slice_exact, encode_to_vec, CodecError, CodecErrorKind,
};

fn main() -> Result<(), CodecError> {
    let port = 8080u16;
    let encoded = encode_to_vec(&port)?;
    assert_eq!(encoded, [0x90, 0x1f]);
    assert_eq!(decode_from_slice_exact::<u16>(&encoded)?, port);

    let text = "api";
    let encoded = encode_to_vec(text)?;
    assert_eq!(encoded, [3, 0, 0, 0, b'a', b'p', b'i']);
    assert_eq!(decode_from_slice_exact::<String>(&encoded)?, text);

    let (value, remaining) = decode_from_slice::<u8>(&[1, 2])?;
    assert_eq!(value, 1);
    assert_eq!(remaining, 1);

    let err = decode_from_slice_exact::<u8>(&[1, 2]).unwrap_err();
    assert_eq!(err.kind(), CodecErrorKind::TrailingBytes);

    Ok(())
}

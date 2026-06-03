use reliakit_codec::{
    decode_from_slice_exact, encode_to_vec, CanonicalDecode, CanonicalEncode, CodecError,
    DecodeSource, EncodeSink,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProtocolMessage {
    id: u32,
    kind: MessageKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MessageKind {
    Ping,
    Data(Vec<u8>),
}

impl CanonicalEncode for ProtocolMessage {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        self.id.encode(writer)?;
        self.kind.encode(writer)
    }
}

impl CanonicalDecode for ProtocolMessage {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        Ok(Self {
            id: u32::decode(reader)?,
            kind: MessageKind::decode(reader)?,
        })
    }
}

impl CanonicalEncode for MessageKind {
    fn encode<W: EncodeSink + ?Sized>(&self, writer: &mut W) -> Result<(), CodecError> {
        match self {
            Self::Ping => 0u8.encode(writer),
            Self::Data(bytes) => {
                1u8.encode(writer)?;
                bytes.encode(writer)
            }
        }
    }
}

impl CanonicalDecode for MessageKind {
    fn decode<R: DecodeSource + ?Sized>(reader: &mut R) -> Result<Self, CodecError> {
        match u8::decode(reader)? {
            0 => Ok(Self::Ping),
            1 => Vec::<u8>::decode(reader).map(Self::Data),
            _ => Err(CodecError::invalid_value(
                "unknown MessageKind tag: expected 0x00 or 0x01",
            )),
        }
    }
}

fn main() -> Result<(), CodecError> {
    let message = ProtocolMessage {
        id: 42,
        kind: MessageKind::Data(vec![1, 2, 3]),
    };

    let encoded = encode_to_vec(&message)?;
    assert_eq!(encoded, [42, 0, 0, 0, 1, 3, 0, 0, 0, 1, 2, 3]);
    assert_eq!(
        decode_from_slice_exact::<ProtocolMessage>(&encoded)?,
        message
    );

    let err = decode_from_slice_exact::<MessageKind>(&[9]).unwrap_err();
    assert_eq!(
        err.message(),
        "unknown MessageKind tag: expected 0x00 or 0x01"
    );

    Ok(())
}

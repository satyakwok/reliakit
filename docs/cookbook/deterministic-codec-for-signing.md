# Deterministic encode/decode for signing

## Problem

You need to hash, sign, or compare data by its bytes: an HMAC over a request, a
content address, a Merkle leaf, a signature payload. General-purpose serializers
do not promise a fixed byte layout: field order, whitespace, float formatting,
and map ordering can all vary between versions or runs. The same logical value
must always produce the same bytes, on every machine and build.

## Use

- `reliakit-codec`: a canonical binary encoding with a fixed, documented byte
  layout and strict decoding.

## Example

```rust
use reliakit_codec::{decode_from_slice_exact, encode_to_vec, CodecError};

fn main() -> Result<(), CodecError> {
    // The byte layout is fixed and documented, so these are stable across runs.
    let port = 8080u16;
    let bytes = encode_to_vec(&port)?;
    assert_eq!(bytes, [0x90, 0x1f]); // little-endian u16

    let text = "api";
    let bytes = encode_to_vec(text)?;
    assert_eq!(bytes, [3, 0, 0, 0, b'a', b'p', b'i']); // u32 length prefix + bytes

    // Decoding is strict: it must consume exactly the input.
    assert_eq!(decode_from_slice_exact::<String>(&bytes)?, text);

    // Sign or hash the canonical bytes, not a formatted string:
    // let mac = hmac(key, &bytes);
    Ok(())
}
```

## Run it

```sh
cargo run -p reliakit-codec --example basic_encoding
cargo run -p reliakit-codec --example protocol_message
```

## Why this works

The encoding pins an exact byte layout and the crate locks it with byte-exact
tests, so encoding a value is reproducible everywhere. Decoding is strict: wrong
type, missing bytes, or trailing bytes are errors (`decode_from_slice_exact`
rejects leftovers with `CodecErrorKind::TrailingBytes`), so a verifier cannot be
tricked by padding. Identical bytes in means identical bytes out, which is exactly
what a signature or hash depends on.

## Common mistakes

- **Signing a formatted string** (`format!`, JSON, `Debug`). Those layouts are not
  guaranteed stable; sign the canonical bytes instead.
- **Ignoring trailing bytes** on the verify side. Use `decode_from_slice_exact`
  so extra bytes are rejected, not silently dropped.
- **Assuming any serializer is canonical.** Most optimize for flexibility, not a
  fixed layout. Use a format that documents and tests its bytes.

## When not to use this

- Do not use it as a general-purpose serialization format for evolving data
  unless the schema is stable. The byte layout is fixed; it is not a
  schema-evolution system.
- It encodes bytes; it does not hash or sign. Pair it with your own HMAC, hash, or
  signature over the canonical bytes.
- For a human-readable or interoperable wire format, use `reliakit-json` (strict,
  bounded JSON) instead; reach for the codec when byte-stability matters more than
  readability.

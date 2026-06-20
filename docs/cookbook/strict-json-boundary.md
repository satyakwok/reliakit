# Strict JSON at the boundary

## Problem

JSON arriving from a network or a file is untrusted: it can be malformed, hostile
(deeply nested to blow the stack, huge to exhaust memory), or quietly wrong
(duplicate keys, trailing data). Many parsers are lenient and accept things you
did not intend, or have no size limits. At a boundary you want strict parsing
with explicit bounds, so bad input is rejected with an error instead of silently
accepted.

## Use

- `reliakit-json`: strict, bounded, deterministic JSON. Parse under a limit
  profile and reject anything outside the RFC.

## Example

```rust
use reliakit_json::{parse_with_limits, to_compact_string, JsonLimits, JsonValue};

fn main() {
    let input = br#"{ "service": "api", "port": 8080, "tags": ["a", "b"] }"#;

    // Parse untrusted bytes under a conservative size/depth profile.
    let value = match parse_with_limits(input, JsonLimits::conservative()) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("rejected: {err}");
            return;
        }
    };

    if let JsonValue::Object(object) = &value {
        // Read defensively: a value that is missing or not a u64 yields None,
        // never a panic, because the input is untrusted.
        let port = object
            .get("port")
            .and_then(JsonValue::as_number)
            .and_then(|n| n.to_u64().ok());
        if let Some(port) = port {
            println!("port = {port}");
        }
    }

    // Deterministic, member-order-preserving output.
    println!("{}", to_compact_string(&value));

    // Strict by default: each of these is an error, not a quiet accept.
    for bad in [&br#"{"a":1,"a":2}"#[..], &b"[1,]"[..], &b"NaN"[..]] {
        assert!(parse_with_limits(bad, JsonLimits::conservative()).is_err());
    }
}
```

## Run it

```sh
cargo run -p reliakit-json --example basic
```

## Why this works

`parse_with_limits` enforces a size and depth profile before and during parsing,
so a hostile document is rejected instead of allocating without bound. Parsing is
strict: duplicate keys, comments, trailing commas, trailing data, and `NaN`/`Inf`
are errors. Serialization is deterministic and preserves member order, so the same
value always produces the same bytes. You validate the *shape* at the edge, then
work with a `JsonValue` you trust.

## Common mistakes

- **Parsing without limits.** An unbounded parser is a denial-of-service vector.
  Use `parse_with_limits` with a profile sized for your inputs.
- **Assuming leniency is harmless.** Duplicate keys and trailing data often signal
  a bug or an attack; strict parsing surfaces them.
- **Relying on JSON text for signing.** Formatting is deterministic here, but for
  signatures prefer a canonical binary form (see
  [Deterministic encode/decode for signing](deterministic-codec-for-signing.md)).

## When not to use this

- It is a strict JSON boundary, not a schema validator. Pair it with your own
  field checks (or `reliakit-validate`) for business rules beyond "is this valid,
  bounded JSON?".
- For a fixed, byte-stable layout to hash or sign, use `reliakit-codec` rather
  than JSON text.
- Limits protect against oversized or deeply nested input; set the profile to fit
  your real payloads rather than leaving them effectively unbounded.

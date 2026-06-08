//! Parse untrusted JSON strictly, then lift the raw fields into validated types
//! — all through the one `reliakit` name.
//!
//! - [`reliakit::json`] parses under conservative limits and rejects malformed
//!   or ambiguous input.
//! - [`reliakit::primitives`] turns the extracted fields into types that hold
//!   their own invariants.
//!
//! Run it:
//!
//! ```sh
//! cargo run -p reliakit --example typed_json --features "json primitives"
//! ```

use reliakit::json::{parse_with_limits, JsonLimits, JsonValue};
use reliakit::primitives::{Hostname, Port};

fn main() {
    // Treat this as untrusted input and parse it under a conservative limit
    // profile (bounded depth, length, and number sizes).
    let input = r#"{ "host": "api.internal", "port": 8080 }"#;
    let value = match parse_with_limits(input.as_bytes(), JsonLimits::conservative()) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("rejected: {err}");
            return;
        }
    };

    let JsonValue::Object(object) = &value else {
        eprintln!("expected a JSON object");
        return;
    };

    // Lift each raw field into a validated primitive. A field that is missing,
    // the wrong JSON type, or out of range simply does not produce a value.
    let host = object
        .get("host")
        .and_then(JsonValue::as_str)
        .and_then(|s| Hostname::new(s).ok());

    let port = object
        .get("port")
        .and_then(JsonValue::as_number)
        .and_then(|n| n.to_u64().ok())
        .and_then(|n| u16::try_from(n).ok())
        .and_then(|p| Port::new(p).ok());

    match (host, port) {
        (Some(host), Some(port)) => println!("typed config: {host}:{port}"),
        _ => println!("config failed typed validation"),
    }
}

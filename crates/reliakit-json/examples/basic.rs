//! Parse untrusted JSON strictly, inspect it, and serialize it back.
//!
//! ```sh
//! cargo run -p reliakit-json --example basic
//! ```

use reliakit_json::{JsonLimits, JsonValue, parse_with_limits, to_compact_string};

fn main() {
    // Treat this as untrusted input: parse under a conservative limit profile.
    let input = r#"{ "service": "api", "port": 8080, "tags": ["a", "b"], "enabled": true }"#;
    let limits = JsonLimits::conservative();

    let value = match parse_with_limits(input.as_bytes(), limits) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("rejected: {err}");
            return;
        }
    };

    if let JsonValue::Object(object) = &value {
        if let Some(port) = object.get("port").and_then(JsonValue::as_number) {
            println!("port = {}", port.to_u64().unwrap());
        }
        if let Some(tags) = object.get("tags").and_then(JsonValue::as_array) {
            println!("tags = {}", tags.len());
        }
    }

    // Deterministic, member-order-preserving serialization.
    println!("compact = {}", to_compact_string(&value));

    // Strict by default: these are all rejected.
    for bad in [r#"{"a":1,"a":2}"#, "{ /* c */ }", "[1,]", "NaN"] {
        match parse_with_limits(bad.as_bytes(), limits) {
            Ok(_) => println!("accepted (unexpected): {bad}"),
            Err(err) => println!("rejected {bad:?}: {}", err.kind_str()),
        }
    }
}

// Small helper so the example prints something readable for each error kind.
trait KindStr {
    fn kind_str(&self) -> &'static str;
}

impl KindStr for reliakit_json::JsonError {
    fn kind_str(&self) -> &'static str {
        use reliakit_json::JsonErrorKind::*;
        match self.kind() {
            DuplicateKey => "duplicate key",
            UnexpectedByte => "unexpected byte",
            InvalidNumber => "invalid number",
            TrailingData => "trailing data",
            _ => "other",
        }
    }
}

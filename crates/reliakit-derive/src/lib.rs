//! Derive macros for `reliakit` traits.
//!
//! This crate provides `#[derive(...)]` support for the trait pairs defined by
//! other `reliakit-*` crates. It is written using only the standard
//! [`proc_macro`] API and pulls in no third-party crates. To stay free of a
//! full Rust-grammar parser, it reads only what the generated code needs — the
//! type name and its field shape — and rejects constructs it does not yet
//! handle with a clear compile error rather than guessing.
//!
//! # Supported types
//!
//! - structs with named fields
//! - tuple structs
//! - unit structs
//! - enums with unit, tuple, and struct variants
//!
//! Unions, generic types, generic enums, enums with explicit discriminants or a
//! `#[repr(...)]`, and empty enums are rejected with a compile error. The JSON
//! derives currently cover structs only; enums are rejected for now.
//!
//! # `reliakit-codec`
//!
//! [`CanonicalEncode`] and [`CanonicalDecode`] generate implementations of the
//! same-named traits from `reliakit-codec`, encoding each field in declaration
//! order. The derived code is exactly what a handwritten implementation would
//! be — one `encode`/`decode` call per field, in order.
//!
//! ```
//! # // The derives reference `::reliakit_codec`, which must be a dependency of
//! # // the crate that uses them.
//! use reliakit_codec::{decode_from_slice_exact, encode_to_vec};
//! use reliakit_derive::{CanonicalDecode, CanonicalEncode};
//!
//! #[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
//! struct Point {
//!     x: u16,
//!     y: u16,
//! }
//!
//! let encoded = encode_to_vec(&Point { x: 10, y: 20 }).unwrap();
//! assert_eq!(encoded, [10, 0, 20, 0]);
//! assert_eq!(decode_from_slice_exact::<Point>(&encoded).unwrap(), Point { x: 10, y: 20 });
//! ```
//!
//! Enums are supported too. Each variant is tagged by its zero-based
//! declaration index, encoded as a little-endian `u32`, followed by the
//! variant's fields in declaration order:
//!
//! ```
//! use reliakit_codec::{decode_from_slice_exact, encode_to_vec};
//! use reliakit_derive::{CanonicalDecode, CanonicalEncode};
//!
//! #[derive(Debug, PartialEq, CanonicalEncode, CanonicalDecode)]
//! enum Message {
//!     Ping,
//!     Pong,
//! }
//!
//! assert_eq!(encode_to_vec(&Message::Ping).unwrap(), [0, 0, 0, 0]);
//! assert_eq!(encode_to_vec(&Message::Pong).unwrap(), [1, 0, 0, 0]);
//! assert_eq!(decode_from_slice_exact::<Message>(&[1, 0, 0, 0]).unwrap(), Message::Pong);
//! ```
//!
//! # `reliakit-json`
//!
//! [`JsonEncode`] and [`JsonDecode`] generate implementations of the same-named
//! `reliakit-json` traits. A struct with named fields becomes a JSON object in
//! declaration order, a tuple struct becomes an array, and a unit struct
//! becomes `null`. Decoding is strict; unknown object fields are ignored.
//!
//! ```
//! use reliakit_derive::{JsonDecode, JsonEncode};
//! use reliakit_json::{from_json_str, to_json_string};
//!
//! #[derive(Debug, PartialEq, JsonEncode, JsonDecode)]
//! struct Point {
//!     x: u16,
//!     y: u16,
//! }
//!
//! let json = to_json_string(&Point { x: 10, y: 20 });
//! assert_eq!(json, r#"{"x":10,"y":20}"#);
//! assert_eq!(from_json_str::<Point>(&json).unwrap(), Point { x: 10, y: 20 });
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use proc_macro::{Delimiter, Spacing, TokenStream, TokenTree};

/// Derives `reliakit_codec::CanonicalEncode`, encoding each field in
/// declaration order (for enums, the variant tag first).
///
/// See the [crate] documentation for supported types and limitations.
#[proc_macro_derive(CanonicalEncode)]
pub fn derive_canonical_encode(input: TokenStream) -> TokenStream {
    match Parsed::from_input(input) {
        Ok(parsed) => parsed.canonical_encode_impl(),
        Err(message) => compile_error(&message),
    }
}

/// Derives `reliakit_codec::CanonicalDecode`, decoding each field in
/// declaration order (for enums, the variant tag first).
///
/// See the [crate] documentation for supported types and limitations.
#[proc_macro_derive(CanonicalDecode)]
pub fn derive_canonical_decode(input: TokenStream) -> TokenStream {
    match Parsed::from_input(input) {
        Ok(parsed) => parsed.canonical_decode_impl(),
        Err(message) => compile_error(&message),
    }
}

/// Derives `reliakit_json::JsonEncode`: a struct with named fields becomes a
/// JSON object (in declaration order), a tuple struct becomes a JSON array, and
/// a unit struct becomes `null`.
///
/// Enums are not supported yet. See the [crate] documentation.
#[proc_macro_derive(JsonEncode)]
pub fn derive_json_encode(input: TokenStream) -> TokenStream {
    match Parsed::from_input(input).and_then(|parsed| parsed.json_encode_impl()) {
        Ok(tokens) => tokens,
        Err(message) => compile_error(&message),
    }
}

/// Derives `reliakit_json::JsonDecode`, the inverse of [`macro@JsonEncode`].
/// Decoding is strict: the JSON shape must match, and required object fields
/// must be present; unknown object fields are ignored.
///
/// Enums are not supported yet. See the [crate] documentation.
#[proc_macro_derive(JsonDecode)]
pub fn derive_json_decode(input: TokenStream) -> TokenStream {
    match Parsed::from_input(input).and_then(|parsed| parsed.json_decode_impl()) {
        Ok(tokens) => tokens,
        Err(message) => compile_error(&message),
    }
}

/// Which item keyword the derive input started with.
enum Kind {
    Struct,
    Enum,
    Union,
}

/// The field shape of a struct body or a single enum variant, reduced to
/// exactly what the generated code needs.
enum Shape {
    /// Named fields, in declaration order.
    Named(Vec<String>),
    /// Tuple fields, by count.
    Tuple(usize),
    /// No fields (unit struct or unit variant).
    Unit,
}

/// One validated enum variant: its name and field shape.
struct Variant {
    name: String,
    shape: Shape,
}

/// The validated body the derive will implement.
enum Body {
    /// A struct with the given field shape.
    Struct(Shape),
    /// An enum with the given variants, in declaration order.
    Enum(Vec<Variant>),
}

/// A validated item ready for code generation.
struct Parsed {
    name: String,
    body: Body,
}

/// One enum variant as read from tokens, before validation.
struct RawVariant {
    name: String,
    /// The variant's field shape, or a message if its syntax is unsupported.
    shape: Result<Shape, String>,
    /// Whether the variant carried an explicit `= discriminant`.
    has_discriminant: bool,
}

/// The item body as read from tokens, before validation.
enum RawBody {
    Struct(Shape),
    Enum(Vec<RawVariant>),
    Union,
}

/// The whole item as read from tokens, before any semantic validation. Kept
/// free of `proc_macro` types so [`validate`] is pure and unit-testable.
struct Raw {
    name: String,
    has_generics: bool,
    saw_repr: bool,
    body: RawBody,
}

impl Parsed {
    /// Reads and validates a derive input.
    fn from_input(input: TokenStream) -> Result<Self, String> {
        validate(classify(input)?)
    }

    fn canonical_encode_impl(&self) -> TokenStream {
        let statements = match &self.body {
            Body::Struct(shape) => struct_encode_statements(shape),
            Body::Enum(variants) => enum_encode_statements(variants),
        };

        format!(
            "impl ::reliakit_codec::CanonicalEncode for {name} {{\n\
             fn encode<__W: ::reliakit_codec::EncodeSink + ?Sized>(&self, __writer: &mut __W) \
             -> ::core::result::Result<(), ::reliakit_codec::CodecError> {{\n\
             {statements}\n\
             ::core::result::Result::Ok(())\n\
             }}\n\
             }}",
            name = self.name,
        )
        .parse()
        .expect("reliakit-derive generated invalid CanonicalEncode tokens")
    }

    fn canonical_decode_impl(&self) -> TokenStream {
        let value = match &self.body {
            Body::Struct(shape) => struct_decode_value(shape),
            Body::Enum(variants) => enum_decode_value(&self.name, variants),
        };

        format!(
            "impl ::reliakit_codec::CanonicalDecode for {name} {{\n\
             fn decode<__R: ::reliakit_codec::DecodeSource + ?Sized>(__reader: &mut __R) \
             -> ::core::result::Result<Self, ::reliakit_codec::CodecError> {{\n\
             {value}\n\
             }}\n\
             }}",
            name = self.name,
        )
        .parse()
        .expect("reliakit-derive generated invalid CanonicalDecode tokens")
    }

    fn json_encode_impl(&self) -> Result<TokenStream, String> {
        let value = match &self.body {
            Body::Struct(shape) => json_encode_value(shape),
            Body::Enum(_) => {
                return Err("reliakit-derive: JsonEncode does not support enums yet".into())
            }
        };

        Ok(format!(
            "impl ::reliakit_json::JsonEncode for {name} {{\n\
             fn to_json_value(&self) -> ::reliakit_json::JsonValue {{\n\
             {value}\n\
             }}\n\
             }}",
            name = self.name,
        )
        .parse()
        .expect("reliakit-derive generated invalid JsonEncode tokens"))
    }

    fn json_decode_impl(&self) -> Result<TokenStream, String> {
        let body = match &self.body {
            Body::Struct(shape) => json_decode_body(shape),
            Body::Enum(_) => {
                return Err("reliakit-derive: JsonDecode does not support enums yet".into())
            }
        };

        Ok(format!(
            "impl ::reliakit_json::JsonDecode for {name} {{\n\
             fn from_json_value(__value: &::reliakit_json::JsonValue) \
             -> ::core::result::Result<Self, ::reliakit_json::JsonDecodeError> {{\n\
             {body}\n\
             }}\n\
             }}",
            name = self.name,
        )
        .parse()
        .expect("reliakit-derive generated invalid JsonDecode tokens"))
    }
}

/// The JSON object key for a field: a raw identifier's `r#` prefix is dropped.
fn json_key(field: &str) -> &str {
    field.strip_prefix("r#").unwrap_or(field)
}

/// The body of a struct's `JsonEncode::to_json_value`.
fn json_encode_value(shape: &Shape) -> String {
    match shape {
        Shape::Named(fields) => {
            let mut inserts = String::new();
            for field in fields {
                let key = json_key(field);
                inserts.push_str(&format!(
                    "__object.insert({key:?}.into(), \
                     ::reliakit_json::JsonEncode::to_json_value(&self.{field}));",
                ));
            }
            format!(
                "let mut __object = ::reliakit_json::JsonObject::new();\n\
                 {inserts}\n\
                 ::reliakit_json::JsonValue::Object(__object)"
            )
        }
        Shape::Tuple(count) => {
            let mut items = String::new();
            for index in 0..*count {
                items.push_str(&format!(
                    "::reliakit_json::JsonEncode::to_json_value(&self.{index}),"
                ));
            }
            format!("::reliakit_json::JsonValue::array([{items}])")
        }
        Shape::Unit => "::reliakit_json::JsonValue::Null".to_string(),
    }
}

/// The body of a struct's `JsonDecode::from_json_value`.
fn json_decode_body(shape: &Shape) -> String {
    match shape {
        Shape::Named(fields) => {
            let mut inner = String::new();
            for field in fields {
                let key = json_key(field);
                let missing = format!("missing field `{key}`");
                inner.push_str(&format!(
                    "{field}: ::reliakit_json::JsonDecode::from_json_value(\
                     __object.get({key:?}).ok_or_else(|| \
                     ::reliakit_json::JsonDecodeError::missing_field({missing:?}))?)?,",
                ));
            }
            format!(
                "let __object = __value.as_object().ok_or_else(|| \
                 ::reliakit_json::JsonDecodeError::unexpected_type(\"expected a JSON object\"))?;\n\
                 ::core::result::Result::Ok(Self {{ {inner} }})"
            )
        }
        Shape::Tuple(count) => {
            let mut inner = String::new();
            for index in 0..*count {
                inner.push_str(&format!(
                    "::reliakit_json::JsonDecode::from_json_value(&__array[{index}])?,"
                ));
            }
            format!(
                "let __array = __value.as_array().ok_or_else(|| \
                 ::reliakit_json::JsonDecodeError::unexpected_type(\"expected a JSON array\"))?;\n\
                 if __array.len() != {count} {{ return ::core::result::Result::Err(\
                 ::reliakit_json::JsonDecodeError::unexpected_type(\
                 \"JSON array has the wrong number of elements\")); }}\n\
                 ::core::result::Result::Ok(Self({inner}))"
            )
        }
        Shape::Unit => "if !__value.is_null() {\n\
             return ::core::result::Result::Err(\
             ::reliakit_json::JsonDecodeError::unexpected_type(\
             \"expected JSON null for a unit struct\"));\n\
             }\n\
             ::core::result::Result::Ok(Self)"
            .to_string(),
    }
}

/// Validates a [`Raw`] item, rejecting unsupported forms with a descriptive
/// message. Pure — it touches no `proc_macro` types, so it is unit-testable.
fn validate(raw: Raw) -> Result<Parsed, String> {
    match raw.body {
        RawBody::Union => Err("reliakit-derive does not support unions".into()),
        RawBody::Struct(shape) => {
            if raw.has_generics {
                return Err("reliakit-derive does not support generic types yet".into());
            }
            Ok(Parsed {
                name: raw.name,
                body: Body::Struct(shape),
            })
        }
        RawBody::Enum(raw_variants) => {
            if raw.has_generics {
                return Err("reliakit-derive does not support generic types yet".into());
            }
            if raw.saw_repr {
                return Err("reliakit-derive does not support `#[repr(...)]` on enums; \
                            variant tags are always the u32 declaration index"
                    .into());
            }
            let mut variants = Vec::new();
            for raw_variant in raw_variants {
                if raw_variant.has_discriminant {
                    return Err(format!(
                        "reliakit-derive does not support explicit enum discriminants \
                         (`{} = ...`); variant tags are the u32 declaration index",
                        raw_variant.name
                    ));
                }
                match raw_variant.shape {
                    Ok(shape) => variants.push(Variant {
                        name: raw_variant.name,
                        shape,
                    }),
                    Err(message) => return Err(message),
                }
            }
            if variants.is_empty() {
                return Err("reliakit-derive cannot derive for an empty enum \
                            (there is no variant to encode or decode)"
                    .into());
            }
            Ok(Parsed {
                name: raw.name,
                body: Body::Enum(variants),
            })
        }
    }
}

/// Reads a derive input into a [`Raw`] item. Touches `proc_macro` types; its
/// happy paths are exercised by the integration and example tests.
fn classify(input: TokenStream) -> Result<Raw, String> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();

    // Find the item keyword, skipping outer attributes and visibility, noting a
    // `#[repr(...)]` so enums can reject it (struct behavior is unchanged).
    let mut idx = 0;
    let mut saw_repr = false;
    let kind = loop {
        match tokens.get(idx) {
            Some(TokenTree::Ident(ident)) => match ident.to_string().as_str() {
                "struct" => break Kind::Struct,
                "enum" => break Kind::Enum,
                "union" => break Kind::Union,
                _ => idx += 1,
            },
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                if attr_is_repr(group.stream()) {
                    saw_repr = true;
                }
                idx += 1;
            }
            Some(_) => idx += 1,
            None => return Err("reliakit-derive: expected a struct, enum, or union".into()),
        }
    };

    idx += 1;
    let name = match tokens.get(idx) {
        Some(TokenTree::Ident(ident)) => ident.to_string(),
        _ => return Err("reliakit-derive: expected a type name after the item keyword".into()),
    };
    idx += 1;

    let has_generics =
        matches!(tokens.get(idx), Some(TokenTree::Punct(punct)) if punct.as_char() == '<');

    let body = if has_generics {
        // A generic item is rejected by validation before its body is used, and
        // `idx` here points at the `<` parameters rather than the body, so don't
        // try to read it. The placeholder body is never inspected.
        match kind {
            Kind::Struct => RawBody::Struct(Shape::Unit),
            Kind::Enum => RawBody::Enum(Vec::new()),
            Kind::Union => RawBody::Union,
        }
    } else {
        match kind {
            // The union body is never read: validation rejects unions outright.
            Kind::Union => RawBody::Union,
            Kind::Struct => match tokens.get(idx) {
                Some(TokenTree::Group(group)) => match group.delimiter() {
                    Delimiter::Brace => RawBody::Struct(Shape::Named(named_fields(group.stream()))),
                    Delimiter::Parenthesis => {
                        RawBody::Struct(Shape::Tuple(count_fields(group.stream())))
                    }
                    _ => return Err("reliakit-derive: unexpected struct body".into()),
                },
                Some(TokenTree::Punct(punct)) if punct.as_char() == ';' => {
                    RawBody::Struct(Shape::Unit)
                }
                _ => return Err("reliakit-derive: unexpected struct body".into()),
            },
            Kind::Enum => match tokens.get(idx) {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    RawBody::Enum(raw_variants(group.stream()))
                }
                _ => return Err("reliakit-derive: expected a braced enum body".into()),
            },
        }
    };

    Ok(Raw {
        name,
        has_generics,
        saw_repr,
        body,
    })
}

/// Encode statements for a struct body (one `encode` call per field, in order).
fn struct_encode_statements(shape: &Shape) -> String {
    let mut body = String::new();
    match shape {
        Shape::Named(fields) => {
            for field in fields {
                body.push_str(&format!(
                    "::reliakit_codec::CanonicalEncode::encode(&self.{field}, __writer)?;",
                ));
            }
        }
        Shape::Tuple(count) => {
            for index in 0..*count {
                body.push_str(&format!(
                    "::reliakit_codec::CanonicalEncode::encode(&self.{index}, __writer)?;",
                ));
            }
        }
        Shape::Unit => {}
    }
    body
}

/// The decode body for a struct (returns `Ok(Self { .. })`).
fn struct_decode_value(shape: &Shape) -> String {
    let construct = match shape {
        Shape::Named(fields) => {
            let mut inner = String::new();
            for field in fields {
                inner.push_str(&format!(
                    "{field}: ::reliakit_codec::CanonicalDecode::decode(__reader)?,",
                ));
            }
            format!("Self {{ {inner} }}")
        }
        Shape::Tuple(count) => {
            let mut inner = String::new();
            for _ in 0..*count {
                inner.push_str("::reliakit_codec::CanonicalDecode::decode(__reader)?,");
            }
            format!("Self({inner})")
        }
        Shape::Unit => "Self".to_string(),
    };
    format!("::core::result::Result::Ok({construct})")
}

/// Encode statements for an enum body: `match self { .. }`, where each arm
/// writes the variant's `u32` declaration-index tag, then its fields in order.
fn enum_encode_statements(variants: &[Variant]) -> String {
    let mut arms = String::new();
    for (index, variant) in variants.iter().enumerate() {
        let tag = index as u32;
        let name = &variant.name;
        let tag_encode =
            format!("::reliakit_codec::CanonicalEncode::encode(&{tag}u32, __writer)?;");
        match &variant.shape {
            Shape::Unit => {
                arms.push_str(&format!("Self::{name} => {{ {tag_encode} }},"));
            }
            Shape::Tuple(count) => {
                let mut pattern = String::new();
                let mut encodes = String::new();
                for i in 0..*count {
                    if i > 0 {
                        pattern.push_str(", ");
                    }
                    pattern.push_str(&format!("__f{i}"));
                    encodes.push_str(&format!(
                        "::reliakit_codec::CanonicalEncode::encode(__f{i}, __writer)?;",
                    ));
                }
                arms.push_str(&format!(
                    "Self::{name}({pattern}) => {{ {tag_encode} {encodes} }},"
                ));
            }
            Shape::Named(fields) => {
                let mut pattern = String::new();
                let mut encodes = String::new();
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        pattern.push_str(", ");
                    }
                    // Bind each named field to a positional local to avoid any
                    // collision with `__writer`.
                    pattern.push_str(&format!("{field}: __f{i}"));
                    encodes.push_str(&format!(
                        "::reliakit_codec::CanonicalEncode::encode(__f{i}, __writer)?;",
                    ));
                }
                arms.push_str(&format!(
                    "Self::{name} {{ {pattern} }} => {{ {tag_encode} {encodes} }},"
                ));
            }
        }
    }
    format!("match self {{ {arms} }}")
}

/// The decode body for an enum: read the `u32` tag, then build the matching
/// variant. An unknown tag is an `invalid_value` codec error.
fn enum_decode_value(name: &str, variants: &[Variant]) -> String {
    let mut arms = String::new();
    for (index, variant) in variants.iter().enumerate() {
        let tag = index as u32;
        let vname = &variant.name;
        let construct = match &variant.shape {
            Shape::Unit => format!("Self::{vname}"),
            Shape::Tuple(count) => {
                let mut inner = String::new();
                for _ in 0..*count {
                    inner.push_str("::reliakit_codec::CanonicalDecode::decode(__reader)?,");
                }
                format!("Self::{vname}({inner})")
            }
            Shape::Named(fields) => {
                let mut inner = String::new();
                for field in fields {
                    inner.push_str(&format!(
                        "{field}: ::reliakit_codec::CanonicalDecode::decode(__reader)?,",
                    ));
                }
                format!("Self::{vname} {{ {inner} }}")
            }
        };
        arms.push_str(&format!("{tag}u32 => {construct},"));
    }

    let message = format!("reliakit-derive: unknown variant tag for {name}");
    format!(
        "let __tag: u32 = ::reliakit_codec::CanonicalDecode::decode(__reader)?;\n\
         ::core::result::Result::Ok(match __tag {{\n\
         {arms}\n\
         _ => return ::core::result::Result::Err(\
         ::reliakit_codec::CodecError::invalid_value({message:?})),\n\
         }})"
    )
}

/// Reads enum variants into [`RawVariant`]s without validating them.
fn raw_variants(stream: TokenStream) -> Vec<RawVariant> {
    let mut variants = Vec::new();
    for segment in top_level_segments(stream) {
        if segment.is_empty() {
            // A trailing comma produces an empty final segment.
            continue;
        }

        // The variant name is the first identifier in the segment (any leading
        // outer attributes are non-ident tokens and are skipped).
        let name_idx = match segment
            .iter()
            .position(|t| matches!(t, TokenTree::Ident(_)))
        {
            Some(i) => i,
            None => {
                variants.push(RawVariant {
                    name: String::new(),
                    shape: Err("reliakit-derive: expected an enum variant name".into()),
                    has_discriminant: false,
                });
                continue;
            }
        };
        let name = match &segment[name_idx] {
            TokenTree::Ident(ident) => ident.to_string(),
            _ => unreachable!("position matched an ident"),
        };

        let mut has_discriminant = false;
        let shape = match segment.get(name_idx + 1) {
            None => Ok(Shape::Unit),
            Some(TokenTree::Group(group)) => match group.delimiter() {
                Delimiter::Parenthesis => Ok(Shape::Tuple(count_fields(group.stream()))),
                Delimiter::Brace => Ok(Shape::Named(named_fields(group.stream()))),
                _ => Err(format!(
                    "reliakit-derive: unsupported syntax in enum variant `{name}`"
                )),
            },
            // An explicit discriminant: record it; validation rejects it. The
            // placeholder shape is never used.
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                has_discriminant = true;
                Ok(Shape::Unit)
            }
            Some(_) => Err(format!(
                "reliakit-derive: unsupported syntax in enum variant `{name}`"
            )),
        };

        variants.push(RawVariant {
            name,
            shape,
            has_discriminant,
        });
    }
    variants
}

/// Returns `true` if an outer-attribute body `[ ... ]` is a `repr` attribute.
fn attr_is_repr(stream: TokenStream) -> bool {
    matches!(stream.into_iter().next(), Some(TokenTree::Ident(ident)) if ident.to_string() == "repr")
}

/// Collects the names of named fields in declaration order.
fn named_fields(stream: TokenStream) -> Vec<String> {
    let mut fields = Vec::new();
    for segment in top_level_segments(stream) {
        // The field name is the first ident immediately followed by a `:`
        // (the field/type separator, which is an `Alone`-spaced colon).
        for window in segment.windows(2) {
            if let (TokenTree::Ident(ident), TokenTree::Punct(punct)) = (&window[0], &window[1]) {
                if punct.as_char() == ':' && punct.spacing() == Spacing::Alone {
                    fields.push(ident.to_string());
                    break;
                }
            }
        }
    }
    fields
}

/// Counts the fields of a tuple body (non-empty top-level segments).
fn count_fields(stream: TokenStream) -> usize {
    top_level_segments(stream)
        .into_iter()
        .filter(|segment| !segment.is_empty())
        .count()
}

/// Splits a token stream on top-level commas, dropping the commas.
fn top_level_segments(stream: TokenStream) -> Vec<Vec<TokenTree>> {
    let mut segments = Vec::new();
    let mut current = Vec::new();
    for token in stream {
        match &token {
            TokenTree::Punct(punct) if punct.as_char() == ',' => {
                segments.push(core::mem::take(&mut current));
            }
            _ => current.push(token),
        }
    }
    if !current.is_empty() {
        segments.push(current);
    }
    segments
}

/// Builds a `compile_error!` invocation carrying `message`.
fn compile_error(message: &str) -> TokenStream {
    format!("::core::compile_error!({message:?});")
        .parse()
        .expect("compile_error message produced invalid tokens")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enum_raw(variants: Vec<RawVariant>, saw_repr: bool, has_generics: bool) -> Raw {
        Raw {
            name: "E".to_string(),
            has_generics,
            saw_repr,
            body: RawBody::Enum(variants),
        }
    }

    fn unit_variant(name: &str) -> RawVariant {
        RawVariant {
            name: name.to_string(),
            shape: Ok(Shape::Unit),
            has_discriminant: false,
        }
    }

    // `Parsed` deliberately has no `Debug`, so these avoid `unwrap`/`unwrap_err`.
    fn err_of(raw: Raw) -> String {
        match validate(raw) {
            Err(message) => message,
            Ok(_) => panic!("expected validation to reject the item"),
        }
    }

    fn ok_of(raw: Raw) -> Parsed {
        match validate(raw) {
            Ok(parsed) => parsed,
            Err(message) => panic!("unexpected validation error: {message}"),
        }
    }

    #[test]
    fn rejects_union() {
        let raw = Raw {
            name: "U".to_string(),
            has_generics: false,
            saw_repr: false,
            body: RawBody::Union,
        };
        assert!(err_of(raw).contains("does not support unions"));
    }

    #[test]
    fn rejects_generic_struct() {
        let raw = Raw {
            name: "S".to_string(),
            has_generics: true,
            saw_repr: false,
            body: RawBody::Struct(Shape::Unit),
        };
        assert!(err_of(raw).contains("does not support generic types yet"));
    }

    #[test]
    fn rejects_generic_enum() {
        let raw = enum_raw(vec![unit_variant("A")], false, true);
        assert!(err_of(raw).contains("does not support generic types yet"));
    }

    #[test]
    fn rejects_repr_enum() {
        let raw = enum_raw(vec![unit_variant("A")], true, false);
        assert!(err_of(raw).contains("does not support `#[repr(...)]` on enums"));
    }

    #[test]
    fn rejects_explicit_discriminant() {
        let raw = enum_raw(
            vec![RawVariant {
                name: "A".to_string(),
                shape: Ok(Shape::Unit),
                has_discriminant: true,
            }],
            false,
            false,
        );
        let err = err_of(raw);
        assert!(err.contains("does not support explicit enum discriminants"));
        assert!(err.contains("`A = ...`"));
    }

    #[test]
    fn rejects_empty_enum() {
        let raw = enum_raw(vec![], false, false);
        assert!(err_of(raw).contains("cannot derive for an empty enum"));
    }

    #[test]
    fn rejects_unsupported_variant_syntax() {
        let raw = enum_raw(
            vec![RawVariant {
                name: "A".to_string(),
                shape: Err("reliakit-derive: unsupported syntax in enum variant `A`".to_string()),
                has_discriminant: false,
            }],
            false,
            false,
        );
        assert!(err_of(raw).contains("unsupported syntax"));
    }

    #[test]
    fn accepts_struct() {
        let raw = Raw {
            name: "S".to_string(),
            has_generics: false,
            saw_repr: false,
            body: RawBody::Struct(Shape::Named(vec!["x".to_string()])),
        };
        let parsed = ok_of(raw);
        assert_eq!(parsed.name, "S");
        assert!(matches!(parsed.body, Body::Struct(Shape::Named(_))));
    }

    #[test]
    fn accepts_enum_preserving_variant_order() {
        let raw = enum_raw(
            vec![
                unit_variant("A"),
                RawVariant {
                    name: "B".to_string(),
                    shape: Ok(Shape::Tuple(1)),
                    has_discriminant: false,
                },
                RawVariant {
                    name: "C".to_string(),
                    shape: Ok(Shape::Named(vec!["id".to_string()])),
                    has_discriminant: false,
                },
            ],
            false,
            false,
        );
        match ok_of(raw).body {
            Body::Enum(variants) => {
                let names: Vec<&str> = variants.iter().map(|v| v.name.as_str()).collect();
                assert_eq!(names, ["A", "B", "C"]);
            }
            Body::Struct(_) => panic!("expected an enum body"),
        }
    }
}

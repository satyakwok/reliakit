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
//! `#[repr(...)]`, and empty enums are rejected with a compile error.
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

/// Which item keyword the derive input started with.
enum Keyword {
    Struct,
    Enum,
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

/// One enum variant: its name and field shape.
struct Variant {
    name: String,
    shape: Shape,
}

/// The body the derive will implement.
enum Body {
    /// A struct with the given field shape.
    Struct(Shape),
    /// An enum with the given variants, in declaration order.
    Enum(Vec<Variant>),
}

struct Parsed {
    name: String,
    body: Body,
}

impl Parsed {
    /// Reads the type name and body shape from a derive input, rejecting
    /// anything outside the supported subset with a descriptive message.
    fn from_input(input: TokenStream) -> Result<Self, String> {
        let tokens: Vec<TokenTree> = input.into_iter().collect();

        // Find the item keyword, skipping outer attributes and visibility.
        // Note whether a `#[repr(...)]` attribute is present so enums can reject
        // it; struct behavior is unchanged (repr is simply ignored there).
        let mut idx = 0;
        let mut saw_repr = false;
        let keyword = loop {
            match tokens.get(idx) {
                Some(TokenTree::Ident(ident)) => match ident.to_string().as_str() {
                    "struct" => break Keyword::Struct,
                    "enum" => break Keyword::Enum,
                    "union" => return Err("reliakit-derive does not support unions".into()),
                    // Visibility (`pub`) or anything else before the keyword.
                    _ => idx += 1,
                },
                // An outer attribute body `[ ... ]` (the leading `#` is a Punct
                // handled by the catch-all below).
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                    if attr_is_repr(group.stream()) {
                        saw_repr = true;
                    }
                    idx += 1;
                }
                Some(_) => idx += 1,
                None => return Err("reliakit-derive: expected a struct or enum".into()),
            }
        };

        // The type name follows the keyword.
        idx += 1;
        let name = match tokens.get(idx) {
            Some(TokenTree::Ident(ident)) => ident.to_string(),
            _ => return Err("reliakit-derive: expected a type name after the item keyword".into()),
        };
        idx += 1;

        // Reject generics rather than mis-parsing them.
        if let Some(TokenTree::Punct(punct)) = tokens.get(idx) {
            if punct.as_char() == '<' {
                return Err("reliakit-derive does not support generic types yet".into());
            }
        }

        // The body is a group (struct/enum) or `;` (unit struct).
        match tokens.get(idx) {
            Some(TokenTree::Group(group)) => match keyword {
                Keyword::Struct => {
                    let shape = match group.delimiter() {
                        Delimiter::Brace => Shape::Named(named_fields(group.stream())),
                        Delimiter::Parenthesis => Shape::Tuple(count_fields(group.stream())),
                        _ => return Err("reliakit-derive: unexpected struct body".into()),
                    };
                    Ok(Self {
                        name,
                        body: Body::Struct(shape),
                    })
                }
                Keyword::Enum => {
                    if saw_repr {
                        return Err("reliakit-derive does not support `#[repr(...)]` on enums; \
                                    variant tags are always the u32 declaration index"
                            .into());
                    }
                    if group.delimiter() != Delimiter::Brace {
                        return Err("reliakit-derive: expected a braced enum body".into());
                    }
                    let variants = parse_variants(group.stream())?;
                    Ok(Self {
                        name,
                        body: Body::Enum(variants),
                    })
                }
            },
            Some(TokenTree::Punct(punct))
                if punct.as_char() == ';' && matches!(keyword, Keyword::Struct) =>
            {
                Ok(Self {
                    name,
                    body: Body::Struct(Shape::Unit),
                })
            }
            _ => Err(match keyword {
                Keyword::Struct => "reliakit-derive: unexpected struct body".into(),
                Keyword::Enum => "reliakit-derive: expected a braced enum body".into(),
            }),
        }
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

/// Parses enum variants in declaration order, rejecting explicit discriminants
/// and unsupported variant syntax.
fn parse_variants(stream: TokenStream) -> Result<Vec<Variant>, String> {
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
            None => return Err("reliakit-derive: expected an enum variant name".into()),
        };
        let name = match &segment[name_idx] {
            TokenTree::Ident(ident) => ident.to_string(),
            _ => unreachable!("position matched an ident"),
        };

        let shape = match segment.get(name_idx + 1) {
            None => Shape::Unit,
            Some(TokenTree::Group(group)) => match group.delimiter() {
                Delimiter::Parenthesis => Shape::Tuple(count_fields(group.stream())),
                Delimiter::Brace => Shape::Named(named_fields(group.stream())),
                _ => {
                    return Err(format!(
                        "reliakit-derive: unsupported syntax in enum variant `{name}`"
                    ))
                }
            },
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => {
                return Err(format!(
                    "reliakit-derive does not support explicit enum discriminants \
                     (`{name} = ...`); variant tags are the u32 declaration index"
                ));
            }
            Some(_) => {
                return Err(format!(
                    "reliakit-derive: unsupported syntax in enum variant `{name}`"
                ))
            }
        };

        variants.push(Variant { name, shape });
    }

    if variants.is_empty() {
        return Err("reliakit-derive cannot derive for an empty enum \
                    (there is no variant to encode or decode)"
            .into());
    }
    Ok(variants)
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

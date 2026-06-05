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
//!
//! Enums, unions, and generic types are rejected for now with a compile error.
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

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use proc_macro::{Delimiter, Spacing, TokenStream, TokenTree};

/// Derives `reliakit_codec::CanonicalEncode`, encoding each field in
/// declaration order.
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
/// declaration order.
///
/// See the [crate] documentation for supported types and limitations.
#[proc_macro_derive(CanonicalDecode)]
pub fn derive_canonical_decode(input: TokenStream) -> TokenStream {
    match Parsed::from_input(input) {
        Ok(parsed) => parsed.canonical_decode_impl(),
        Err(message) => compile_error(&message),
    }
}

/// The shape of a struct the derive can implement, reduced to exactly what the
/// generated code needs.
enum Shape {
    /// Named fields, in declaration order.
    Named(Vec<String>),
    /// Tuple fields, by count.
    Tuple(usize),
    /// Unit struct, no fields.
    Unit,
}

struct Parsed {
    name: String,
    shape: Shape,
}

impl Parsed {
    /// Reads the type name and field shape from a derive input, rejecting
    /// anything outside the supported subset with a descriptive message.
    fn from_input(input: TokenStream) -> Result<Self, String> {
        let tokens: Vec<TokenTree> = input.into_iter().collect();

        // Find the item keyword, skipping outer attributes and visibility.
        let mut idx = 0;
        loop {
            match tokens.get(idx) {
                Some(TokenTree::Ident(ident)) => {
                    let kw = ident.to_string();
                    match kw.as_str() {
                        "struct" => break,
                        "enum" => {
                            return Err("reliakit-derive does not support enums yet".into());
                        }
                        "union" => {
                            return Err("reliakit-derive does not support unions".into());
                        }
                        // Visibility (`pub`) or anything before `struct`: skip.
                        _ => idx += 1,
                    }
                }
                // Attribute `#[..]`, visibility group `pub(..)`, etc.: skip.
                Some(_) => idx += 1,
                None => return Err("reliakit-derive: expected a struct".into()),
            }
        }

        // The type name follows the `struct` keyword.
        idx += 1;
        let name = match tokens.get(idx) {
            Some(TokenTree::Ident(ident)) => ident.to_string(),
            _ => return Err("reliakit-derive: expected a type name after `struct`".into()),
        };
        idx += 1;

        // Reject generics rather than mis-parsing them.
        if let Some(TokenTree::Punct(punct)) = tokens.get(idx) {
            if punct.as_char() == '<' {
                return Err("reliakit-derive does not support generic types yet".into());
            }
        }

        // The body is a brace group (named), paren group (tuple), or `;` (unit).
        let shape = match tokens.get(idx) {
            Some(TokenTree::Group(group)) => match group.delimiter() {
                Delimiter::Brace => Shape::Named(named_fields(group.stream())),
                Delimiter::Parenthesis => Shape::Tuple(count_fields(group.stream())),
                _ => return Err("reliakit-derive: unexpected struct body".into()),
            },
            Some(TokenTree::Punct(punct)) if punct.as_char() == ';' => Shape::Unit,
            _ => return Err("reliakit-derive: unexpected struct body".into()),
        };

        Ok(Self { name, shape })
    }

    fn canonical_encode_impl(&self) -> TokenStream {
        let mut body = String::new();
        match &self.shape {
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

        format!(
            "impl ::reliakit_codec::CanonicalEncode for {name} {{\n\
             fn encode<__W: ::reliakit_codec::EncodeSink + ?Sized>(&self, __writer: &mut __W) \
             -> ::core::result::Result<(), ::reliakit_codec::CodecError> {{\n\
             {body}\n\
             ::core::result::Result::Ok(())\n\
             }}\n\
             }}",
            name = self.name,
        )
        .parse()
        .expect("reliakit-derive generated invalid CanonicalEncode tokens")
    }

    fn canonical_decode_impl(&self) -> TokenStream {
        let construct = match &self.shape {
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

        format!(
            "impl ::reliakit_codec::CanonicalDecode for {name} {{\n\
             fn decode<__R: ::reliakit_codec::DecodeSource + ?Sized>(__reader: &mut __R) \
             -> ::core::result::Result<Self, ::reliakit_codec::CodecError> {{\n\
             ::core::result::Result::Ok({construct})\n\
             }}\n\
             }}",
            name = self.name,
        )
        .parse()
        .expect("reliakit-derive generated invalid CanonicalDecode tokens")
    }
}

/// Collects the names of named struct fields in declaration order.
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

/// Counts the fields of a tuple struct (non-empty top-level segments).
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

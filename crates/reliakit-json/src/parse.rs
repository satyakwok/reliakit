//! Strict, bounded JSON parser.

use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::error::{JsonError, JsonErrorKind, JsonLimitKind, JsonPath, JsonPathSegment};
use crate::limits::JsonLimits;
use crate::number::{JsonNumber, is_valid_json_number};
use crate::value::{JsonObject, JsonValue};

/// Parses a JSON value from UTF-8 bytes using the default
/// [`JsonLimits`].
pub fn parse(input: &[u8]) -> Result<JsonValue, JsonError> {
    parse_with_limits(input, JsonLimits::new())
}

/// Parses a JSON value from a `&str` using the default [`JsonLimits`].
pub fn parse_str(input: &str) -> Result<JsonValue, JsonError> {
    parse_with_limits(input.as_bytes(), JsonLimits::new())
}

/// Parses a JSON value from UTF-8 bytes with explicit [`JsonLimits`].
pub fn parse_with_limits(input: &[u8], limits: JsonLimits) -> Result<JsonValue, JsonError> {
    if input.len() > limits.max_input_bytes() {
        return Err(JsonError::new(
            JsonErrorKind::LimitExceeded(JsonLimitKind::InputBytes),
            0,
            1,
            1,
        )
        .with_path(JsonPath::default()));
    }

    if let Err(e) = core::str::from_utf8(input) {
        let offset = e.valid_up_to();
        let (line, column) = line_column(input, offset);
        return Err(
            JsonError::new(JsonErrorKind::InvalidUtf8, offset, line, column)
                .with_path(JsonPath::default()),
        );
    }

    // Reject a leading UTF-8 byte-order mark (valid UTF-8, but not valid JSON).
    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Err(
            JsonError::new(JsonErrorKind::InvalidUtf8, 0, 1, 1).with_path(JsonPath::default())
        );
    }

    let mut parser = Parser::new(input, limits);
    parser.skip_ws();
    let value = parser.parse_value(0)?;
    parser.skip_ws();
    if parser.pos != parser.input.len() {
        return Err(parser.error(JsonErrorKind::TrailingData));
    }
    Ok(value)
}

fn line_column(input: &[u8], offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;
    for &b in &input[..offset.min(input.len())] {
        if b == b'\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
    limits: JsonLimits,
    nodes: usize,
    decoded_string_bytes: usize,
    path: Vec<JsonPathSegment>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a [u8], limits: JsonLimits) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            column: 1,
            limits,
            nodes: 0,
            decoded_string_bytes: 0,
            path: Vec::new(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn bump(&mut self) -> u8 {
        let b = self.input[self.pos];
        self.pos += 1;
        if b == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        b
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn error(&self, kind: JsonErrorKind) -> JsonError {
        JsonError::new(kind, self.pos, self.line, self.column)
            .with_path(JsonPath::from_segments(self.path.clone()))
    }

    fn error_at(&self, kind: JsonErrorKind, pos: usize, line: usize, column: usize) -> JsonError {
        JsonError::new(kind, pos, line, column)
            .with_path(JsonPath::from_segments(self.path.clone()))
    }

    fn limit(&self, kind: JsonLimitKind) -> JsonError {
        self.error(JsonErrorKind::LimitExceeded(kind))
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.nodes += 1;
        if self.nodes > self.limits.max_total_nodes() {
            return Err(self.limit(JsonLimitKind::TotalNodes));
        }
        match self.peek() {
            Some(b'{') => {
                let d = depth + 1;
                if d > self.limits.max_depth() {
                    return Err(self.limit(JsonLimitKind::Depth));
                }
                self.parse_object(d)
            }
            Some(b'[') => {
                let d = depth + 1;
                if d > self.limits.max_depth() {
                    return Err(self.limit(JsonLimitKind::Depth));
                }
                self.parse_array(d)
            }
            Some(b'"') => {
                let s =
                    self.parse_string(self.limits.max_string_bytes(), JsonLimitKind::StringBytes)?;
                Ok(JsonValue::String(s))
            }
            Some(b't') => self.parse_literal(b"true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", JsonValue::Bool(false)),
            Some(b'n') => self.parse_literal(b"null", JsonValue::Null),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(self.error(JsonErrorKind::UnexpectedByte)),
            None => Err(self.error(JsonErrorKind::UnexpectedEof)),
        }
    }

    fn parse_literal(&mut self, word: &[u8], value: JsonValue) -> Result<JsonValue, JsonError> {
        for &expected in word {
            match self.peek() {
                Some(b) if b == expected => {
                    self.bump();
                }
                Some(_) => return Err(self.error(JsonErrorKind::UnexpectedByte)),
                None => return Err(self.error(JsonErrorKind::UnexpectedEof)),
            }
        }
        Ok(value)
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.bump(); // consume '{'
        let mut object = JsonObject::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();

        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.bump();
            return Ok(JsonValue::Object(object));
        }

        loop {
            self.skip_ws();
            // A key must come next.
            if self.peek() != Some(b'"') {
                return match self.peek() {
                    None => Err(self.error(JsonErrorKind::UnexpectedEof)),
                    _ => Err(self.error(JsonErrorKind::UnexpectedByte)),
                };
            }

            let key_pos = self.pos;
            let key_line = self.line;
            let key_column = self.column;
            let key = self.parse_string(self.limits.max_key_bytes(), JsonLimitKind::KeyBytes)?;

            if !seen.insert(key.clone()) {
                return Err(self.error_at(
                    JsonErrorKind::DuplicateKey,
                    key_pos,
                    key_line,
                    key_column,
                ));
            }
            if seen.len() > self.limits.max_object_members() {
                return Err(self.limit(JsonLimitKind::ObjectMembers));
            }

            self.skip_ws();
            if self.peek() != Some(b':') {
                return match self.peek() {
                    None => Err(self.error(JsonErrorKind::UnexpectedEof)),
                    _ => Err(self.error(JsonErrorKind::UnexpectedByte)),
                };
            }
            self.bump(); // ':'
            self.skip_ws();

            self.path.push(JsonPathSegment::Key(key.clone()));
            let value = self.parse_value(depth)?;
            self.path.pop();
            object.push_unique(key, value);

            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.bump();
                }
                Some(b'}') => {
                    self.bump();
                    return Ok(JsonValue::Object(object));
                }
                None => return Err(self.error(JsonErrorKind::UnexpectedEof)),
                _ => return Err(self.error(JsonErrorKind::UnexpectedByte)),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.bump(); // consume '['
        let mut items: Vec<JsonValue> = Vec::new();

        self.skip_ws();
        if self.peek() == Some(b']') {
            self.bump();
            return Ok(JsonValue::Array(items));
        }

        loop {
            self.skip_ws();
            if items.len() >= self.limits.max_array_items() {
                return Err(self.limit(JsonLimitKind::ArrayItems));
            }

            self.path.push(JsonPathSegment::Index(items.len()));
            let value = self.parse_value(depth)?;
            self.path.pop();
            items.push(value);

            self.skip_ws();
            match self.peek() {
                Some(b',') => {
                    self.bump();
                }
                Some(b']') => {
                    self.bump();
                    return Ok(JsonValue::Array(items));
                }
                None => return Err(self.error(JsonErrorKind::UnexpectedEof)),
                _ => return Err(self.error(JsonErrorKind::UnexpectedByte)),
            }
        }
    }

    fn parse_string(
        &mut self,
        max_bytes: usize,
        limit_kind: JsonLimitKind,
    ) -> Result<String, JsonError> {
        self.bump(); // consume opening '"'
        let mut out = String::new();

        loop {
            match self.peek() {
                None => return Err(self.error(JsonErrorKind::UnexpectedEof)),
                Some(b'"') => {
                    self.bump();
                    self.decoded_string_bytes = self.decoded_string_bytes.saturating_add(out.len());
                    if self.decoded_string_bytes > self.limits.max_total_decoded_string_bytes() {
                        return Err(self.limit(JsonLimitKind::TotalDecodedStringBytes));
                    }
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.bump();
                    self.parse_escape(&mut out)?;
                }
                Some(b) if b < 0x20 => {
                    return Err(self.error(JsonErrorKind::UnescapedControlCharacter));
                }
                Some(b) => {
                    let len = utf8_len(b);
                    // Input is validated UTF-8, so this slice is a whole scalar.
                    let scalar = &self.input[self.pos..self.pos + len];
                    out.push_str(core::str::from_utf8(scalar).expect("validated UTF-8"));
                    for _ in 0..len {
                        self.bump();
                    }
                }
            }

            if out.len() > max_bytes {
                return Err(self.limit(limit_kind));
            }
        }
    }

    fn parse_escape(&mut self, out: &mut String) -> Result<(), JsonError> {
        match self.peek() {
            None => Err(self.error(JsonErrorKind::UnexpectedEof)),
            Some(b'"') => {
                out.push('"');
                self.bump();
                Ok(())
            }
            Some(b'\\') => {
                out.push('\\');
                self.bump();
                Ok(())
            }
            Some(b'/') => {
                out.push('/');
                self.bump();
                Ok(())
            }
            Some(b'b') => {
                out.push('\u{08}');
                self.bump();
                Ok(())
            }
            Some(b'f') => {
                out.push('\u{0C}');
                self.bump();
                Ok(())
            }
            Some(b'n') => {
                out.push('\n');
                self.bump();
                Ok(())
            }
            Some(b'r') => {
                out.push('\r');
                self.bump();
                Ok(())
            }
            Some(b't') => {
                out.push('\t');
                self.bump();
                Ok(())
            }
            Some(b'u') => {
                self.bump();
                let hi = self.parse_hex4()?;
                if (0xD800..=0xDBFF).contains(&hi) {
                    // Expect a following low-surrogate escape.
                    if self.peek() != Some(b'\\') {
                        return Err(self.error(JsonErrorKind::LoneSurrogate));
                    }
                    self.bump();
                    if self.peek() != Some(b'u') {
                        return Err(self.error(JsonErrorKind::LoneSurrogate));
                    }
                    self.bump();
                    let lo = self.parse_hex4()?;
                    if !(0xDC00..=0xDFFF).contains(&lo) {
                        return Err(self.error(JsonErrorKind::LoneSurrogate));
                    }
                    let scalar = 0x10000 + (((hi as u32) - 0xD800) << 10) + ((lo as u32) - 0xDC00);
                    out.push(char::from_u32(scalar).expect("valid scalar from surrogate pair"));
                    Ok(())
                } else if (0xDC00..=0xDFFF).contains(&hi) {
                    Err(self.error(JsonErrorKind::LoneSurrogate))
                } else {
                    out.push(char::from_u32(hi as u32).expect("non-surrogate is a valid scalar"));
                    Ok(())
                }
            }
            Some(_) => Err(self.error(JsonErrorKind::InvalidEscape)),
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, JsonError> {
        let mut value: u16 = 0;
        for _ in 0..4 {
            match self.peek() {
                None => return Err(self.error(JsonErrorKind::UnexpectedEof)),
                Some(b) => match hex_value(b) {
                    Some(digit) => {
                        value = (value << 4) | digit;
                        self.bump();
                    }
                    None => return Err(self.error(JsonErrorKind::InvalidUnicodeEscape)),
                },
            }
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.pos;
        let start_line = self.line;
        let start_column = self.column;
        while let Some(b) = self.peek() {
            if matches!(b, b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                self.bump();
            } else {
                break;
            }
        }
        let token = &self.input[start..self.pos];
        if token.len() > self.limits.max_number_bytes() {
            return Err(self.error_at(
                JsonErrorKind::LimitExceeded(JsonLimitKind::NumberBytes),
                start,
                start_line,
                start_column,
            ));
        }
        let text = core::str::from_utf8(token).expect("validated UTF-8");
        if !is_valid_json_number(text) {
            return Err(self.error_at(
                JsonErrorKind::InvalidNumber,
                start,
                start_line,
                start_column,
            ));
        }
        Ok(JsonValue::Number(JsonNumber::from_validated(
            text.to_string(),
        )))
    }
}

fn utf8_len(lead: u8) -> usize {
    if lead < 0x80 {
        1
    } else if lead < 0xE0 {
        2
    } else if lead < 0xF0 {
        3
    } else {
        4
    }
}

fn hex_value(b: u8) -> Option<u16> {
    match b {
        b'0'..=b'9' => Some((b - b'0') as u16),
        b'a'..=b'f' => Some((b - b'a' + 10) as u16),
        b'A'..=b'F' => Some((b - b'A' + 10) as u16),
        _ => None,
    }
}

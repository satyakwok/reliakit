//! The owned JSON value model.

use alloc::string::String;
use alloc::vec::Vec;

use crate::number::JsonNumber;

/// An owned JSON value.
///
/// JSON has exactly six value kinds, so this enum is intentionally exhaustive —
/// you can `match` it without a wildcard arm.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, preserving its exact source text.
    Number(JsonNumber),
    /// A string (decoded Unicode scalar values).
    String(String),
    /// An array.
    Array(Vec<JsonValue>),
    /// An object with unique keys in insertion order.
    Object(JsonObject),
}

impl JsonValue {
    /// Returns `true` if this is [`JsonValue::Null`].
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Returns the boolean if this is a [`JsonValue::Bool`].
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the number if this is a [`JsonValue::Number`].
    pub fn as_number(&self) -> Option<&JsonNumber> {
        match self {
            JsonValue::Number(n) => Some(n),
            _ => None,
        }
    }

    /// Returns the string if this is a [`JsonValue::String`].
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the array if this is a [`JsonValue::Array`].
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            JsonValue::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Returns the object if this is a [`JsonValue::Object`].
    pub fn as_object(&self) -> Option<&JsonObject> {
        match self {
            JsonValue::Object(o) => Some(o),
            _ => None,
        }
    }
}

/// A single object member (key/value pair).
#[derive(Debug, Clone, PartialEq)]
pub struct JsonMember {
    key: String,
    value: JsonValue,
}

impl JsonMember {
    /// The member key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The member value.
    pub fn value(&self) -> &JsonValue {
        &self.value
    }
}

/// A JSON object: members in insertion order with unique keys.
///
/// Keys are guaranteed unique — the parser rejects duplicates, and
/// [`insert`](Self::insert) replaces an existing key in place rather than
/// adding a second entry. Lookup is linear; object size is bounded by
/// [`JsonLimits`](crate::JsonLimits) when parsing untrusted input.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct JsonObject {
    entries: Vec<JsonMember>,
}

impl JsonObject {
    /// Creates an empty object.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub(crate) fn push_unique(&mut self, key: String, value: JsonValue) {
        self.entries.push(JsonMember { key, value });
    }

    /// Returns the value for `key`, if present.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.entries.iter().find(|m| m.key == key).map(|m| &m.value)
    }

    /// Returns `true` if `key` is present.
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.iter().any(|m| m.key == key)
    }

    /// Inserts or replaces `key`. If the key already exists its value is
    /// replaced in place (preserving position) and the old value is returned;
    /// otherwise the member is appended.
    pub fn insert(&mut self, key: String, value: JsonValue) -> Option<JsonValue> {
        if let Some(member) = self.entries.iter_mut().find(|m| m.key == key) {
            Some(core::mem::replace(&mut member.value, value))
        } else {
            self.entries.push(JsonMember { key, value });
            None
        }
    }

    /// The number of members.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the object has no members.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterates over members in insertion order.
    pub fn iter(&self) -> core::slice::Iter<'_, JsonMember> {
        self.entries.iter()
    }
}

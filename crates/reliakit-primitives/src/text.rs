use crate::{PrimitiveError, PrimitiveResult};
use alloc::string::String;
use core::{fmt, ops::Deref, str::FromStr};

// ── Slug ─────────────────────────────────────────────────────────────────────

/// URL-safe slug: lowercase ASCII alphanumeric characters and hyphens.
///
/// Rules: non-empty, only `[a-z0-9-]`, does not start or end with `-`,
/// no consecutive `--`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Slug(String);

impl Slug {
    /// Creates a new `Slug`. Returns `Invalid` if the value violates slug rules.
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        if !is_valid_slug(&value) {
            return Err(PrimitiveError::Invalid {
                message: "slug must be lowercase alphanumeric with hyphens, must not start or end with a hyphen, and must not contain consecutive hyphens",
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying slug string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

fn is_valid_slug(s: &str) -> bool {
    if s.starts_with('-') || s.ends_with('-') {
        return false;
    }
    let mut prev = ' ';
    for c in s.chars() {
        if !matches!(c, 'a'..='z' | '0'..='9' | '-') {
            return false;
        }
        if c == '-' && prev == '-' {
            return false;
        }
        prev = c;
    }
    true
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Slug {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for Slug {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Slug {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Slug {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for Slug {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Slug {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Slug {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for Slug {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<Slug> for String {
    fn from(value: Slug) -> Self {
        value.into_inner()
    }
}

// ── Email ─────────────────────────────────────────────────────────────────────

/// Email address with basic structural validation.
///
/// Checks: exactly one `@`, non-empty local part and domain, domain contains
/// at least one `.`, domain labels are non-empty, no whitespace. Not a full
/// RFC 5321 validator.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Email(String);

impl Email {
    /// Creates a new `Email`. Returns `Invalid` if the value fails structural checks.
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        if !is_valid_email(&value) {
            return Err(PrimitiveError::Invalid {
                message: "invalid email address",
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying email string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns the local part (before `@`).
    pub fn local(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    /// Returns the domain part (after `@`).
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

fn is_valid_email(s: &str) -> bool {
    if s.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    let at_count = s.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return false;
    }
    let mut parts = s.splitn(2, '@');
    let local = parts.next().unwrap_or("");
    let domain = parts.next().unwrap_or("");
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    if !domain.contains('.') || domain.split('.').any(str::is_empty) {
        return false;
    }
    true
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Email {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for Email {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Email {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Email {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for Email {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Email {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Email {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for Email {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<Email> for String {
    fn from(value: Email) -> Self {
        value.into_inner()
    }
}

// ── HttpUrl ───────────────────────────────────────────────────────────────────

/// HTTP or HTTPS URL with scheme validation.
///
/// Must start with `http://` or `https://` and have a non-empty host.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HttpUrl(String);

impl HttpUrl {
    /// Creates a new `HttpUrl`. Returns `Invalid` if the scheme is missing or
    /// the host is empty.
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        let after_scheme = strip_http_scheme(&value).ok_or(PrimitiveError::Invalid {
            message: "URL must start with http:// or https://",
        })?;
        let host = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
        if host.is_empty() || host.chars().all(|c| c.is_whitespace()) {
            return Err(PrimitiveError::Invalid {
                message: "URL must have a non-empty host",
            });
        }
        if after_scheme.chars().any(|c| c.is_whitespace()) {
            return Err(PrimitiveError::Invalid {
                message: "URL must not contain whitespace",
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying URL string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns `true` if the URL uses `https`.
    pub fn is_https(&self) -> bool {
        self.0.len() >= 8 && self.0[..8].eq_ignore_ascii_case("https://")
    }
}

fn strip_http_scheme(value: &str) -> Option<&str> {
    if value
        .as_bytes()
        .get(..8)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"https://"))
    {
        Some(&value[8..])
    } else if value
        .as_bytes()
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"http://"))
    {
        Some(&value[7..])
    } else {
        None
    }
}

impl fmt::Display for HttpUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for HttpUrl {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for HttpUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for HttpUrl {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for HttpUrl {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for HttpUrl {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for HttpUrl {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for HttpUrl {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for HttpUrl {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for HttpUrl {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<HttpUrl> for String {
    fn from(value: HttpUrl) -> Self {
        value.into_inner()
    }
}

// ── HexString ─────────────────────────────────────────────────────────────────

/// String of valid hexadecimal characters, with optional `0x`/`0X` prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexString(String);

impl HexString {
    /// Creates a new `HexString`. Returns `Invalid` if any character is not a
    /// valid hex digit (after stripping an optional `0x`/`0X` prefix).
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        let hex_part = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
            .unwrap_or(&value);
        if hex_part.is_empty() {
            return Err(PrimitiveError::Invalid {
                message: "hex string must not be empty after prefix",
            });
        }
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(PrimitiveError::Invalid {
                message: "hex string must contain only hexadecimal characters (0-9, a-f, A-F)",
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying hex string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns `true` if the value was stored with a `0x`/`0X` prefix.
    pub fn has_prefix(&self) -> bool {
        self.0.starts_with("0x") || self.0.starts_with("0X")
    }

    /// Returns only the hex digit characters, without any `0x`/`0X` prefix.
    pub fn hex_digits(&self) -> &str {
        self.0
            .strip_prefix("0x")
            .or_else(|| self.0.strip_prefix("0X"))
            .unwrap_or(&self.0)
    }
}

impl fmt::Display for HexString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for HexString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for HexString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for HexString {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for HexString {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for HexString {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for HexString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for HexString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for HexString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for HexString {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<HexString> for String {
    fn from(value: HexString) -> Self {
        value.into_inner()
    }
}

// ── Base64 ────────────────────────────────────────────────────────────────────

/// Standard (RFC 4648) base64 string with required, correct padding.
///
/// Rules: non-empty, length is a multiple of `4`, every non-padding character is
/// in the standard alphabet (`A-Z`, `a-z`, `0-9`, `+`, `/`), and `=` padding (at
/// most two) appears only at the end. This is a *format* check; it does not
/// decode the data. The URL-safe alphabet (`-`/`_`) is not accepted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Base64(String);

impl Base64 {
    /// Creates a new `Base64`. Returns an error if the value is empty or is not
    /// well-formed standard base64 (see the type docs for the exact rules).
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        let bytes = value.as_bytes();
        if bytes.len() % 4 != 0 {
            return Err(PrimitiveError::Invalid {
                message: "base64 length must be a multiple of 4",
            });
        }
        let pad = bytes.iter().rev().take_while(|&&b| b == b'=').count();
        if pad > 2 {
            return Err(PrimitiveError::Invalid {
                message: "base64 has at most two padding characters",
            });
        }
        // Every character before the padding must be in the standard alphabet;
        // because `=` is not in the alphabet, this also rejects interior padding.
        if !bytes[..bytes.len() - pad]
            .iter()
            .all(|&b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/')
        {
            return Err(PrimitiveError::Invalid {
                message: "base64 contains a character outside the standard alphabet",
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying base64 string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns `true` if the value carries `=` padding.
    pub fn is_padded(&self) -> bool {
        self.0.ends_with('=')
    }

    /// Returns the number of bytes this base64 string decodes to.
    pub fn decoded_len(&self) -> usize {
        let pad = self.0.bytes().rev().take_while(|&b| b == b'=').count();
        self.0.len() / 4 * 3 - pad
    }
}

impl fmt::Display for Base64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Base64 {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Base64 {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for Base64 {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Base64 {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Base64 {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for Base64 {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Base64 {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Base64 {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for Base64 {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<Base64> for String {
    fn from(value: Base64) -> Self {
        value.into_inner()
    }
}

// ── Identifier ────────────────────────────────────────────────────────────────

/// A conservative ASCII identifier: a letter or `_`, then letters, digits, or
/// `_`.
///
/// Rules: non-empty, the first character is `[A-Za-z_]`, and every remaining
/// character is `[A-Za-z0-9_]`. Useful for handles, keys, and machine-generated
/// names that must be safe across many systems.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier(String);

impl Identifier {
    /// Creates a new `Identifier`. Returns an error if the value is empty or
    /// contains a character not allowed at its position (see the type docs).
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        let mut chars = value.chars();
        match chars.next() {
            None => return Err(PrimitiveError::Empty),
            Some(first) if !(first.is_ascii_alphabetic() || first == '_') => {
                return Err(PrimitiveError::Invalid {
                    message: "identifier must start with an ASCII letter or underscore",
                });
            }
            Some(_) => {}
        }
        if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(PrimitiveError::Invalid {
                message: "identifier may contain only ASCII letters, digits, and underscores",
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying identifier string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Identifier {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for Identifier {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Identifier {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Identifier {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for Identifier {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Identifier {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Identifier {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for Identifier {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.into_inner()
    }
}

// ── Hostname ──────────────────────────────────────────────────────────────────

/// A DNS hostname following the RFC 1123 rules.
///
/// Rules: non-empty, at most 253 characters total, split into dot-separated
/// labels where each label is 1–63 characters of `[A-Za-z0-9-]` and does not
/// start or end with a hyphen. Empty labels (a leading, trailing, or doubled
/// dot) are rejected. The check is case-preserving and does not resolve the name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hostname(String);

impl Hostname {
    /// Creates a new `Hostname`. Returns an error if the value is empty, too
    /// long, or has a label that violates the RFC 1123 rules (see the type docs).
    pub fn new(value: impl Into<String>) -> PrimitiveResult<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        if value.len() > 253 {
            return Err(PrimitiveError::TooLong {
                max: 253,
                actual: value.len(),
            });
        }
        for label in value.split('.') {
            if label.is_empty() {
                return Err(PrimitiveError::Invalid {
                    message: "hostname label must not be empty",
                });
            }
            if label.len() > 63 {
                return Err(PrimitiveError::Invalid {
                    message: "hostname label must not exceed 63 characters",
                });
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(PrimitiveError::Invalid {
                    message: "hostname label must not start or end with a hyphen",
                });
            }
            if !label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            {
                return Err(PrimitiveError::Invalid {
                    message: "hostname label may contain only letters, digits, and hyphens",
                });
            }
        }
        Ok(Self(value))
    }

    /// Returns the underlying hostname string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper and returns the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Iterates over the dot-separated labels, from left to right.
    pub fn labels(&self) -> impl Iterator<Item = &str> + '_ {
        self.0.split('.')
    }
}

impl fmt::Display for Hostname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Hostname {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Hostname {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl TryFrom<&str> for Hostname {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Hostname {
    type Error = PrimitiveError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for Hostname {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl PartialEq<str> for Hostname {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Hostname {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Hostname {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&String> for Hostname {
    fn eq(&self, other: &&String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<Hostname> for String {
    fn from(value: Hostname) -> Self {
        value.into_inner()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Base64, Email, HexString, Hostname, HttpUrl, Identifier, Slug};
    use crate::{PrimitiveError, PrimitiveErrorKind};

    // Slug
    #[test]
    fn slug_accepts_valid() {
        assert_eq!(Slug::new("my-service").unwrap().as_str(), "my-service");
        assert_eq!(Slug::new("api-v2").unwrap().as_str(), "api-v2");
        assert_eq!(Slug::new("user123").unwrap().as_str(), "user123");
    }

    #[test]
    fn slug_rejects_empty() {
        assert_eq!(Slug::new("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn slug_rejects_uppercase() {
        assert!(Slug::new("MySlug").is_err());
    }

    #[test]
    fn slug_rejects_leading_hyphen() {
        assert!(Slug::new("-bad").is_err());
    }

    #[test]
    fn slug_rejects_trailing_hyphen() {
        assert!(Slug::new("bad-").is_err());
    }

    #[test]
    fn slug_rejects_consecutive_hyphens() {
        assert!(Slug::new("bad--slug").is_err());
    }

    #[test]
    fn slug_rejects_spaces() {
        assert!(Slug::new("has space").is_err());
    }

    #[test]
    fn slug_display() {
        use alloc::string::ToString;
        assert_eq!(Slug::new("hello").unwrap().to_string(), "hello");
    }

    #[test]
    fn slug_deref() {
        let s = Slug::new("hello").unwrap();
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn slug_from_str_and_string_comparisons() {
        let slug = "hello".parse::<Slug>().unwrap();
        let owned = String::from("hello");
        assert_eq!(slug, "hello");
        assert_eq!(slug, owned);
        assert!("Hello".parse::<Slug>().is_err());
    }

    #[test]
    fn slug_converts_into_string() {
        let slug = Slug::new("hello").unwrap();
        let inner = String::from(slug);
        assert_eq!(inner, "hello");
    }

    // Email
    #[test]
    fn email_accepts_valid() {
        let e = Email::new("user@example.com").unwrap();
        assert_eq!(e.local(), "user");
        assert_eq!(e.domain(), "example.com");
    }

    #[test]
    fn email_rejects_empty() {
        assert_eq!(Email::new("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn email_rejects_missing_at() {
        assert!(Email::new("nodomain").is_err());
    }

    #[test]
    fn email_rejects_multiple_at() {
        assert!(Email::new("a@b@c.com").is_err());
    }

    #[test]
    fn email_rejects_no_dot_in_domain() {
        assert!(Email::new("user@nodot").is_err());
    }

    #[test]
    fn email_rejects_empty_domain_labels() {
        assert!(Email::new("user@example..com").is_err());
        assert!(Email::new("user@.example.com").is_err());
        assert!(Email::new("user@example.com.").is_err());
    }

    #[test]
    fn email_rejects_spaces() {
        assert!(Email::new("us er@example.com").is_err());
    }

    #[test]
    fn email_rejects_tab() {
        assert!(Email::new("user\t@example.com").is_err());
    }

    #[test]
    fn email_rejects_newline() {
        assert!(Email::new("user\n@example.com").is_err());
    }

    #[test]
    fn url_rejects_whitespace_host() {
        assert!(HttpUrl::new("http://   ").is_err());
    }

    #[test]
    fn url_rejects_whitespace_in_path() {
        assert!(HttpUrl::new("https://ex ample.com").is_err());
    }

    #[test]
    fn email_display() {
        use alloc::string::ToString;
        assert_eq!(Email::new("a@b.com").unwrap().to_string(), "a@b.com");
    }

    #[test]
    fn email_from_str_and_string_comparisons() {
        let email = "a@b.com".parse::<Email>().unwrap();
        let owned = String::from("a@b.com");
        assert_eq!(email, "a@b.com");
        assert_eq!(email, owned);
        assert!("bad".parse::<Email>().is_err());
    }

    #[test]
    fn email_string_ergonomics() {
        let email = Email::try_from(String::from("a@b.com")).unwrap();
        let borrowed: &str = email.as_ref();
        assert_eq!(borrowed, "a@b.com");
        assert_eq!(&*email, "a@b.com");

        let inner = String::from(email);
        assert_eq!(inner, "a@b.com");
    }

    // HttpUrl
    #[test]
    fn url_accepts_http() {
        let u = HttpUrl::new("http://example.com").unwrap();
        assert!(!u.is_https());
    }

    #[test]
    fn url_accepts_https() {
        let u = HttpUrl::new("https://example.com/path").unwrap();
        assert!(u.is_https());
    }

    #[test]
    fn url_rejects_empty() {
        assert_eq!(HttpUrl::new("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn url_rejects_missing_scheme() {
        assert!(HttpUrl::new("ftp://example.com").is_err());
    }

    #[test]
    fn url_rejects_empty_host() {
        assert!(HttpUrl::new("https://").is_err());
    }

    #[test]
    fn url_rejects_missing_host_before_path() {
        assert!(HttpUrl::new("https:///path").is_err());
    }

    #[test]
    fn url_display() {
        use alloc::string::ToString;
        let u = HttpUrl::new("https://example.com").unwrap();
        assert_eq!(u.to_string(), "https://example.com");
    }

    #[test]
    fn url_is_https_uppercase_scheme() {
        let u = HttpUrl::new("HTTPS://example.com").unwrap();
        assert!(u.is_https());
    }

    #[test]
    fn url_accepts_uppercase_http_scheme() {
        let u = HttpUrl::new("HTTP://example.com").unwrap();
        assert!(!u.is_https());
    }

    #[test]
    fn url_is_http_not_https() {
        let u = HttpUrl::new("http://example.com").unwrap();
        assert!(!u.is_https());
    }

    #[test]
    fn url_from_str_and_string_comparisons() {
        let url = "https://example.com".parse::<HttpUrl>().unwrap();
        let owned = String::from("https://example.com");
        assert_eq!(url, "https://example.com");
        assert_eq!(url, owned);
        assert!("ftp://example.com".parse::<HttpUrl>().is_err());
    }

    #[test]
    fn url_string_ergonomics() {
        let url = HttpUrl::try_from(String::from("https://example.com")).unwrap();
        let borrowed: &str = url.as_ref();
        assert_eq!(borrowed, "https://example.com");
        assert_eq!(&*url, "https://example.com");

        let inner = String::from(url);
        assert_eq!(inner, "https://example.com");
    }

    // HexString
    #[test]
    fn hex_accepts_plain() {
        let h = HexString::new("deadbeef").unwrap();
        assert_eq!(h.hex_digits(), "deadbeef");
        assert!(!h.has_prefix());
    }

    #[test]
    fn hex_accepts_prefixed() {
        let h = HexString::new("0xdeadbeef").unwrap();
        assert_eq!(h.hex_digits(), "deadbeef");
        assert!(h.has_prefix());
    }

    #[test]
    fn hex_accepts_uppercase() {
        assert!(HexString::new("DEADBEEF").is_ok());
    }

    #[test]
    fn hex_rejects_empty() {
        assert_eq!(HexString::new("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn hex_rejects_prefix_only() {
        assert!(HexString::new("0x").is_err());
    }

    #[test]
    fn hex_rejects_invalid_chars() {
        assert!(HexString::new("xyz").is_err());
    }

    #[test]
    fn hex_display() {
        use alloc::string::ToString;
        assert_eq!(HexString::new("ff00").unwrap().to_string(), "ff00");
    }

    #[test]
    fn hex_from_str_and_string_comparisons() {
        let hex = "ff00".parse::<HexString>().unwrap();
        let owned = String::from("ff00");
        assert_eq!(hex, "ff00");
        assert_eq!(hex, owned);
        assert!("xyz".parse::<HexString>().is_err());
    }

    #[test]
    fn hex_string_ergonomics() {
        let hex = HexString::try_from(String::from("ff00")).unwrap();
        let borrowed: &str = hex.as_ref();
        assert_eq!(borrowed, "ff00");
        assert_eq!(&*hex, "ff00");

        let inner = String::from(hex);
        assert_eq!(inner, "ff00");
    }

    #[test]
    fn base64_accepts_valid() {
        assert_eq!(Base64::new("aGVsbG8=").unwrap().as_str(), "aGVsbG8=");
        assert!(Base64::new("YWJjZA==").is_ok()); // two pads
        assert!(Base64::new("YWJjZGU+").is_ok()); // '+' and no pad
        assert!(Base64::new("ab/+ZZ90").is_ok());
    }

    #[test]
    fn base64_rejects_bad() {
        assert_eq!(
            Base64::new("").unwrap_err().kind(),
            PrimitiveErrorKind::Empty
        );
        assert_eq!(
            Base64::new("aGVsbG8").unwrap_err().kind(), // not a multiple of 4
            PrimitiveErrorKind::InvalidFormat
        );
        assert!(Base64::new("ab-_ZZ90").is_err()); // url-safe alphabet
        assert!(Base64::new("ab=cZZ90").is_err()); // interior padding
        assert!(Base64::new("ab======").is_err()); // too much padding
    }

    #[test]
    fn base64_padding_and_decoded_len() {
        let b = Base64::new("aGVsbG8=").unwrap(); // "hello" -> 5 bytes
        assert!(b.is_padded());
        assert_eq!(b.decoded_len(), 5);
        let b = Base64::new("YWJjZA==").unwrap(); // "abcd" -> 4 bytes
        assert_eq!(b.decoded_len(), 4);
        let b = Base64::new("YWJjZGZn").unwrap(); // 6 bytes, no pad
        assert!(!b.is_padded());
        assert_eq!(b.decoded_len(), 6);
    }

    #[test]
    fn identifier_accepts_valid() {
        assert_eq!(Identifier::new("user_id").unwrap().as_str(), "user_id");
        assert!(Identifier::new("_private").is_ok());
        assert!(Identifier::new("A1").is_ok());
        assert!(Identifier::new("x").is_ok());
    }

    #[test]
    fn identifier_rejects_bad() {
        assert_eq!(
            Identifier::new("").unwrap_err().kind(),
            PrimitiveErrorKind::Empty
        );
        assert!(Identifier::new("3bad").is_err()); // starts with digit
        assert!(Identifier::new("has space").is_err());
        assert!(Identifier::new("dash-no").is_err());
        assert!(Identifier::new("café").is_err()); // non-ascii
    }

    #[test]
    fn hostname_accepts_valid() {
        assert_eq!(
            Hostname::new("api.example.com").unwrap().as_str(),
            "api.example.com"
        );
        assert!(Hostname::new("localhost").is_ok());
        assert!(Hostname::new("a-b.c-d.example").is_ok());
        let h = Hostname::new("api.example.com").unwrap();
        let labels: alloc::vec::Vec<&str> = h.labels().collect();
        assert_eq!(labels, ["api", "example", "com"]);
    }

    #[test]
    fn hostname_rejects_bad() {
        assert_eq!(
            Hostname::new("").unwrap_err().kind(),
            PrimitiveErrorKind::Empty
        );
        assert!(Hostname::new("-bad.com").is_err()); // leading hyphen
        assert!(Hostname::new("bad-.com").is_err()); // trailing hyphen
        assert!(Hostname::new("a..b").is_err()); // empty label
        assert!(Hostname::new(".leading").is_err());
        assert!(Hostname::new("trailing.").is_err());
        assert!(Hostname::new("under_score.com").is_err()); // underscore not allowed
        assert!(Hostname::new(String::from("a").repeat(64)).is_err());
        let too_long = alloc::format!("{}.com", String::from("a").repeat(252));
        assert!(Hostname::new(too_long).is_err());
    }

    // The owned/reference comparisons below intentionally exercise the
    // `PartialEq<String>` and `PartialEq<&String>` impls, which `cmp_owned` and
    // `op_ref` would otherwise rewrite away.
    #[test]
    #[allow(clippy::cmp_owned, clippy::op_ref)]
    fn base64_conversions_and_traits() {
        let from_str: Base64 = "YWJj".parse().unwrap();
        let try_ref = Base64::try_from("YWJj").unwrap();
        let try_owned = Base64::try_from(String::from("YWJj")).unwrap();
        assert_eq!(from_str, try_ref);
        assert_eq!(try_ref, try_owned);

        assert_eq!(try_ref.to_string(), "YWJj"); // Display
        let as_ref: &str = try_ref.as_ref(); // AsRef
        assert_eq!(as_ref, "YWJj");
        assert_eq!(&*try_ref, "YWJj"); // Deref
        assert!(try_ref == "YWJj"); // PartialEq<&str>
        assert!(try_ref == *"YWJj"); // PartialEq<str>
        assert!(try_ref == String::from("YWJj")); // PartialEq<String>
        assert!(try_ref == &String::from("YWJj")); // PartialEq<&String>
        assert_eq!(String::from(try_owned), "YWJj"); // From<Base64> for String
    }

    #[test]
    #[allow(clippy::cmp_owned, clippy::op_ref)]
    fn identifier_conversions_and_traits() {
        let from_str: Identifier = "user_id".parse().unwrap();
        let try_ref = Identifier::try_from("user_id").unwrap();
        let try_owned = Identifier::try_from(String::from("user_id")).unwrap();
        assert_eq!(from_str, try_ref);
        assert_eq!(try_ref, try_owned);

        assert_eq!(try_ref.to_string(), "user_id");
        let as_ref: &str = try_ref.as_ref();
        assert_eq!(as_ref, "user_id");
        assert_eq!(&*try_ref, "user_id");
        assert!(try_ref == "user_id");
        assert!(try_ref == *"user_id");
        assert!(try_ref == String::from("user_id"));
        assert!(try_ref == &String::from("user_id"));
        assert_eq!(String::from(try_owned), "user_id");
    }

    #[test]
    #[allow(clippy::cmp_owned, clippy::op_ref)]
    fn hostname_conversions_and_traits() {
        let from_str: Hostname = "example.com".parse().unwrap();
        let try_ref = Hostname::try_from("example.com").unwrap();
        let try_owned = Hostname::try_from(String::from("example.com")).unwrap();
        assert_eq!(from_str, try_ref);
        assert_eq!(try_ref, try_owned);

        assert_eq!(try_ref.to_string(), "example.com");
        let as_ref: &str = try_ref.as_ref();
        assert_eq!(as_ref, "example.com");
        assert_eq!(&*try_ref, "example.com");
        assert!(try_ref == "example.com");
        assert!(try_ref == *"example.com");
        assert!(try_ref == String::from("example.com"));
        assert!(try_ref == &String::from("example.com"));
        assert_eq!(String::from(try_owned), "example.com");
    }
}

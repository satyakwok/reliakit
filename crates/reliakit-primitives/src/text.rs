use crate::{PrimitiveError, PrimitiveResult};
use alloc::string::String;
use core::{fmt, ops::Deref};

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

    pub fn as_str(&self) -> &str {
        &self.0
    }

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

// ── Email ─────────────────────────────────────────────────────────────────────

/// Email address with basic structural validation.
///
/// Checks: exactly one `@`, non-empty local part and domain, domain contains
/// at least one `.`, no whitespace. Not a full RFC 5321 validator.
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

    pub fn as_str(&self) -> &str {
        &self.0
    }

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
    if s.contains(' ') {
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
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
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

impl TryFrom<&str> for Email {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
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
        let lower = value.to_lowercase();
        let after_scheme = if let Some(rest) = lower.strip_prefix("https://") {
            rest
        } else if let Some(rest) = lower.strip_prefix("http://") {
            rest
        } else {
            return Err(PrimitiveError::Invalid {
                message: "URL must start with http:// or https://",
            });
        };
        if after_scheme.is_empty() {
            return Err(PrimitiveError::Invalid {
                message: "URL must have a non-empty host",
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    /// Returns `true` if the URL uses `https`.
    pub fn is_https(&self) -> bool {
        self.0.len() >= 8 && self.0[..8].eq_ignore_ascii_case("https://")
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

impl TryFrom<&str> for HttpUrl {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
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

    pub fn as_str(&self) -> &str {
        &self.0
    }

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

impl TryFrom<&str> for HexString {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Email, HexString, HttpUrl, Slug};
    use crate::PrimitiveError;

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
    fn email_rejects_spaces() {
        assert!(Email::new("us er@example.com").is_err());
    }

    #[test]
    fn email_display() {
        use alloc::string::ToString;
        assert_eq!(Email::new("a@b.com").unwrap().to_string(), "a@b.com");
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
    fn url_is_http_not_https() {
        let u = HttpUrl::new("http://example.com").unwrap();
        assert!(!u.is_https());
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
}

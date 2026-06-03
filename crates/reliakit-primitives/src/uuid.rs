use crate::{PrimitiveError, PrimitiveResult};
#[cfg(feature = "alloc")]
use alloc::string::String;
use core::{fmt, str::FromStr};

/// UUID in canonical `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` format.
///
/// Validates format and stores the parsed bytes. Accepts both upper and
/// lowercase hex. Display always outputs lowercase.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Uuid([u8; 16]);

impl Uuid {
    /// Parses a UUID from its canonical string representation.
    pub fn parse(s: &str) -> PrimitiveResult<Self> {
        if s.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        parse_uuid_bytes(s)
            .map(Self)
            .ok_or(PrimitiveError::Invalid {
                message: "UUID must be in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            })
    }

    /// Returns the raw bytes of the UUID.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Returns the UUID version nibble (bits 12-15 of the third group).
    pub fn version(&self) -> u8 {
        (self.0[6] >> 4) & 0x0f
    }
}

fn parse_uuid_bytes(s: &str) -> Option<[u8; 16]> {
    if s.len() != 36 {
        return None;
    }
    let b = s.as_bytes();
    if b[8] != b'-' || b[13] != b'-' || b[18] != b'-' || b[23] != b'-' {
        return None;
    }
    let groups = [(0, 8), (9, 13), (14, 18), (19, 23), (24, 36)];
    let mut result = [0u8; 16];
    let mut byte_idx = 0;
    for (start, end) in groups {
        let mut i = start;
        while i < end {
            let hi = hex_val(b[i])?;
            let lo = hex_val(b[i + 1])?;
            result[byte_idx] = (hi << 4) | lo;
            byte_idx += 1;
            i += 2;
        }
    }
    Some(result)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let b = &self.0;
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            b[0], b[1], b[2], b[3],
            b[4], b[5],
            b[6], b[7],
            b[8], b[9],
            b[10], b[11], b[12], b[13], b[14], b[15]
        )
    }
}

impl FromStr for Uuid {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl PartialEq<str> for Uuid {
    fn eq(&self, other: &str) -> bool {
        Self::parse(other).is_ok_and(|other| self == &other)
    }
}

impl PartialEq<&str> for Uuid {
    fn eq(&self, other: &&str) -> bool {
        self.eq(*other)
    }
}

#[cfg(feature = "alloc")]
impl PartialEq<String> for Uuid {
    fn eq(&self, other: &String) -> bool {
        self.eq(other.as_str())
    }
}

#[cfg(feature = "alloc")]
impl PartialEq<&String> for Uuid {
    fn eq(&self, other: &&String) -> bool {
        self.eq(other.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Uuid;
    use crate::PrimitiveError;
    use alloc::string::ToString;

    const SAMPLE: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn parses_valid_uuid() {
        let u = Uuid::parse(SAMPLE).unwrap();
        assert_eq!(u.to_string(), SAMPLE);
    }

    #[test]
    fn parses_uppercase() {
        let upper = "550E8400-E29B-41D4-A716-446655440000";
        let u = Uuid::parse(upper).unwrap();
        assert_eq!(u.to_string(), SAMPLE);
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(Uuid::parse("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn rejects_too_short() {
        assert!(Uuid::parse("550e8400-e29b-41d4-a716").is_err());
    }

    #[test]
    fn rejects_missing_dashes() {
        assert!(Uuid::parse("550e8400e29b41d4a716446655440000").is_err());
    }

    #[test]
    fn rejects_invalid_hex() {
        assert!(Uuid::parse("550e8400-e29b-41d4-a716-44665544000g").is_err());
    }

    #[test]
    fn version() {
        let u = Uuid::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(u.version(), 4);
    }

    #[test]
    fn as_bytes_roundtrip() {
        let u = Uuid::parse(SAMPLE).unwrap();
        assert_eq!(u.as_bytes().len(), 16);
    }

    #[test]
    fn display_is_lowercase() {
        let upper = "550E8400-E29B-41D4-A716-446655440000";
        let u = Uuid::parse(upper).unwrap();
        assert_eq!(u.to_string(), upper.to_lowercase());
    }

    #[test]
    fn from_str_and_string_comparisons() {
        let uuid = SAMPLE.parse::<Uuid>().unwrap();
        let owned = SAMPLE.to_string();
        assert_eq!(uuid, SAMPLE);
        assert_eq!(uuid, owned);
        assert!("not-a-uuid".parse::<Uuid>().is_err());
    }
}

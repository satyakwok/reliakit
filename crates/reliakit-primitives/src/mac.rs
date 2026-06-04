use crate::{PrimitiveError, PrimitiveResult};
use core::{fmt, str::FromStr};

/// A 48-bit IEEE 802 MAC address, stored as six octets.
///
/// [`parse`](Self::parse) accepts the common `aa:bb:cc:dd:ee:ff` and
/// `aa-bb-cc-dd-ee-ff` text forms (one consistent separator, lower- or
/// upper-case hex). The type is allocation-free and `no_std`; [`Display`] always
/// renders the canonical lowercase, colon-separated form.
///
/// [`Display`]: core::fmt::Display
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Builds a MAC address directly from six octets. Always valid.
    pub const fn from_octets(octets: [u8; 6]) -> Self {
        Self(octets)
    }

    /// Parses a MAC address in `aa:bb:cc:dd:ee:ff` or `aa-bb-cc-dd-ee-ff` form.
    ///
    /// The separator must be `:` or `-` and the same throughout. Returns an
    /// error for an empty string, the wrong length, an inconsistent separator,
    /// or a non-hex octet.
    pub fn parse(value: &str) -> PrimitiveResult<Self> {
        if value.is_empty() {
            return Err(PrimitiveError::Empty);
        }
        let bytes = value.as_bytes();
        // 6 two-digit octets plus 5 separators.
        if bytes.len() != 17 {
            return Err(PrimitiveError::Invalid {
                message: "MAC address must be 17 characters: six octets and five separators",
            });
        }
        let sep = bytes[2];
        if sep != b':' && sep != b'-' {
            return Err(PrimitiveError::Invalid {
                message: "MAC address separator must be ':' or '-'",
            });
        }
        let mut octets = [0u8; 6];
        let mut i = 0;
        while i < 6 {
            let pos = i * 3;
            if i < 5 && bytes[pos + 2] != sep {
                return Err(PrimitiveError::Invalid {
                    message: "MAC address must use a single, consistent separator",
                });
            }
            let hi = hex_digit(bytes[pos])?;
            let lo = hex_digit(bytes[pos + 1])?;
            octets[i] = (hi << 4) | lo;
            i += 1;
        }
        Ok(Self(octets))
    }

    /// Returns the six octets.
    pub const fn octets(&self) -> [u8; 6] {
        self.0
    }

    /// Returns `true` if this is a multicast address (low bit of the first
    /// octet set).
    pub const fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }

    /// Returns `true` if this is a unicast address.
    pub const fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }

    /// Returns `true` if this address is locally administered (second-lowest bit
    /// of the first octet set), as opposed to a universally administered (OUI)
    /// address.
    pub const fn is_local(&self) -> bool {
        self.0[0] & 0x02 != 0
    }

    /// Returns `true` if this address is universally administered.
    pub const fn is_universal(&self) -> bool {
        !self.is_local()
    }
}

/// Converts one ASCII hex digit to its 0–15 value.
fn hex_digit(b: u8) -> PrimitiveResult<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(PrimitiveError::Invalid {
            message: "MAC address octets must be hexadecimal",
        }),
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let o = self.0;
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            o[0], o[1], o[2], o[3], o[4], o[5]
        )
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(octets: [u8; 6]) -> Self {
        Self::from_octets(octets)
    }
}

impl From<MacAddress> for [u8; 6] {
    fn from(mac: MacAddress) -> Self {
        mac.0
    }
}

impl TryFrom<&str> for MacAddress {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for MacAddress {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::MacAddress;
    use crate::PrimitiveErrorKind;

    #[test]
    fn parses_colon_and_dash_forms() {
        let m = MacAddress::parse("0A:1b:2C:3d:4E:5f").unwrap();
        assert_eq!(m.octets(), [0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f]);
        let d = MacAddress::parse("0a-1b-2c-3d-4e-5f").unwrap();
        assert_eq!(d, m);
    }

    #[test]
    fn display_is_canonical_lowercase_colon() {
        let m = MacAddress::from_octets([0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45]);
        extern crate alloc;
        use alloc::string::ToString;
        assert_eq!(m.to_string(), "ab:cd:ef:01:23:45");
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(
            MacAddress::parse("").unwrap_err().kind(),
            PrimitiveErrorKind::Empty
        );
        assert!(MacAddress::parse("aa:bb:cc:dd:ee").is_err()); // too short
        assert!(MacAddress::parse("aa:bb:cc:dd:ee:ff:00").is_err()); // too long
        assert!(MacAddress::parse("aa:bb:cc-dd:ee:ff").is_err()); // mixed separators
        assert!(MacAddress::parse("aa:bb:cc:dd:ee:gg").is_err()); // non-hex
        assert!(MacAddress::parse("aabb.ccdd.eeff").is_err()); // wrong format
    }

    #[test]
    fn classification_bits() {
        // Multicast: low bit of first octet set.
        assert!(MacAddress::from_octets([0x01, 0, 0, 0, 0, 0]).is_multicast());
        assert!(MacAddress::from_octets([0x02, 0, 0, 0, 0, 0]).is_unicast());
        // Locally administered: second-lowest bit set.
        assert!(MacAddress::from_octets([0x02, 0, 0, 0, 0, 0]).is_local());
        assert!(MacAddress::from_octets([0x00, 0, 0, 0, 0, 0]).is_universal());
    }
}

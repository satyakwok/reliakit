use crate::{PrimitiveError, PrimitiveResult};
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use core::str::FromStr;

/// An IP network in CIDR notation: a base address plus a prefix length.
///
/// `Cidr` validates that the prefix length is in range for the address family
/// (`0..=32` for IPv4, `0..=128` for IPv6) at construction. It accepts both IPv4
/// and IPv6 via [`core::net::IpAddr`], stores the address exactly as supplied
/// (host bits are not cleared), and offers membership testing with
/// [`contains`](Cidr::contains) and the canonical network base with
/// [`network`](Cidr::network).
///
/// This type is allocation-free and `no_std`-friendly.
///
/// # Examples
///
/// ```
/// use reliakit_primitives::Cidr;
///
/// let net: Cidr = "192.168.1.0/24".parse().unwrap();
/// assert!(net.contains("192.168.1.42".parse().unwrap()));
/// assert!(!net.contains("192.168.2.1".parse().unwrap()));
/// assert_eq!(net.prefix_len(), 24);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cidr {
    addr: IpAddr,
    prefix_len: u8,
}

impl Cidr {
    /// Creates a `Cidr` from an address and prefix length.
    ///
    /// Returns [`PrimitiveError::Invalid`] if `prefix_len` exceeds the maximum
    /// for the address family (32 for IPv4, 128 for IPv6).
    pub fn new(addr: IpAddr, prefix_len: u8) -> PrimitiveResult<Self> {
        let max = match addr {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };
        if prefix_len > max {
            return Err(PrimitiveError::Invalid {
                message: "prefix length out of range for the address family",
            });
        }
        Ok(Self { addr, prefix_len })
    }

    /// Returns the base address exactly as supplied (host bits not cleared).
    pub const fn address(&self) -> IpAddr {
        self.addr
    }

    /// Returns the prefix length (number of leading network bits).
    pub const fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    /// Returns `true` if the network is IPv4.
    pub const fn is_ipv4(&self) -> bool {
        matches!(self.addr, IpAddr::V4(_))
    }

    /// Returns `true` if the network is IPv6.
    pub const fn is_ipv6(&self) -> bool {
        matches!(self.addr, IpAddr::V6(_))
    }

    /// Returns the canonical network address with host bits cleared.
    ///
    /// For `192.168.1.42/24` this returns `192.168.1.0`.
    pub fn network(&self) -> IpAddr {
        match self.addr {
            IpAddr::V4(a) => {
                IpAddr::V4(Ipv4Addr::from_bits(a.to_bits() & mask_v4(self.prefix_len)))
            }
            IpAddr::V6(a) => {
                IpAddr::V6(Ipv6Addr::from_bits(a.to_bits() & mask_v6(self.prefix_len)))
            }
        }
    }

    /// Returns `true` if `ip` falls within this network.
    ///
    /// An address of a different family than the network is never contained.
    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, ip) {
            (IpAddr::V4(net), IpAddr::V4(probe)) => {
                let m = mask_v4(self.prefix_len);
                net.to_bits() & m == probe.to_bits() & m
            }
            (IpAddr::V6(net), IpAddr::V6(probe)) => {
                let m = mask_v6(self.prefix_len);
                net.to_bits() & m == probe.to_bits() & m
            }
            _ => false,
        }
    }
}

/// Builds a left-aligned mask of `prefix` set bits for a 32-bit address.
fn mask_v4(prefix: u8) -> u32 {
    match prefix {
        0 => 0,
        p if p >= 32 => u32::MAX,
        p => u32::MAX << (32 - p),
    }
}

/// Builds a left-aligned mask of `prefix` set bits for a 128-bit address.
fn mask_v6(prefix: u8) -> u128 {
    match prefix {
        0 => 0,
        p if p >= 128 => u128::MAX,
        p => u128::MAX << (128 - p),
    }
}

impl fmt::Display for Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.addr, self.prefix_len)
    }
}

impl FromStr for Cidr {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr_part, prefix_part) = s.split_once('/').ok_or(PrimitiveError::Invalid {
            message: "CIDR must be written as address/prefix",
        })?;
        let addr = addr_part
            .parse::<IpAddr>()
            .map_err(|_| PrimitiveError::Invalid {
                message: "invalid IP address in CIDR",
            })?;
        let prefix_len = prefix_part
            .parse::<u8>()
            .map_err(|_| PrimitiveError::Invalid {
                message: "invalid prefix length in CIDR",
            })?;
        Self::new(addr, prefix_len)
    }
}

impl TryFrom<&str> for Cidr {
    type Error = PrimitiveError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::Cidr;
    use crate::PrimitiveErrorKind;
    use core::net::IpAddr;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn parses_ipv4_cidr() {
        let net: Cidr = "192.168.1.0/24".parse().unwrap();
        assert!(net.is_ipv4());
        assert_eq!(net.prefix_len(), 24);
        assert_eq!(net.address(), ip("192.168.1.0"));
    }

    #[test]
    fn ipv4_contains_membership() {
        let net: Cidr = "10.0.0.0/8".parse().unwrap();
        assert!(net.contains(ip("10.255.255.255")));
        assert!(net.contains(ip("10.0.0.1")));
        assert!(!net.contains(ip("11.0.0.1")));
    }

    #[test]
    fn host_bits_preserved_but_network_masks() {
        let net: Cidr = "192.168.1.42/24".parse().unwrap();
        assert_eq!(net.address(), ip("192.168.1.42"));
        assert_eq!(net.network(), ip("192.168.1.0"));
    }

    #[test]
    fn prefix_zero_matches_everything() {
        let net: Cidr = "0.0.0.0/0".parse().unwrap();
        assert!(net.contains(ip("8.8.8.8")));
        assert!(net.contains(ip("255.255.255.255")));
    }

    #[test]
    fn prefix_32_is_single_host() {
        let net: Cidr = "192.168.1.5/32".parse().unwrap();
        assert!(net.contains(ip("192.168.1.5")));
        assert!(!net.contains(ip("192.168.1.6")));
    }

    #[test]
    fn parses_ipv6_cidr() {
        let net: Cidr = "2001:db8::/32".parse().unwrap();
        assert!(net.is_ipv6());
        assert_eq!(net.prefix_len(), 32);
        assert!(net.contains(ip("2001:db8:1234::1")));
        assert!(!net.contains(ip("2001:db9::1")));
    }

    #[test]
    fn ipv6_prefix_128_single_host() {
        let net: Cidr = "::1/128".parse().unwrap();
        assert!(net.contains(ip("::1")));
        assert!(!net.contains(ip("::2")));
    }

    #[test]
    fn cross_family_never_contained() {
        let v4: Cidr = "10.0.0.0/8".parse().unwrap();
        assert!(!v4.contains(ip("::1")));
        let v6: Cidr = "2001:db8::/32".parse().unwrap();
        assert!(!v6.contains(ip("10.0.0.1")));
    }

    #[test]
    fn rejects_prefix_out_of_range() {
        assert_eq!(
            "192.168.0.0/33".parse::<Cidr>().unwrap_err().kind(),
            PrimitiveErrorKind::InvalidFormat
        );
        assert_eq!(
            "2001:db8::/129".parse::<Cidr>().unwrap_err().kind(),
            PrimitiveErrorKind::InvalidFormat
        );
    }

    #[test]
    fn rejects_malformed() {
        assert!("192.168.0.0".parse::<Cidr>().is_err()); // no prefix
        assert!("not-an-ip/24".parse::<Cidr>().is_err());
        assert!("192.168.0.0/abc".parse::<Cidr>().is_err());
        assert!("192.168.0.0/".parse::<Cidr>().is_err());
    }

    #[test]
    fn display_round_trips() {
        let net: Cidr = "172.16.0.0/12".parse().unwrap();
        assert_eq!(net.to_string(), "172.16.0.0/12");
        let v6: Cidr = "fe80::/10".parse().unwrap();
        assert_eq!(v6.to_string(), "fe80::/10");
    }

    #[test]
    fn try_from_str() {
        assert!(Cidr::try_from("10.0.0.0/8").is_ok());
        assert!(Cidr::try_from("bad").is_err());
    }
}

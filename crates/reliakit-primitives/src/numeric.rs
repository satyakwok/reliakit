use crate::{PrimitiveError, PrimitiveResult};
use core::fmt;

/// Percentage value from 0 to 100 inclusive.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u8);

impl Percent {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 100;

    /// Creates a new percentage value.
    pub fn new(value: u8) -> PrimitiveResult<Self> {
        if value > Self::MAX {
            return Err(PrimitiveError::OutOfRange {
                min: Self::MIN as u128,
                max: Self::MAX as u128,
                actual: value as u128,
            });
        }
        Ok(Self(value))
    }

    /// Returns the integer percentage value.
    pub const fn get(self) -> u8 {
        self.0
    }

    /// Returns the percentage as a fraction between 0.0 and 1.0.
    pub fn as_fraction(self) -> f64 {
        f64::from(self.0) / 100.0
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl TryFrom<u8> for Percent {
    type Error = PrimitiveError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Percent> for u8 {
    fn from(value: Percent) -> Self {
        value.get()
    }
}

/// TCP/UDP port number from 1 to 65535 inclusive.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Port(u16);

impl Port {
    pub const MIN: u16 = 1;
    pub const MAX: u16 = 65535;

    /// Creates a new port.
    pub fn new(value: u16) -> PrimitiveResult<Self> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(PrimitiveError::OutOfRange {
                min: Self::MIN as u128,
                max: Self::MAX as u128,
                actual: value as u128,
            });
        }
        Ok(Self(value))
    }

    /// Returns the port number.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u16> for Port {
    type Error = PrimitiveError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Port> for u16 {
    fn from(value: Port) -> Self {
        value.get()
    }
}

/// Byte size value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteSize(u64);

impl ByteSize {
    /// Creates a size from bytes.
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Creates a size from kibibytes.
    pub const fn from_kb(kb: u64) -> Self {
        Self(kb.saturating_mul(1024))
    }

    /// Creates a size from mebibytes.
    pub const fn from_mb(mb: u64) -> Self {
        Self(mb.saturating_mul(1024).saturating_mul(1024))
    }

    /// Creates a size from gibibytes.
    pub const fn from_gb(gb: u64) -> Self {
        Self(
            gb.saturating_mul(1024)
                .saturating_mul(1024)
                .saturating_mul(1024),
        )
    }

    /// Returns the size in bytes.
    pub const fn as_bytes(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        let bytes = self.0;
        if bytes < KB {
            write!(f, "{bytes} B")
        } else if bytes < MB {
            write!(f, "{:.2} KB", bytes as f64 / KB as f64)
        } else if bytes < GB {
            write!(f, "{:.2} MB", bytes as f64 / MB as f64)
        } else {
            write!(f, "{:.2} GB", bytes as f64 / GB as f64)
        }
    }
}

impl From<u64> for ByteSize {
    fn from(value: u64) -> Self {
        Self::from_bytes(value)
    }
}

impl From<ByteSize> for u64 {
    fn from(value: ByteSize) -> Self {
        value.as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteSize, Percent, Port};
    use crate::PrimitiveError;
    use alloc::string::ToString;

    #[test]
    fn percent_accepts_boundary_values() {
        assert_eq!(Percent::new(0).unwrap().get(), 0);
        assert_eq!(Percent::new(50).unwrap().get(), 50);
        assert_eq!(Percent::new(100).unwrap().get(), 100);
    }

    #[test]
    fn percent_rejects_out_of_range() {
        assert_eq!(
            Percent::new(101).unwrap_err(),
            PrimitiveError::OutOfRange {
                min: 0,
                max: 100,
                actual: 101
            }
        );
    }

    #[test]
    fn percent_display_prints_percent_sign() {
        assert_eq!(Percent::new(42).unwrap().to_string(), "42%");
    }

    #[test]
    fn percent_fraction() {
        assert_eq!(Percent::new(25).unwrap().as_fraction(), 0.25);
    }

    #[test]
    fn port_accepts_boundaries() {
        assert_eq!(Port::new(1).unwrap().get(), 1);
        assert_eq!(Port::new(65535).unwrap().get(), 65535);
    }

    #[test]
    fn port_rejects_zero() {
        assert_eq!(
            Port::new(0).unwrap_err(),
            PrimitiveError::OutOfRange {
                min: 1,
                max: 65535,
                actual: 0
            }
        );
    }

    #[test]
    fn byte_size_constructors_work() {
        assert_eq!(ByteSize::from_bytes(512).as_bytes(), 512);
        assert_eq!(ByteSize::from_kb(1).as_bytes(), 1024);
        assert_eq!(ByteSize::from_mb(1).as_bytes(), 1024 * 1024);
        assert_eq!(ByteSize::from_gb(1).as_bytes(), 1024 * 1024 * 1024);
    }

    #[test]
    fn byte_size_display_works() {
        assert_eq!(ByteSize::from_bytes(512).to_string(), "512 B");
        assert_eq!(ByteSize::from_kb(1).to_string(), "1.00 KB");
        assert_eq!(ByteSize::from_kb(1536 / 1024).to_string(), "1.00 KB");
        assert_eq!(ByteSize::from_bytes(1536).to_string(), "1.50 KB");
        assert_eq!(
            ByteSize::from_bytes(1024 * 1024 + 512 * 1024).to_string(),
            "1.50 MB"
        );
        assert_eq!(
            ByteSize::from_bytes(1024 * 1024 * 1024 + 512 * 1024 * 1024).to_string(),
            "1.50 GB"
        );
    }
}

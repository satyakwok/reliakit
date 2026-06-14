use crate::{PrimitiveError, PrimitiveResult};
use core::fmt;

/// Percentage value from 0 to 100 inclusive.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u8);

impl Percent {
    /// Minimum allowed integer percentage.
    pub const MIN: u8 = 0;
    /// Maximum allowed integer percentage.
    pub const MAX: u8 = 100;

    /// Creates a new percentage value.
    pub const fn new(value: u8) -> PrimitiveResult<Self> {
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
    /// Minimum allowed TCP/UDP port.
    pub const MIN: u16 = 1;
    /// Maximum allowed TCP/UDP port.
    pub const MAX: u16 = 65535;

    /// Creates a new port.
    pub const fn new(value: u16) -> PrimitiveResult<Self> {
        if value < Self::MIN {
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

    /// Creates a size from kibibytes (1 KiB = 1024 bytes).
    ///
    /// Saturates to `u64::MAX` on overflow instead of panicking.
    pub const fn from_kb(kb: u64) -> Self {
        Self(kb.saturating_mul(1024))
    }

    /// Creates a size from mebibytes (1 MiB = 1024 KiB).
    ///
    /// Saturates to `u64::MAX` on overflow instead of panicking.
    pub const fn from_mb(mb: u64) -> Self {
        Self(mb.saturating_mul(1024 * 1024))
    }

    /// Creates a size from gibibytes (1 GiB = 1024 MiB).
    ///
    /// Saturates to `u64::MAX` on overflow instead of panicking.
    pub const fn from_gb(gb: u64) -> Self {
        Self(gb.saturating_mul(1024 * 1024 * 1024))
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

// ── PositiveInt ───────────────────────────────────────────────────────────────

/// Integer value strictly greater than zero.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PositiveInt(u64);

impl PositiveInt {
    /// Creates a `PositiveInt`. Returns `OutOfRange` if `value` is zero.
    pub const fn new(value: u64) -> PrimitiveResult<Self> {
        if value == 0 {
            return Err(PrimitiveError::OutOfRange {
                min: 1,
                max: u64::MAX as u128,
                actual: 0,
            });
        }
        Ok(Self(value))
    }

    /// Returns the positive integer value.
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for PositiveInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u64> for PositiveInt {
    type Error = PrimitiveError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveInt> for u64 {
    fn from(value: PositiveInt) -> Self {
        value.get()
    }
}

// ── PositiveFloat ─────────────────────────────────────────────────────────────

/// Finite floating-point value strictly greater than zero.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct PositiveFloat(f64);

impl PositiveFloat {
    /// Creates a `PositiveFloat`. Returns `Invalid` if `value` is not finite
    /// or is not greater than zero.
    pub fn new(value: f64) -> PrimitiveResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            return Err(PrimitiveError::Invalid {
                message: "value must be a finite positive number greater than zero",
            });
        }
        Ok(Self(value))
    }

    /// Returns the positive floating-point value.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl fmt::Display for PositiveFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<f64> for PositiveFloat {
    type Error = PrimitiveError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// ── PercentFloat ─────────────────────────────────────────────────────────────

/// Percentage value as `f64` in the range `0.0..=100.0`.
///
/// Use this when decimal precision is required. For integer percentages, use
/// [`Percent`].
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct PercentFloat(f64);

impl PercentFloat {
    /// Minimum allowed floating-point percentage.
    pub const MIN: f64 = 0.0;
    /// Maximum allowed floating-point percentage.
    pub const MAX: f64 = 100.0;

    /// Creates a `PercentFloat`. Returns `Invalid` if `value` is not finite
    /// or is outside `0.0..=100.0`.
    pub fn new(value: f64) -> PrimitiveResult<Self> {
        if !value.is_finite() || !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(PrimitiveError::Invalid {
                message: "percentage must be a finite number between 0.0 and 100.0 inclusive",
            });
        }
        Ok(Self(value))
    }

    /// Returns the floating-point percentage value.
    pub fn get(self) -> f64 {
        self.0
    }

    /// Returns the value as a fraction between `0.0` and `1.0`.
    pub fn as_fraction(self) -> f64 {
        self.0 / 100.0
    }
}

impl fmt::Display for PercentFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl TryFrom<f64> for PercentFloat {
    type Error = PrimitiveError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteSize, Percent, PercentFloat, Port, PositiveFloat, PositiveInt};
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

    #[test]
    fn percent_try_from_u8() {
        assert_eq!(Percent::try_from(50u8).unwrap().get(), 50);
        assert!(Percent::try_from(101u8).is_err());
    }

    #[test]
    fn percent_from_into_u8() {
        let p = Percent::new(75).unwrap();
        let v: u8 = p.into();
        assert_eq!(v, 75);
    }

    #[test]
    fn port_try_from_u16() {
        assert_eq!(Port::try_from(8080u16).unwrap().get(), 8080);
        assert!(Port::try_from(0u16).is_err());
    }

    #[test]
    fn port_from_into_u16() {
        let p = Port::new(443).unwrap();
        let v: u16 = p.into();
        assert_eq!(v, 443);
    }

    #[test]
    fn port_display() {
        assert_eq!(Port::new(8080).unwrap().to_string(), "8080");
    }

    #[test]
    fn byte_size_from_u64() {
        let s = ByteSize::from(2048u64);
        assert_eq!(s.as_bytes(), 2048);
    }

    #[test]
    fn byte_size_into_u64() {
        let s = ByteSize::from_bytes(4096);
        let v: u64 = s.into();
        assert_eq!(v, 4096);
    }

    #[test]
    fn positive_int_accepts_nonzero() {
        assert_eq!(PositiveInt::new(1).unwrap().get(), 1);
        assert_eq!(PositiveInt::new(u64::MAX).unwrap().get(), u64::MAX);
    }

    #[test]
    fn positive_int_rejects_zero() {
        assert!(PositiveInt::new(0).is_err());
    }

    #[test]
    fn positive_int_display() {
        assert_eq!(PositiveInt::new(42).unwrap().to_string(), "42");
    }

    #[test]
    fn positive_int_try_from_and_into() {
        let p = PositiveInt::try_from(10u64).unwrap();
        let v: u64 = p.into();
        assert_eq!(v, 10);
    }

    #[test]
    fn positive_float_accepts_positive() {
        assert_eq!(PositiveFloat::new(0.001).unwrap().get(), 0.001);
        assert_eq!(PositiveFloat::new(f64::MAX).unwrap().get(), f64::MAX);
    }

    #[test]
    fn positive_float_rejects_zero() {
        assert!(PositiveFloat::new(0.0).is_err());
    }

    #[test]
    fn positive_float_rejects_negative() {
        assert!(PositiveFloat::new(-1.0).is_err());
    }

    #[test]
    fn positive_float_rejects_nan() {
        assert!(PositiveFloat::new(f64::NAN).is_err());
    }

    #[test]
    fn positive_float_rejects_infinity() {
        assert!(PositiveFloat::new(f64::INFINITY).is_err());
    }

    #[test]
    fn positive_float_try_from() {
        assert!(PositiveFloat::try_from(1.5f64).is_ok());
        assert!(PositiveFloat::try_from(0.0f64).is_err());
    }

    #[test]
    fn percentage_f64_accepts_boundaries() {
        assert_eq!(PercentFloat::new(0.0).unwrap().get(), 0.0);
        assert_eq!(PercentFloat::new(50.5).unwrap().get(), 50.5);
        assert_eq!(PercentFloat::new(100.0).unwrap().get(), 100.0);
    }

    #[test]
    fn percentage_f64_rejects_out_of_range() {
        assert!(PercentFloat::new(-0.1).is_err());
        assert!(PercentFloat::new(100.1).is_err());
    }

    #[test]
    fn percentage_f64_rejects_nan() {
        assert!(PercentFloat::new(f64::NAN).is_err());
    }

    #[test]
    fn percentage_f64_as_fraction() {
        assert_eq!(PercentFloat::new(25.0).unwrap().as_fraction(), 0.25);
    }

    #[test]
    fn percentage_f64_display() {
        assert_eq!(PercentFloat::new(42.5).unwrap().to_string(), "42.5%");
    }

    #[test]
    fn percentage_f64_try_from() {
        assert!(PercentFloat::try_from(50.0f64).is_ok());
        assert!(PercentFloat::try_from(101.0f64).is_err());
    }
}

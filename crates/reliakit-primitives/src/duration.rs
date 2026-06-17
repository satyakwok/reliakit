use crate::{PrimitiveError, PrimitiveResult};
#[cfg(feature = "alloc")]
use alloc::string::String;
use core::{fmt, str::FromStr, time::Duration};

/// Human-readable duration parsed from strings like `1h`, `30m`, `45s`,
/// `500ms`, or combinations such as `1h30m45s`.
///
/// Supported units: `h` (hours), `m` (minutes), `s` (seconds), `ms`
/// (milliseconds). Units must appear in descending order, each at most once.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HumanDuration(Duration);

impl HumanDuration {
    /// Parses a human-readable duration string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use reliakit_primitives::HumanDuration;
    /// let d = HumanDuration::parse("1h30m").unwrap();
    /// assert_eq!(d.as_secs(), 5400);
    /// ```
    pub fn parse(s: &str) -> PrimitiveResult<Self> {
        if s.is_empty() {
            return Err(PrimitiveError::Empty);
        }

        // Unit rank: higher = larger unit. Each unit may appear at most once,
        // and units must be provided in strictly descending order (h > m > s > ms).
        const RANK_H: u8 = 4;
        const RANK_M: u8 = 3;
        const RANK_S: u8 = 2;
        const RANK_MS: u8 = 1;

        let mut total_nanos: u128 = 0;
        let mut last_rank: u8 = u8::MAX;
        let mut pos = 0;
        let bytes = s.as_bytes();

        while pos < bytes.len() {
            // Parse digits
            let num_start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos == num_start {
                return Err(PrimitiveError::Invalid {
                    message: "expected a number before unit",
                });
            }
            let num_str = &s[num_start..pos];
            let num = parse_u64(num_str).ok_or(PrimitiveError::Invalid {
                message: "duration number is too large",
            })?;

            // Parse unit (1 or 2 ASCII alpha chars)
            let unit_start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
                pos += 1;
            }
            let unit = &s[unit_start..pos];

            let (nanos_per_unit, rank): (u128, u8) = match unit {
                "h" => (3_600 * 1_000_000_000, RANK_H),
                "m" => (60 * 1_000_000_000, RANK_M),
                "s" => (1_000_000_000, RANK_S),
                "ms" => (1_000_000, RANK_MS),
                _ => {
                    return Err(PrimitiveError::Invalid {
                        message: "unknown time unit; use h, m, s, or ms",
                    });
                }
            };

            if rank >= last_rank {
                return Err(PrimitiveError::Invalid {
                    message: "units must be in descending order (h, m, s, ms) with no duplicates",
                });
            }
            last_rank = rank;

            let component =
                (num as u128)
                    .checked_mul(nanos_per_unit)
                    .ok_or(PrimitiveError::Invalid {
                        message: "duration overflow",
                    })?;

            total_nanos = total_nanos
                .checked_add(component)
                .ok_or(PrimitiveError::Invalid {
                    message: "duration overflow",
                })?;
        }

        let secs =
            u64::try_from(total_nanos / 1_000_000_000).map_err(|_| PrimitiveError::Invalid {
                message: "duration overflow: total duration exceeds maximum representable value",
            })?;
        let nanos = (total_nanos % 1_000_000_000) as u32;
        Ok(Self(Duration::new(secs, nanos)))
    }

    /// Returns the underlying `core::time::Duration`.
    pub fn as_duration(self) -> Duration {
        self.0
    }

    /// Returns the total number of whole seconds.
    pub fn as_secs(self) -> u64 {
        self.0.as_secs()
    }

    /// Returns the total number of whole milliseconds.
    pub fn as_millis(self) -> u128 {
        self.0.as_millis()
    }
}

fn parse_u64(s: &str) -> Option<u64> {
    if s.is_empty() {
        return None;
    }
    let mut result: u64 = 0;
    for c in s.chars() {
        let digit = c.to_digit(10)? as u64;
        result = result.checked_mul(10)?.checked_add(digit)?;
    }
    Some(result)
}

impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_secs = self.0.as_secs();
        let millis = self.0.subsec_millis();
        let h = total_secs / 3600;
        let m = (total_secs % 3600) / 60;
        let s = total_secs % 60;

        let mut wrote = false;
        if h > 0 {
            write!(f, "{h}h")?;
            wrote = true;
        }
        if m > 0 {
            write!(f, "{m}m")?;
            wrote = true;
        }
        if s > 0 || millis > 0 {
            if s > 0 {
                write!(f, "{s}s")?;
            }
            if millis > 0 {
                write!(f, "{millis}ms")?;
            }
            wrote = true;
        }
        if !wrote {
            write!(f, "0s")?;
        }
        Ok(())
    }
}

impl FromStr for HumanDuration {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl PartialEq<str> for HumanDuration {
    fn eq(&self, other: &str) -> bool {
        Self::parse(other).is_ok_and(|other| self == &other)
    }
}

impl PartialEq<&str> for HumanDuration {
    fn eq(&self, other: &&str) -> bool {
        self.eq(*other)
    }
}

#[cfg(feature = "alloc")]
impl PartialEq<String> for HumanDuration {
    fn eq(&self, other: &String) -> bool {
        self.eq(other.as_str())
    }
}

#[cfg(feature = "alloc")]
impl PartialEq<&String> for HumanDuration {
    fn eq(&self, other: &&String) -> bool {
        self.eq(other.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::HumanDuration;
    use crate::PrimitiveError;
    use alloc::string::ToString;

    #[test]
    fn parses_seconds() {
        assert_eq!(HumanDuration::parse("45s").unwrap().as_secs(), 45);
    }

    #[test]
    fn parses_minutes() {
        assert_eq!(HumanDuration::parse("2m").unwrap().as_secs(), 120);
    }

    #[test]
    fn parses_hours() {
        assert_eq!(HumanDuration::parse("1h").unwrap().as_secs(), 3600);
    }

    #[test]
    fn parses_milliseconds() {
        assert_eq!(HumanDuration::parse("500ms").unwrap().as_millis(), 500);
    }

    #[test]
    fn parses_combination() {
        let d = HumanDuration::parse("1h30m45s").unwrap();
        assert_eq!(d.as_secs(), 3600 + 1800 + 45);
    }

    #[test]
    fn parses_minutes_and_seconds() {
        assert_eq!(HumanDuration::parse("2m30s").unwrap().as_secs(), 150);
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(HumanDuration::parse("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn rejects_unknown_unit() {
        assert!(HumanDuration::parse("5d").is_err());
    }

    #[test]
    fn rejects_no_number() {
        assert!(HumanDuration::parse("s").is_err());
    }

    #[test]
    fn rejects_out_of_order_units() {
        assert!(HumanDuration::parse("1s1h").is_err());
    }

    #[test]
    fn rejects_duplicate_units() {
        assert!(HumanDuration::parse("1h1h").is_err());
    }

    #[test]
    fn rejects_ms_before_s() {
        assert!(HumanDuration::parse("500ms30s").is_err());
    }

    #[test]
    fn as_duration() {
        let d = HumanDuration::parse("1s").unwrap();
        assert_eq!(d.as_duration().as_secs(), 1);
    }

    #[test]
    fn display_seconds() {
        assert_eq!(HumanDuration::parse("45s").unwrap().to_string(), "45s");
    }

    #[test]
    fn display_combined() {
        assert_eq!(HumanDuration::parse("1h30m").unwrap().to_string(), "1h30m");
    }

    #[test]
    fn display_zero() {
        assert_eq!(HumanDuration::parse("0s").unwrap().to_string(), "0s");
    }

    #[test]
    fn display_mixed_seconds_and_millis() {
        assert_eq!(
            HumanDuration::parse("1s500ms").unwrap().to_string(),
            "1s500ms"
        );
    }

    #[test]
    fn display_millis_only() {
        assert_eq!(HumanDuration::parse("500ms").unwrap().to_string(), "500ms");
    }

    #[test]
    fn rejects_duration_that_overflows_u64_seconds() {
        // u64::MAX hours * 3600 seconds/hour >> u64::MAX seconds
        assert!(HumanDuration::parse("18446744073709551615h").is_err());
    }

    #[test]
    fn from_str_and_string_comparisons() {
        let duration = "1m30s".parse::<HumanDuration>().unwrap();
        let owned = "90s".to_string();
        assert_eq!(duration, "1m30s");
        assert_eq!(duration, owned);
        assert!("1s1m".parse::<HumanDuration>().is_err());
    }
}

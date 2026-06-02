use crate::{PrimitiveError, PrimitiveResult};
use alloc::string::{String, ToString};
use core::fmt;

/// Semantic version in the form `MAJOR.MINOR.PATCH` with optional pre-release
/// and build metadata identifiers.
///
/// Parses `1.2.3`, `1.2.3-beta.1`, `1.2.3+build.456`, and
/// `1.2.3-alpha.1+build.456`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
    build: Option<String>,
}

impl SemVer {
    /// Creates a `SemVer` with no pre-release or build metadata.
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
            build: None,
        }
    }

    /// Parses a semver string.
    pub fn parse(s: &str) -> PrimitiveResult<Self> {
        if s.is_empty() {
            return Err(PrimitiveError::Empty);
        }

        let (s, build) = if let Some(idx) = s.find('+') {
            let b = s[idx + 1..].to_string();
            if b.is_empty() {
                return Err(PrimitiveError::Invalid {
                    message: "build metadata must not be empty after '+'",
                });
            }
            (&s[..idx], Some(b))
        } else {
            (s, None)
        };

        let (s, pre) = if let Some(idx) = s.find('-') {
            let p = s[idx + 1..].to_string();
            if p.is_empty() {
                return Err(PrimitiveError::Invalid {
                    message: "pre-release identifier must not be empty after '-'",
                });
            }
            (&s[..idx], Some(p))
        } else {
            (s, None)
        };

        let mut parts = s.splitn(4, '.');
        let major = parse_version_component(parts.next().unwrap_or(""))?;
        let minor = parse_version_component(parts.next().unwrap_or(""))?;
        let patch = parse_version_component(parts.next().unwrap_or(""))?;

        if parts.next().is_some() {
            return Err(PrimitiveError::Invalid {
                message: "semver must have exactly three dot-separated components",
            });
        }

        Ok(Self {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }

    pub fn major(&self) -> u64 {
        self.major
    }
    pub fn minor(&self) -> u64 {
        self.minor
    }
    pub fn patch(&self) -> u64 {
        self.patch
    }

    /// Returns the pre-release identifier if present.
    pub fn pre(&self) -> Option<&str> {
        self.pre.as_deref()
    }

    /// Returns the build metadata if present.
    pub fn build(&self) -> Option<&str> {
        self.build.as_deref()
    }

    /// Returns `true` if this is a pre-release version.
    pub fn is_pre_release(&self) -> bool {
        self.pre.is_some()
    }
}

fn parse_version_component(s: &str) -> PrimitiveResult<u64> {
    if s.is_empty() {
        return Err(PrimitiveError::Invalid {
            message: "semver component must not be empty",
        });
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(PrimitiveError::Invalid {
            message: "semver component must not have leading zeros",
        });
    }
    parse_u64(s).ok_or(PrimitiveError::Invalid {
        message: "semver component must be a non-negative integer",
    })
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

impl fmt::Display for SemVer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{pre}")?;
        }
        if let Some(build) = &self.build {
            write!(f, "+{build}")?;
        }
        Ok(())
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let v = (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));
        if v != core::cmp::Ordering::Equal {
            return v;
        }
        // Per semver spec §11: pre-release < release when core version is equal.
        match (&self.pre, &other.pre) {
            (None, None) => core::cmp::Ordering::Equal,
            (Some(_), None) => core::cmp::Ordering::Less,
            (None, Some(_)) => core::cmp::Ordering::Greater,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SemVer;
    use crate::PrimitiveError;
    use alloc::string::ToString;

    #[test]
    fn parses_simple() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major(), 1);
        assert_eq!(v.minor(), 2);
        assert_eq!(v.patch(), 3);
        assert!(v.pre().is_none());
        assert!(v.build().is_none());
    }

    #[test]
    fn parses_with_pre_release() {
        let v = SemVer::parse("2.0.0-beta.1").unwrap();
        assert_eq!(v.pre(), Some("beta.1"));
        assert!(v.is_pre_release());
    }

    #[test]
    fn parses_with_build() {
        let v = SemVer::parse("1.0.0+build.456").unwrap();
        assert_eq!(v.build(), Some("build.456"));
    }

    #[test]
    fn parses_pre_and_build() {
        let v = SemVer::parse("1.0.0-alpha.1+build.001").unwrap();
        assert_eq!(v.pre(), Some("alpha.1"));
        assert_eq!(v.build(), Some("build.001"));
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(SemVer::parse("").unwrap_err(), PrimitiveError::Empty);
    }

    #[test]
    fn rejects_missing_components() {
        assert!(SemVer::parse("1.2").is_err());
    }

    #[test]
    fn rejects_too_many_components() {
        assert!(SemVer::parse("1.2.3.4").is_err());
    }

    #[test]
    fn rejects_leading_zeros() {
        assert!(SemVer::parse("1.02.3").is_err());
    }

    #[test]
    fn rejects_non_numeric() {
        assert!(SemVer::parse("a.b.c").is_err());
    }

    #[test]
    fn rejects_empty_pre_release() {
        assert!(SemVer::parse("1.0.0-").is_err());
    }

    #[test]
    fn rejects_empty_build() {
        assert!(SemVer::parse("1.0.0+").is_err());
    }

    #[test]
    fn display() {
        assert_eq!(SemVer::parse("1.2.3").unwrap().to_string(), "1.2.3");
        assert_eq!(
            SemVer::parse("2.0.0-beta.1").unwrap().to_string(),
            "2.0.0-beta.1"
        );
        assert_eq!(
            SemVer::parse("1.0.0+build").unwrap().to_string(),
            "1.0.0+build"
        );
        assert_eq!(
            SemVer::parse("1.0.0-alpha+build").unwrap().to_string(),
            "1.0.0-alpha+build"
        );
    }

    #[test]
    fn new_constructor() {
        let v = SemVer::new(1, 0, 0);
        assert_eq!(v.to_string(), "1.0.0");
    }

    #[test]
    fn ordering() {
        let v1 = SemVer::parse("1.0.0").unwrap();
        let v2 = SemVer::parse("2.0.0").unwrap();
        let v3 = SemVer::parse("1.1.0").unwrap();
        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v3 < v2);
    }

    #[test]
    fn pre_release_sorts_below_release() {
        let release = SemVer::parse("1.0.0").unwrap();
        let pre = SemVer::parse("1.0.0-alpha").unwrap();
        assert!(pre < release);
        assert!(release > pre);
    }

    #[test]
    fn pre_release_compared_lexicographically() {
        let alpha = SemVer::parse("1.0.0-alpha").unwrap();
        let beta = SemVer::parse("1.0.0-beta").unwrap();
        assert!(alpha < beta);
    }
}

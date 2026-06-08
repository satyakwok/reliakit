use core::fmt;

/// The health of a component or a whole service.
///
/// The three states are ordered by severity: `Healthy < Degraded < Unhealthy`.
/// That ordering is the whole point — aggregating a set of statuses is just
/// taking the [worst](Health::worst) (maximum) one, so `Ord` does the work.
///
/// - `Healthy` — fully operational.
/// - `Degraded` — operational but impaired (serving with reduced capacity,
///   higher latency, or a non-critical dependency down).
/// - `Unhealthy` — not able to serve.
///
/// This is a deliberately closed enum: callers are expected to match all three
/// states (e.g. to map to HTTP codes), so it is **not** `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Health {
    /// Fully operational.
    Healthy,
    /// Operational but impaired.
    Degraded,
    /// Not able to serve.
    Unhealthy,
}

impl Health {
    /// Maps `true` to [`Healthy`](Health::Healthy) and `false` to
    /// [`Unhealthy`](Health::Unhealthy).
    ///
    /// Handy for turning a boolean probe (a ping, a connection test) into a
    /// status. Use [`from_ok_or`](Health::from_ok_or) when a failure should only
    /// degrade.
    pub const fn from_ok(ok: bool) -> Self {
        if ok {
            Self::Healthy
        } else {
            Self::Unhealthy
        }
    }

    /// Maps `true` to [`Healthy`](Health::Healthy) and `false` to `on_failure`.
    pub const fn from_ok_or(ok: bool, on_failure: Health) -> Self {
        if ok {
            Self::Healthy
        } else {
            on_failure
        }
    }

    /// Returns the more severe (worse) of two statuses.
    pub const fn worst(self, other: Health) -> Health {
        if self.severity() >= other.severity() {
            self
        } else {
            other
        }
    }

    /// Returns the less severe (better) of two statuses.
    pub const fn best(self, other: Health) -> Health {
        if self.severity() <= other.severity() {
            self
        } else {
            other
        }
    }

    /// Caps the status at `ceiling` — a status more severe than `ceiling` is
    /// lowered to `ceiling`.
    ///
    /// This expresses "this signal can degrade the service but must not be able
    /// to bring it down", e.g. `status.capped_at(Health::Degraded)`.
    pub const fn capped_at(self, ceiling: Health) -> Health {
        self.best(ceiling)
    }

    /// Returns `true` only when [`Healthy`](Health::Healthy).
    pub const fn is_healthy(self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Returns `true` when [`Degraded`](Health::Degraded).
    pub const fn is_degraded(self) -> bool {
        matches!(self, Self::Degraded)
    }

    /// Returns `true` when [`Unhealthy`](Health::Unhealthy).
    pub const fn is_unhealthy(self) -> bool {
        matches!(self, Self::Unhealthy)
    }

    /// Returns `true` when the service can still serve — `Healthy` or
    /// `Degraded`, i.e. not `Unhealthy`.
    ///
    /// This is the usual readiness check: a degraded service still takes
    /// traffic.
    pub const fn is_operational(self) -> bool {
        !self.is_unhealthy()
    }

    /// Returns the lowercase string form (`"healthy"`, `"degraded"`,
    /// `"unhealthy"`), matching [`Display`](Health).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
        }
    }

    /// Severity rank: `0` healthy, `1` degraded, `2` unhealthy.
    const fn severity(self) -> u8 {
        match self {
            Self::Healthy => 0,
            Self::Degraded => 1,
            Self::Unhealthy => 2,
        }
    }
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How much a component's health matters to the overall result.
///
/// Used by [`Check`](crate::Check) and [`HealthReport`](crate::HealthReport) to
/// decide how a failing component affects the aggregate.
///
/// Closed enum (not `#[non_exhaustive]`); the default is [`Critical`](Criticality::Critical).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Criticality {
    /// The component's status contributes to the aggregate unchanged: if it is
    /// [`Unhealthy`](Health::Unhealthy), the whole service is `Unhealthy`.
    #[default]
    Critical,
    /// The component's failure is capped at [`Degraded`](Health::Degraded): an
    /// `Optional` component going down degrades the service but cannot bring it
    /// down on its own.
    Optional,
}

impl Criticality {
    /// Applies this criticality to a raw status, returning the status it
    /// actually contributes to the aggregate.
    ///
    /// `Critical` returns `status` unchanged; `Optional` caps it at
    /// [`Degraded`](Health::Degraded).
    pub const fn apply(self, status: Health) -> Health {
        match self {
            Self::Critical => status,
            Self::Optional => status.capped_at(Health::Degraded),
        }
    }

    /// Returns `true` for [`Critical`](Criticality::Critical).
    pub const fn is_critical(self) -> bool {
        matches!(self, Self::Critical)
    }
}

#[cfg(test)]
mod tests {
    use super::{Criticality, Health};

    #[test]
    fn severity_ordering() {
        assert!(Health::Healthy < Health::Degraded);
        assert!(Health::Degraded < Health::Unhealthy);
    }

    #[test]
    fn worst_and_best() {
        assert_eq!(Health::Healthy.worst(Health::Degraded), Health::Degraded);
        assert_eq!(Health::Unhealthy.worst(Health::Degraded), Health::Unhealthy);
        assert_eq!(Health::Healthy.best(Health::Unhealthy), Health::Healthy);
        assert_eq!(Health::Degraded.best(Health::Degraded), Health::Degraded);
    }

    #[test]
    fn capped_at() {
        assert_eq!(
            Health::Unhealthy.capped_at(Health::Degraded),
            Health::Degraded
        );
        assert_eq!(Health::Healthy.capped_at(Health::Degraded), Health::Healthy);
        assert_eq!(
            Health::Degraded.capped_at(Health::Unhealthy),
            Health::Degraded
        );
    }

    #[test]
    fn predicates() {
        assert!(Health::Healthy.is_healthy());
        assert!(!Health::Degraded.is_healthy());
        assert!(Health::Degraded.is_degraded());
        assert!(!Health::Healthy.is_degraded());
        assert!(Health::Healthy.is_operational());
        assert!(Health::Degraded.is_operational());
        assert!(!Health::Unhealthy.is_operational());
        assert!(Health::Unhealthy.is_unhealthy());
        assert!(!Health::Healthy.is_unhealthy());
    }

    #[test]
    fn from_ok() {
        assert_eq!(Health::from_ok(true), Health::Healthy);
        assert_eq!(Health::from_ok(false), Health::Unhealthy);
        assert_eq!(
            Health::from_ok_or(false, Health::Degraded),
            Health::Degraded
        );
        assert_eq!(Health::from_ok_or(true, Health::Degraded), Health::Healthy);
    }

    #[test]
    fn display_and_as_str() {
        assert_eq!(Health::Healthy.as_str(), "healthy");
        assert_eq!(Health::Degraded.as_str(), "degraded");
        assert_eq!(Health::Unhealthy.as_str(), "unhealthy");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn display_matches_as_str() {
        use alloc::format;
        assert_eq!(format!("{}", Health::Healthy), "healthy");
        assert_eq!(format!("{}", Health::Degraded), "degraded");
        assert_eq!(format!("{}", Health::Unhealthy), "unhealthy");
    }

    #[test]
    fn criticality_apply() {
        assert_eq!(Criticality::default(), Criticality::Critical);
        assert_eq!(
            Criticality::Critical.apply(Health::Unhealthy),
            Health::Unhealthy
        );
        assert_eq!(
            Criticality::Optional.apply(Health::Unhealthy),
            Health::Degraded
        );
        assert_eq!(
            Criticality::Optional.apply(Health::Healthy),
            Health::Healthy
        );
        assert!(Criticality::Critical.is_critical());
        assert!(!Criticality::Optional.is_critical());
    }
}

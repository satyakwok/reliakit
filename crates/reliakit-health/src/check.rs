use crate::{Criticality, Health};

/// A single named health check, borrowing its strings.
///
/// `Check` is a small `Copy` value that pairs a component name with its
/// [`Health`], a [`Criticality`], and an optional detail message. It borrows its
/// `&str` fields, so it needs no allocation and works in `no_std` without
/// `alloc` — build a fixed array of `Check`s and hand them to [`aggregate`].
///
/// For a dynamically built, owned collection use
/// [`HealthReport`](crate::HealthReport) (requires `alloc`).
///
/// # Example
///
/// ```
/// use reliakit_health::{aggregate, Check, Health};
///
/// let checks = [
///     Check::new("database", Health::Healthy),
///     Check::new("cache", Health::Unhealthy).optional().with_detail("primary down"),
/// ];
///
/// // The cache is optional, so its failure only degrades the service.
/// assert_eq!(aggregate(checks), Health::Degraded);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Check<'a> {
    /// The component name.
    pub name: &'a str,
    /// The reported status.
    pub status: Health,
    /// How the status contributes to an aggregate.
    pub criticality: Criticality,
    /// Optional human-readable detail (a reason, a metric).
    pub detail: Option<&'a str>,
}

impl<'a> Check<'a> {
    /// Creates a `Critical` check with no detail.
    pub const fn new(name: &'a str, status: Health) -> Self {
        Self {
            name,
            status,
            criticality: Criticality::Critical,
            detail: None,
        }
    }

    /// Marks the check [`Critical`](Criticality::Critical) (the default).
    pub const fn critical(mut self) -> Self {
        self.criticality = Criticality::Critical;
        self
    }

    /// Marks the check [`Optional`](Criticality::Optional): its failure is capped
    /// at [`Degraded`](Health::Degraded).
    pub const fn optional(mut self) -> Self {
        self.criticality = Criticality::Optional;
        self
    }

    /// Attaches a detail message.
    pub const fn with_detail(mut self, detail: &'a str) -> Self {
        self.detail = Some(detail);
        self
    }

    /// Returns the status this check actually contributes to an aggregate, after
    /// applying its [`Criticality`].
    pub const fn effective(self) -> Health {
        self.criticality.apply(self.status)
    }
}

/// Aggregates an iterator of [`Check`]s into a single [`Health`].
///
/// The result is the worst (most severe) [effective](Check::effective) status,
/// so a `Critical` `Unhealthy` makes the whole result `Unhealthy` while an
/// `Optional` `Unhealthy` only degrades it. An empty input is
/// [`Healthy`](Health::Healthy) — nothing is wrong.
///
/// # Example
///
/// ```
/// use reliakit_health::{aggregate, Check, Health};
///
/// let all_good = [Check::new("a", Health::Healthy), Check::new("b", Health::Healthy)];
/// assert_eq!(aggregate(all_good), Health::Healthy);
///
/// let one_down = [Check::new("a", Health::Healthy), Check::new("b", Health::Unhealthy)];
/// assert_eq!(aggregate(one_down), Health::Unhealthy);
/// ```
pub fn aggregate<'a, I>(checks: I) -> Health
where
    I: IntoIterator<Item = Check<'a>>,
{
    checks
        .into_iter()
        .fold(Health::Healthy, |acc, check| acc.worst(check.effective()))
}

#[cfg(test)]
mod tests {
    use super::{aggregate, Check};
    use crate::{Criticality, Health};

    #[test]
    fn builder_sets_fields() {
        let c = Check::new("db", Health::Degraded)
            .optional()
            .with_detail("slow");
        assert_eq!(c.name, "db");
        assert_eq!(c.status, Health::Degraded);
        assert_eq!(c.criticality, Criticality::Optional);
        assert_eq!(c.detail, Some("slow"));
    }

    #[test]
    fn effective_applies_criticality() {
        assert_eq!(
            Check::new("x", Health::Unhealthy).critical().effective(),
            Health::Unhealthy
        );
        assert_eq!(
            Check::new("x", Health::Unhealthy).optional().effective(),
            Health::Degraded
        );
    }

    #[test]
    fn aggregate_empty_is_healthy() {
        let none: [Check; 0] = [];
        assert_eq!(aggregate(none), Health::Healthy);
    }

    #[test]
    fn aggregate_takes_worst_effective() {
        let checks = [
            Check::new("a", Health::Healthy),
            Check::new("b", Health::Degraded),
            Check::new("c", Health::Healthy),
        ];
        assert_eq!(aggregate(checks), Health::Degraded);
    }

    #[test]
    fn aggregate_critical_unhealthy_downs_everything() {
        let checks = [
            Check::new("a", Health::Healthy),
            Check::new("b", Health::Unhealthy),
        ];
        assert_eq!(aggregate(checks), Health::Unhealthy);
    }

    #[test]
    fn aggregate_optional_unhealthy_only_degrades() {
        let checks = [
            Check::new("db", Health::Healthy),
            Check::new("cache", Health::Unhealthy).optional(),
        ];
        assert_eq!(aggregate(checks), Health::Degraded);
    }

    #[test]
    fn aggregate_critical_beats_optional() {
        // An optional failure degrades, but a critical failure still downs it.
        let checks = [
            Check::new("cache", Health::Unhealthy).optional(),
            Check::new("db", Health::Unhealthy).critical(),
        ];
        assert_eq!(aggregate(checks), Health::Unhealthy);
    }
}

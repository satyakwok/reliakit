use crate::{Criticality, Health};
use alloc::string::String;
use alloc::vec::Vec;

/// An owned, named component health record held by a [`HealthReport`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Component {
    /// The component name.
    pub name: String,
    /// The reported status.
    pub status: Health,
    /// How the status contributes to the aggregate.
    pub criticality: Criticality,
    /// Optional human-readable detail (a reason, a metric).
    pub detail: Option<String>,
}

impl Component {
    /// Returns the status this component contributes to the aggregate, after
    /// applying its [`Criticality`].
    pub fn effective(&self) -> Health {
        self.criticality.apply(self.status)
    }
}

/// Per-status counts of the components in a [`HealthReport`].
///
/// Counts use each component's **raw** reported status (what a status page
/// shows), not the criticality-adjusted one used by
/// [`HealthReport::overall`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Summary {
    /// Number of components reported [`Healthy`](Health::Healthy).
    pub healthy: usize,
    /// Number of components reported [`Degraded`](Health::Degraded).
    pub degraded: usize,
    /// Number of components reported [`Unhealthy`](Health::Unhealthy).
    pub unhealthy: usize,
}

impl Summary {
    /// Total number of components counted.
    pub const fn total(&self) -> usize {
        self.healthy + self.degraded + self.unhealthy
    }
}

/// A dynamically built collection of component health records that rolls up to
/// one overall [`Health`].
///
/// Build it from your own checks, then ask for [`overall`](Self::overall) (the
/// criticality-aware worst-case status) for a `/health` or readiness endpoint,
/// a [`summary`](Self::summary) of counts for a status page, or the
/// [`reasons`](Self::reasons) behind any non-healthy component.
///
/// Requires the `alloc` feature (enabled by default via `std`). For an
/// allocation-free aggregate over a fixed array, use
/// [`aggregate`](crate::aggregate) with [`Check`](crate::Check) instead.
///
/// # Example
///
/// ```
/// use reliakit_health::{Health, HealthReport};
///
/// let report = HealthReport::new()
///     .critical("database", Health::Healthy)
///     .optional("cache", Health::Unhealthy)
///     .critical("queue", Health::Degraded);
///
/// // database ok, queue degraded, cache down-but-optional -> degraded overall.
/// assert_eq!(report.overall(), Health::Degraded);
/// assert!(report.is_operational());
///
/// let summary = report.summary();
/// assert_eq!(summary.healthy, 1);
/// assert_eq!(summary.degraded, 1);
/// assert_eq!(summary.unhealthy, 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HealthReport {
    components: Vec<Component>,
}

impl HealthReport {
    /// Creates an empty report. An empty report is [`Healthy`](Health::Healthy).
    pub const fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    /// Adds a component with an explicit criticality, returning `self` for
    /// chaining.
    pub fn with(
        mut self,
        name: impl Into<String>,
        status: Health,
        criticality: Criticality,
    ) -> Self {
        self.push(name, status, criticality);
        self
    }

    /// Adds a [`Critical`](Criticality::Critical) component, returning `self`.
    pub fn critical(self, name: impl Into<String>, status: Health) -> Self {
        self.with(name, status, Criticality::Critical)
    }

    /// Adds an [`Optional`](Criticality::Optional) component, returning `self`.
    pub fn optional(self, name: impl Into<String>, status: Health) -> Self {
        self.with(name, status, Criticality::Optional)
    }

    /// Adds a component in place (non-consuming).
    pub fn push(&mut self, name: impl Into<String>, status: Health, criticality: Criticality) {
        self.components.push(Component {
            name: name.into(),
            status,
            criticality,
            detail: None,
        });
    }

    /// Attaches a detail message to the most recently added component.
    ///
    /// No-op if the report is empty. Chains after a `with`/`critical`/`optional`
    /// call.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        if let Some(last) = self.components.last_mut() {
            last.detail = Some(detail.into());
        }
        self
    }

    /// Returns the overall status: the worst
    /// [effective](Component::effective) status across all components, with
    /// criticality applied. An empty report is [`Healthy`](Health::Healthy).
    pub fn overall(&self) -> Health {
        self.components
            .iter()
            .fold(Health::Healthy, |acc, c| acc.worst(c.effective()))
    }

    /// Returns per-status counts of the components (by raw status).
    pub fn summary(&self) -> Summary {
        let mut s = Summary::default();
        for c in &self.components {
            match c.status {
                Health::Healthy => s.healthy += 1,
                Health::Degraded => s.degraded += 1,
                Health::Unhealthy => s.unhealthy += 1,
            }
        }
        s
    }

    /// Returns `true` if the overall status is operational (not `Unhealthy`).
    pub fn is_operational(&self) -> bool {
        self.overall().is_operational()
    }

    /// Returns `true` if the overall status is `Healthy`.
    pub fn is_healthy(&self) -> bool {
        self.overall().is_healthy()
    }

    /// Returns the components in insertion order.
    pub fn components(&self) -> &[Component] {
        &self.components
    }

    /// Iterates the components whose **raw** status equals `status`.
    pub fn by_status(&self, status: Health) -> impl Iterator<Item = &Component> {
        self.components.iter().filter(move |c| c.status == status)
    }

    /// Iterates `(name, detail)` for every component that is not
    /// [`Healthy`](Health::Healthy), in insertion order.
    ///
    /// This is the list to surface in a status page or an alert.
    pub fn reasons(&self) -> impl Iterator<Item = (&str, Option<&str>)> {
        self.components
            .iter()
            .filter(|c| !c.status.is_healthy())
            .map(|c| (c.name.as_str(), c.detail.as_deref()))
    }

    /// Returns the number of components.
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Returns `true` if the report has no components.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{HealthReport, Summary};
    use crate::{Criticality, Health};
    use alloc::vec::Vec;

    #[test]
    fn empty_report_is_healthy() {
        let r = HealthReport::new();
        assert!(r.is_empty());
        assert_eq!(r.overall(), Health::Healthy);
        assert!(r.is_healthy());
        assert!(r.is_operational());
        assert_eq!(r.summary(), Summary::default());
    }

    #[test]
    fn all_healthy() {
        let r = HealthReport::new()
            .critical("a", Health::Healthy)
            .critical("b", Health::Healthy);
        assert_eq!(r.overall(), Health::Healthy);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn critical_unhealthy_downs_service() {
        let r = HealthReport::new()
            .critical("db", Health::Unhealthy)
            .optional("cache", Health::Healthy);
        assert_eq!(r.overall(), Health::Unhealthy);
        assert!(!r.is_operational());
    }

    #[test]
    fn optional_unhealthy_only_degrades() {
        let r = HealthReport::new()
            .critical("db", Health::Healthy)
            .optional("cache", Health::Unhealthy);
        assert_eq!(r.overall(), Health::Degraded);
        assert!(r.is_operational());
        assert!(!r.is_healthy());
    }

    #[test]
    fn summary_counts_raw_status() {
        let r = HealthReport::new()
            .critical("a", Health::Healthy)
            .critical("b", Health::Degraded)
            .optional("c", Health::Unhealthy)
            .critical("d", Health::Healthy);
        let s = r.summary();
        assert_eq!(s.healthy, 2);
        assert_eq!(s.degraded, 1);
        assert_eq!(s.unhealthy, 1);
        assert_eq!(s.total(), 4);
    }

    #[test]
    fn detail_attaches_to_last() {
        let r = HealthReport::new()
            .critical("db", Health::Healthy)
            .optional("cache", Health::Unhealthy)
            .detail("primary node down");
        let reasons: Vec<(&str, Option<&str>)> = r.reasons().collect();
        assert_eq!(reasons, [("cache", Some("primary node down"))]);
    }

    #[test]
    fn detail_on_empty_is_noop() {
        let r = HealthReport::new().detail("nothing to attach to");
        assert!(r.is_empty());
    }

    #[test]
    fn reasons_lists_non_healthy_in_order() {
        let r = HealthReport::new()
            .critical("a", Health::Healthy)
            .critical("b", Health::Unhealthy)
            .critical("c", Health::Degraded);
        let names: Vec<&str> = r.reasons().map(|(n, _)| n).collect();
        assert_eq!(names, ["b", "c"]);
    }

    #[test]
    fn by_status_filters() {
        let r = HealthReport::new()
            .critical("a", Health::Healthy)
            .critical("b", Health::Unhealthy)
            .critical("c", Health::Healthy);
        let healthy: Vec<&str> = r
            .by_status(Health::Healthy)
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(healthy, ["a", "c"]);
        assert_eq!(r.by_status(Health::Degraded).count(), 0);
    }

    #[test]
    fn push_in_place_with_explicit_criticality() {
        let mut r = HealthReport::new();
        r.push("svc", Health::Degraded, Criticality::Critical);
        assert_eq!(r.len(), 1);
        assert_eq!(r.overall(), Health::Degraded);
    }

    #[test]
    fn components_accessor_and_effective() {
        let r = HealthReport::new().optional("cache", Health::Unhealthy);
        let c = &r.components()[0];
        assert_eq!(c.status, Health::Unhealthy);
        assert_eq!(c.effective(), Health::Degraded);
    }
}

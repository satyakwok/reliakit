//! Health status types and a criticality-aware aggregator.
//!
//! `reliakit-health` answers one question: *given the state of my components,
//! what is the overall health of the service?* It is plain data: it does not
//! run checks, read the clock, or perform I/O. You report each component's
//! status; it rolls them up into one [`Health`], applying per-component
//! [`Criticality`] so a non-critical dependency going down *degrades* the
//! service instead of *downing* it.
//!
//! Typical homes for it: a `/health` or `/readyz` endpoint, a Kubernetes
//! readiness/liveness probe, or a status page.
//!
//! # Two ways to aggregate
//!
//! - Allocation-free: build a fixed array of [`Check`]s (which borrow their
//!   strings) and call [`aggregate`]. Works in `no_std` without `alloc`.
//! - Owned and dynamic: build a [`HealthReport`] with the
//!   `critical`/`optional`/`with` builder, then ask for
//!   [`overall`](HealthReport::overall), a [`summary`](HealthReport::summary),
//!   or the [`reasons`](HealthReport::reasons). Requires `alloc`.
//!
//! # Roll-up rules
//!
//! The overall status is the worst (most severe) effective status, where
//! `Healthy < Degraded < Unhealthy`. A `Critical` component contributes its
//! status unchanged; an `Optional` component's status is capped at `Degraded`.
//! An empty set is `Healthy`.
//!
//! # Example
//!
//! ```
//! use reliakit_health::{Health, HealthReport};
//!
//! let report = HealthReport::new()
//!     .critical("database", Health::Healthy)
//!     .optional("cache", Health::Unhealthy) // non-critical: only degrades
//!     .critical("queue", Health::Degraded)
//!     .detail("redelivery backlog");
//!
//! let overall = report.overall();
//! assert_eq!(overall, Health::Degraded);
//!
//! // Map to an HTTP status for a /health endpoint.
//! let http = if overall.is_operational() { 200 } else { 503 };
//! assert_eq!(http, 200);
//! ```
//!
//! # Composing with the other resilience crates
//!
//! The other `reliakit-*` resilience crates produce signals this crate
//! summarizes: a tripped circuit breaker maps to an `Unhealthy` (or `Degraded`)
//! component, a full bulkhead or rate limiter to `Degraded`, an expired deadline
//! to `Degraded`. `reliakit-health` only reports; it never changes behavior.
//!
//! # Feature flags
//!
//! - `std` (default) enables the standard library and implies `alloc`.
//! - `alloc` enables [`HealthReport`], [`Component`], and [`Summary`]. The
//!   [`Health`], [`Criticality`], and [`Check`] types and [`aggregate`] need
//!   neither.
//!
//! # `no_std`
//!
//! The crate is `no_std`-friendly. With `--no-default-features` you get the
//! allocation-free core ([`Health`], [`Criticality`], [`Check`], [`aggregate`]);
//! add `alloc` for the owned [`HealthReport`].

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod check;
mod health;
#[cfg(feature = "alloc")]
mod report;

pub use check::{Check, aggregate};
pub use health::{Criticality, Health};
#[cfg(feature = "alloc")]
pub use report::{Component, HealthReport, Summary};

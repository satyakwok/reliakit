#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Umbrella crate for the Reliakit reliability toolkit.
//!
//! This crate has no logic of its own. It re-exports the individual `reliakit-*`
//! crates behind feature flags so you can depend on a single name and enable
//! only the building blocks you need. Nothing is pulled in by default beyond the
//! `std` flag; each module below appears only when its feature is enabled.
//!
//! Add it and pick the pieces you want:
//!
//! ```toml
//! reliakit = { version = "0.2", features = ["ratelimit", "secret"] }
//! ```
//!
//! ```
//! # // Requires the `ratelimit` and `secret` features (both on under `full`).
//! use reliakit::ratelimit::RateLimiter;
//! use reliakit::secret::SecretString;
//!
//! let mut limiter = RateLimiter::new(5, 1, 1);
//! assert!(limiter.try_acquire_one(0));
//!
//! let api_key = SecretString::from_string("rk_live_value");
//! assert_eq!(format!("{api_key}"), "[REDACTED]");
//! ```
//!
//! For `no_std`, disable default features and add `alloc` where a module needs
//! owned storage:
//!
//! ```toml
//! reliakit = { version = "0.2", default-features = false, features = ["alloc", "primitives"] }
//! ```
//!
//! # Features
//!
//! | Feature | Re-exports |
//! |---|---|
//! | `core` | [`reliakit_core`] as [`core`]; also enables the clock-aware `*_now` methods of any enabled resilience crate |
//! | `primitives` | [`reliakit_primitives`] as [`primitives`] |
//! | `secret` | [`reliakit_secret`] as [`secret`] |
//! | `validate` | [`reliakit_validate`] as [`validate`] |
//! | `collections` | [`reliakit_collections`] as [`collections`] |
//! | `codec` | [`reliakit_codec`] as [`codec`] |
//! | `csv` | [`reliakit_csv`] as [`csv`] |
//! | `backoff` | [`reliakit_backoff`] as [`backoff`] |
//! | `retry` | [`reliakit_retry`] as [`retry`] |
//! | `bulkhead` | [`reliakit_bulkhead`] as [`bulkhead`] |
//! | `health` | [`reliakit_health`] as [`health`] |
//! | `circuit` | [`reliakit_circuit`] as [`circuit`] |
//! | `ratelimit` | [`reliakit_ratelimit`] as [`ratelimit`] |
//! | `timeout` | [`reliakit_timeout`] as [`timeout`] |
//! | `json` | [`reliakit_json`] as [`json`] |
//! | `derive` | [`reliakit_derive`] as [`mod@derive`] |
//! | `decide` | [`reliakit_decide`] as [`decide`] |
//! | `full` | all of the above |
//!
//! `std` (on by default) implies `alloc`; both forward to the enabled crates.
//! The integration features `json-canonical`, `json-primitives`, `json-validate`,
//! and `codec-primitives` turn on the matching cross-crate features.

#[cfg(feature = "core")]
pub use reliakit_core as core;

#[cfg(feature = "primitives")]
pub use reliakit_primitives as primitives;

#[cfg(feature = "secret")]
pub use reliakit_secret as secret;

#[cfg(feature = "validate")]
pub use reliakit_validate as validate;

#[cfg(feature = "collections")]
pub use reliakit_collections as collections;

#[cfg(feature = "codec")]
pub use reliakit_codec as codec;

#[cfg(feature = "csv")]
pub use reliakit_csv as csv;

#[cfg(feature = "backoff")]
pub use reliakit_backoff as backoff;

#[cfg(feature = "retry")]
pub use reliakit_retry as retry;

#[cfg(feature = "bulkhead")]
pub use reliakit_bulkhead as bulkhead;

#[cfg(feature = "health")]
pub use reliakit_health as health;

#[cfg(feature = "circuit")]
pub use reliakit_circuit as circuit;

#[cfg(feature = "ratelimit")]
pub use reliakit_ratelimit as ratelimit;

#[cfg(feature = "timeout")]
pub use reliakit_timeout as timeout;

#[cfg(feature = "json")]
pub use reliakit_json as json;

#[cfg(feature = "derive")]
pub use reliakit_derive as derive;

#[cfg(feature = "decide")]
pub use reliakit_decide as decide;

//! Bounded and reliability-oriented collection types for Rust.
//!
//! `reliakit-collections` provides collection types with enforced size
//! constraints. The bounds are expressed as const generic parameters and
//! checked at construction time. Mutations that would violate the bounds
//! return errors rather than panicking.
//!
//! # Types
//!
//! - [`BoundedVec<T, MIN, MAX>`], an owned `Vec<T>` constrained to hold
//!   between `MIN` and `MAX` elements inclusive.
//! - [`BoundedMap<K, V, MIN, MAX>`], an insertion-ordered key-value map with
//!   unique keys and an enforced entry-count range.
//! - [`BoundedSet<T, MIN, MAX>`], an insertion-ordered set of unique elements
//!   with an enforced count range.
//! - [`RingBuffer<T>`], a fixed-capacity circular buffer that overwrites the
//!   oldest element when full (a rolling window that never fails to push).
//!
//! [`BoundedMap`] and [`BoundedSet`] are backed by a `Vec` and use linear scans
//! for lookup, so they stay deterministic and dependency-free (no hashing or
//! ordering machinery) and are meant for the small, bounded sizes their bounds
//! describe.
//!
//! # Examples
//!
//! ```
//! use reliakit_collections::BoundedVec;
//!
//! // A list that must have between 1 and 10 recipients
//! type RecipientList = BoundedVec<String, 1, 10>;
//!
//! let mut recipients = RecipientList::new(vec!["alice@example.com".into()]).unwrap();
//! recipients.push("bob@example.com".into()).unwrap();
//! assert_eq!(recipients.len(), 2);
//! ```
//!
//! Mutations that would violate bounds are rejected:
//!
//! ```
//! use reliakit_collections::BoundedVec;
//!
//! let mut v = BoundedVec::<i32, 1, 2>::new(vec![1, 2]).unwrap();
//! assert!(v.push(3).is_err()); // at capacity
//! assert!(v.pop().is_ok());    // still above minimum
//! assert!(v.pop().is_err());   // would go below minimum
//! ```
//!
//! # Feature flags
//!
//! - `std` (default) enables `std::error::Error` for [`CollectionError`] and
//!   implies `alloc`.
//! - `alloc` enables [`BoundedVec`], which is backed by `Vec<T>`.
//!
//! # `no_std`
//!
//! The crate supports `no_std`. [`BoundedVec`] requires the `alloc` feature
//! (enabled by default via `std`). The error types ([`CollectionError`],
//! [`CollectionResult`]) are available without `alloc`.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod bounded_map;
#[cfg(feature = "alloc")]
mod bounded_set;
#[cfg(feature = "alloc")]
mod bounded_vec;
mod error;
#[cfg(feature = "alloc")]
mod ring_buffer;

#[cfg(feature = "alloc")]
pub use bounded_map::BoundedMap;
#[cfg(feature = "alloc")]
pub use bounded_set::BoundedSet;
#[cfg(feature = "alloc")]
pub use bounded_vec::BoundedVec;
pub use error::{CollectionError, CollectionResult};
#[cfg(feature = "alloc")]
pub use ring_buffer::RingBuffer;

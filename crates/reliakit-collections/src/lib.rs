//! Bounded and reliability-oriented collection types for Rust.
//!
//! `reliakit-collections` provides collection types with enforced size
//! constraints. The bounds are expressed as const generic parameters and
//! checked at construction time. Mutations that would violate the bounds
//! return errors rather than panicking.
//!
//! # Types
//!
//! - [`BoundedVec<T, MIN, MAX>`] — an owned `Vec<T>` constrained to hold
//!   between `MIN` and `MAX` elements inclusive.
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

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod bounded_vec;
mod error;

pub use bounded_vec::BoundedVec;
pub use error::{CollectionError, CollectionResult};

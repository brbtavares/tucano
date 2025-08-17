// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! # Collection Module
//!
//! This module provides specialized collection types used throughout the integration crate
//! for handling variable-cardinality data structures common in financial trading systems.
//!
//! ## Core Types
//!
//! - [`OneOrMany<T>`] - Represents exactly one or multiple items (never empty)
//! - [`NoneOneOrMany<T>`] - Represents zero, one, or multiple items (can be empty)
//! - [`FnvIndexMap<K, V>`] - Fast hash map using FNV hasher for better performance
//! - [`FnvIndexSet<T>`] - Fast hash set using FNV hasher for better performance
//!
//! ## Use Cases in Trading Context
//!
//! These types are particularly useful for:
//! - Market data subscriptions (none, one instrument, or many instruments)
//! - Order execution results (one fill, or multiple partial fills)
//! - Event handling (single event or batch of events)
//! - Configuration parameters (optional single value or list of values)
//!
//! ## Performance Considerations
//!
//! The FNV hasher types (`FnvIndexMap`, `FnvIndexSet`) provide better performance
//! than the default hasher for small keys commonly used in trading applications
//! (like instrument symbols, exchange IDs, etc.).

pub mod none_one_or_many;
pub mod one_or_many;

/// Fast IndexMap using FNV hasher for better performance with small keys
pub type FnvIndexMap<K, V> = indexmap::IndexMap<K, V, fnv::FnvBuildHasher>;

/// Fast IndexSet using FNV hasher for better performance with small keys
pub type FnvIndexSet<T> = indexmap::IndexSet<T, fnv::FnvBuildHasher>;

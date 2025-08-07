#![forbid(unsafe_code)]
#![warn(
    unused,
    clippy::cognitive_complexity,
    unused_crate_dependencies,
    unused_extern_crates,
    clippy::unused_self,
    clippy::useless_let_if_seq,
    missing_debug_implementations,
    rust_2018_idioms
)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, type_alias_bounds)]

//! # Markets - Simplified Abstractions
//! 
//! Core traits and types for exchanges, instruments, and assets.
//! Focused on essential abstractions without specific implementations.

use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// Defines exchange abstractions
pub mod exchange;

/// Defines asset abstractions  
pub mod asset;

/// Defines instrument abstractions and side enum
pub mod side;

/// Re-export key traits for convenience
pub use exchange::{Exchange, ExchangeId};
pub use asset::{Asset, AssetType};
pub use side::{Instrument, Side};

/// A keyed value utility
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Constructor,
)]
pub struct Keyed<Key, Value> {
    pub key: Key,
    pub value: Value,
}

/// Instrument Underlying containing a base and quote asset.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Underlying<AssetKey> {
    pub base: AssetKey,
    pub quote: AssetKey,
}

impl<AssetKey> Underlying<AssetKey> {
    pub fn new<A>(base: A, quote: A) -> Self
    where
        A: Into<AssetKey>,
    {
        Self {
            base: base.into(),
            quote: quote.into(),
        }
    }
}

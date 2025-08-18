// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
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

//! DISCLAIMER (summary): For educational/experimental use only. No investment advice or affiliation. No third-party compensation. Profit/ProfitDLL © Nelógica. Technical integration only. Read the full README & DISCLAIMER.
//! # 🏛️ Markets - Simplified Market Abstractions
//!
//! Fundamental traits and types for exchanges, instruments, and financial assets.
//! Focused on essential abstractions without specific implementations.
//!
//! ## 🎯 Design Philosophy
//!
//! This module implements a **hybrid** architecture that combines:
//! - **Reusable Abstractions**: Generic traits for maximum flexibility
//! - **Specific Implementations**: Brazilian types with native terminology
//! - **Extensibility**: Easy addition of new exchanges and instruments
//!
//! ## 🏗️ Main Modules
//!
//! - `exchange`: Exchange abstractions and identifiers
//! - `asset`: Financial asset definitions and types
//! - `instrument`: Financial instrument abstractions
//! - `side`: Operation side enumeration (Buy/Sell)
//! - `b3`: Brazilian Exchange (B3) specific definitions
//!
//! ## 💡 Fundamental Concepts
//!
//! ### Exchange
//! Represents a market or exchange where instruments are traded:
//! ```rust,ignore
//! use markets::{Exchange, ExchangeId};
//!
//! struct B3Exchange;
//! impl Exchange for B3Exchange {
//!     type ExchangeId = B3ExchangeId;
//!     fn id(&self) -> Self::ExchangeId { /* ... */ }
//!     fn name(&self) -> &'static str { "B3" }
//! }
//! ```
//!
//! ### Instrument
//! Defines tradable financial instruments:
//! ```rust,ignore
//! use markets::{Instrument, InstrumentKind};
//!
//! struct Stock {
//!     symbol: String,
//!     kind: InstrumentKind,
//! }
//! ```
//!
//! ### Asset
//! Represents underlying financial assets:
//! ```rust,ignore
//! use markets::{Asset, AssetType};
//!
//! struct BrazilianReal;
//! impl Asset for BrazilianReal {
//!     fn symbol(&self) -> &str { "BRL" }
//!     fn asset_type(&self) -> AssetType { AssetType::Currency }
//! }
//! ```
//!
//! ## 🇧🇷 Brazilian Market Support
//!
//! - **B3 Integration**: Native support for the Brazilian Exchange
//! - **ProfitDLL**: Connectivity via Nelógica
//! - **Local Terminology**: Use of Brazilian market-specific terms
//! - **Regulation**: Compliance with CVM rules
//!

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
// Silence unused_crate_dependencies for transitional re-export of tucano-profitdll

/// Re-exports main traits for convenient usage.
///
/// Allows easy import of the module's fundamental traits
/// without specifying the full path to each submodule.
pub use asset::{Asset, AssetType};
pub use exchange::{Exchange, ExchangeId};
pub use instrument::{Instrument, InstrumentKind, MarketDataInstrument};
pub use side::Side;

/// Defines abstractions for financial exchanges.
///
/// Contains traits and types to represent different markets
/// and exchanges where financial instruments are traded.
pub mod exchange;

/// Defines abstractions for financial assets.
///
/// Includes definitions for different asset types such as
/// currencies, stocks, commodities, etc., with their specific
/// characteristics and identification methods.
pub mod asset;

/// Defines abstractions for financial instruments.
///
/// Contains traits and structures to represent tradable instruments
/// such as stocks, options, futures, etc., including market metadata
/// and identification.
pub mod instrument;

/// Defines the operation side enumeration.
///
/// Specifies whether an operation is a buy (Buy) or sell (Sell),
/// fundamental for order definition and flow analysis.
pub mod side;

/// Utility for values with an associated key.
///
/// Generic structure that combines a key with a value,
/// useful for mapping data with specific identifiers
/// in a type-safe and efficient way.
///
/// # Example
/// ```rust,ignore
/// use markets::Keyed;
///
/// let keyed_price = Keyed::new("PETR4", 25.50);
/// assert_eq!(keyed_price.key, "PETR4");
/// assert_eq!(keyed_price.value, 25.50);
/// ```
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Constructor,
)]
pub struct Keyed<Key, Value> {
    pub key: Key,
    pub value: Value,
}

impl<Key, Value> AsRef<Value> for Keyed<Key, Value> {
    fn as_ref(&self) -> &Value {
        &self.value
    }
}

impl<Key, Value> Display for Keyed<Key, Value>
where
    Key: Display,
    Value: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.key, self.value)
    }
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

// Re-exports
// Re-export commonly used instrument struct
pub use crate::instrument::ConcreteInstrument;

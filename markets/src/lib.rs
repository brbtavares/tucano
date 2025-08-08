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
//! 
//! ## Modules
//! - `broker`: Broker abstraction layer with ProfitDLL integration
//! - `b3`: Brazilian Stock Exchange (B3) asset definitions
//! - `profit_dll`: ProfitDLL integration (real DLL on Windows, mock elsewhere)

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Re-export key traits for convenience
pub use asset::{Asset, AssetType};
pub use instrument::{Instrument, InstrumentKind, MarketDataInstrument};
pub use side::Side;
pub use exchange::{Exchange, ExchangeId};

/// Defines exchange abstractions
pub mod exchange;

/// Defines asset abstractions  
pub mod asset;

/// Defines instrument abstractions
pub mod instrument;

/// Defines side enum
pub mod side;

/// A keyed value utility
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

// Module declarations
pub mod broker;
pub mod b3;

// ProfitDLL integration - conditional compilation
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub mod profit_dll_complete;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use profit_dll_complete as profit_dll;

#[cfg(not(all(target_os = "windows", feature = "real_dll")))]
pub mod profit_dll;

// Re-exports
pub use broker::*;
pub use b3::*;
// Re-export profit_dll types selectively to avoid conflicts
pub use profit_dll::{
    CallbackEvent, ConnectionState, BookAction, NResult, ProfitConnector,
    AssetIdentifier, AccountIdentifier, SendOrder, OrderValidity, ProfitError,
    // Note: OrderSide is already re-exported from broker
};

// Constants
pub use profit_dll::{
    NL_OK, NL_INTERNAL_ERROR, NL_NOT_INITIALIZED, NL_INVALID_ARGS,
    NL_WAITING_SERVER, NL_NO_LOGIN, NL_NO_LICENSE,
};

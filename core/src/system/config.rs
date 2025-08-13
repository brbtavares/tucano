use execution::client::mock::MockExecutionConfig;
/// Configuration module for trading system components.
///
/// Provides data structures for configuring various aspects of a trading system,
/// including instruments and execution components.
use tucano_markets::ConcreteInstrument; // updated import for shared instrument
use tucano_markets::{exchange::ExchangeId, Underlying};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Placeholder types for configuration
pub type AssetNameExchange = String;
pub type InstrumentNameExchange = String;
pub type InstrumentNameInternal = String;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum InstrumentKind {
    Spot,
    Future(FutureContract),
    Option(OptionContract),
    Perpetual(PerpetualContract),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct FutureContract {
    pub expiry: String,
    pub contract_size: f64,
    pub settlement_asset: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct OptionContract {
    pub strike: f64,
    pub expiry: String,
    pub contract_size: f64,
    pub settlement_asset: String,
    pub kind: String,
    pub exercise: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PerpetualContract {
    pub funding_rate: f64,
    pub contract_size: f64,
    pub settlement_asset: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct InstrumentQuoteAsset {
    pub asset: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InstrumentSpec {
    pub quantity: InstrumentSpecQuantity,
    pub price: Option<f64>,
    pub notional: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InstrumentSpecQuantity {
    pub unit: OrderQuantityUnits,
    pub min: f64,
    pub increment: f64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum OrderQuantityUnits {
    Asset(String),
    Contract,
    Quote,
}

use derive_more::From;

/// Top-level configuration for a full trading system.
///
/// Contains configuration for all instruments and execution components.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SystemConfig {
    /// Configurations for all instruments the system will track.
    pub instruments: Vec<InstrumentConfig>,

    /// Configurations for all execution components.
    pub executions: Vec<ExecutionConfig>,
}

/// Convenient minimal instrument configuration, used to generate an [`Instrument`] on startup.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InstrumentConfig {
    /// Exchange identifier where the instrument is traded.
    pub exchange: ExchangeId,

    /// Exchange-specific name for the instrument (e.g., "BTCUSDT").
    pub name_exchange: InstrumentNameExchange,

    /// Underlying asset pair for the instrument.
    pub underlying: Underlying<AssetNameExchange>,

    /// Quote asset for the instrument.
    pub quote: InstrumentQuoteAsset,

    /// Type of the instrument (spot, perpetual, future, option).
    pub kind: InstrumentKind,

    /// Optional additional specifications for the instrument.
    pub spec: Option<InstrumentSpec>,
}

/// Configuration for an execution link.
///
/// Represents different types of execution configurations,
/// currently only supporting mock execution for backtesting.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, From)]
#[serde(untagged)]
pub enum ExecutionConfig {
    /// Mock execution configuration for backtesting
    Mock(MockExecutionConfig),
}

impl From<InstrumentConfig> for ConcreteInstrument {
    fn from(value: InstrumentConfig) -> Self {
        Self {
            symbol: value.underlying.base.clone(),
            market: "default_market".to_string(),
            exchange: value.exchange,
            underlying: Some(format!(
                "{}_{}",
                value.underlying.base, value.underlying.quote
            )),
            name_exchange: value.name_exchange,
        }
    }
}


//! "toucan" facade crate
//!
//! Provides a single entry point that re-exports the main modules
//! of the Toucan ecosystem. Useful for users who prefer to depend on
//! just one crate.
//!
//! # Example
//! ```rust
//! use toucan::core; // access modules via re-export
//! use toucan::markets::ExchangeId; // market enum
//! let _exchange: ExchangeId = ExchangeId::B3;
//! // Engine available at toucan::core::engine, construction requires specific dependencies.
//! ```

// Re-export of internal crates with organized namespaces
pub use toucan_analytics as analytics;
pub use toucan_core as core;
pub use toucan_data as data;
pub use toucan_execution as execution;
pub use toucan_instrument as markets;
pub use toucan_integration as integration;
pub use toucan_risk as risk;
pub use toucan_strategies as strategies;
pub use toucan_trader as trader;

// Flat (shallow) re-export of very frequently used symbols
pub use toucan_core::{engine::Engine, EngineEvent, Sequence};
pub use toucan_instrument::{ExchangeId, Side};

// Optional prelude for single import
pub mod prelude {
    pub use crate::core::{engine::Engine, EngineEvent, Sequence};
    pub use crate::execution::{order, trade};
    pub use crate::markets::{ExchangeId, Side};
    pub use crate::trader::{algo::AlgoStrategy, on_trading_disabled::OnTradingDisabled};
}

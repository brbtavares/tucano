// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
//! "tucano" facade crate
//!
//! Provides a single entry point that re-exports the main modules
//! of the Tucano ecosystem. Useful for users who prefer to depend on
//! just one crate.
//!
//! # Example
//! ```rust
//! use tucano::core; // access modules via re-export
//! use tucano::markets::ExchangeId; // market enum
//! let _exchange: ExchangeId = ExchangeId::B3;
//! // Engine available at tucano::core::engine, construction requires specific dependencies.
//! ```

// Re-export of internal crates with organized namespaces
// Re-export of external crate tucano-profitdll to avoid local module name ambiguity
pub use tucano_analytics as analytics;
pub use tucano_core as core;
pub use tucano_data as data;
pub use tucano_execution as execution;
pub use tucano_integration as integration;
pub use tucano_markets as markets;
pub use tucano_profitdll as profitdll;
pub use tucano_risk as risk;
pub use tucano_strategies as strategies;
pub use tucano_trader as trader;

// Flat (shallow) re-export of very frequently used symbols
pub use tucano_core::{engine::Engine, EngineEvent, Sequence};
pub use tucano_markets::{ExchangeId, Side};

// Optional prelude for single import
pub mod prelude {
    pub use crate::core::{engine::Engine, EngineEvent, Sequence};
    pub use crate::execution::{order, trade};
    pub use crate::markets::{ExchangeId, Side};
    pub use crate::trader::{algo::AlgoStrategy, on_trading_disabled::OnTradingDisabled};
}

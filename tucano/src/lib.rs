//! Crate fachada "tucano"
//!
//! Fornece um ponto único de entrada que re-exporta os principais módulos
//! do ecossistema Tucano. Útil para usuários que preferem depender de
//! apenas uma crate.
//!
//! # Exemplo
//! ```rust
//! use tucano::engine::Engine; // re-export de tucano-core
//! use tucano::markets::ExchangeId; // re-export de tucano-markets
//! ```

// Re-export de crates internas com namespaces organizados
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

// Re-export plano (superficial) de símbolos de uso muito frequente
pub use tucano_core::{engine::Engine, EngineEvent, Sequence};
pub use tucano_markets::{ExchangeId, Side};

// Prelude opcional para import único
pub mod prelude {
    pub use crate::core::{engine::Engine, EngineEvent, Sequence};
    pub use crate::execution::{order, trade};
    pub use crate::markets::{ExchangeId, Side};
    pub use crate::trader::{algo::AlgoStrategy, on_trading_disabled::OnTradingDisabled};
}

// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Crate fachada "tucano"
//!
//! Fornece um ponto único de entrada que re-exporta os principais módulos
//! do ecossistema Tucano. Útil para usuários que preferem depender de
//! apenas uma crate.
//!
//! # Exemplo
//! ```rust
//! use tucano::core; // acesso aos módulos via re-export
//! use tucano::markets::ExchangeId; // enum de mercados
//! let _exchange: ExchangeId = ExchangeId::B3;
//! // Engine disponível em tucano::core::engine, construção exige dependências específicas.
//! ```

// Re-export de crates internas com namespaces organizados
pub use tucano_analytics as analytics;
pub use tucano_core as core;
pub use tucano_data as data;
pub use tucano_execution as execution;
pub use tucano_integration as integration;
pub use tucano_markets as markets;
pub use profitdll::*;
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

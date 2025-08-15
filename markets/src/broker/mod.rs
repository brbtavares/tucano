// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Broker abstraction layer
//!
//! Interfaces unificadas para interação com diferentes brokers e provedores
//! de dados. Implementações concretas específicas (ex: ProfitDLL) residem em
//! crates externas como `tucano-profitdll`.

pub mod traits;
pub use traits::*;

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

//! DISCLAIMER (resumo): Uso educacional/experimental. Sem recomendação de investimento.
//! Sem afiliação institucional ou remuneração de terceiros. Profit/ProfitDLL © Nelógica.
//! Integração meramente técnica. Leia README & DISCLAIMER completos.
//! # 🏛️ Markets - Abstrações Simplificadas de Mercado
//!
//! Traits e tipos fundamentais para exchanges, instrumentos e ativos financeiros.
//! Focado em abstrações essenciais sem implementações específicas.
//!
//! ## 🎯 Filosofia de Design
//!
//! Este módulo implementa uma arquitetura **híbrida** que combina:
//! - **Abstrações Reutilizáveis**: Traits genéricos para máxima flexibilidade
//! - **Implementações Específicas**: Tipos brasileiros com terminologia nativa
//! - **Extensibilidade**: Fácil adição de novos exchanges e instrumentos
//!
//! ## 🏗️ Módulos Principais
//!
//! - `exchange`: Abstrações de exchange e identificadores
//! - `asset`: Definições de ativos financeiros e tipos
//! - `instrument`: Abstrações de instrumentos financeiros
//! - `side`: Enumeração de lados de operação (Buy/Sell)
//! - `b3`: Definições específicas da Bolsa Brasileira (B3)
//! - `broker`: Camada de abstração de corretoras (abstrata – integrações externas em crates próprias)
//!
//! ## 💡 Conceitos Fundamentais
//!
//! ### Exchange
//! Representa um mercado ou bolsa onde instrumentos são negociados:
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
//! Define instrumentos financeiros negociáveis:
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
//! Representa ativos financeiros subjacentes:
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
//! ## 🇧🇷 Suporte ao Mercado Brasileiro
//!
//! - **B3 Integration**: Suporte nativo à Bolsa Brasileira
//! - **ProfitDLL**: Conectividade através da Nelógica
//! - **Terminologia Local**: Uso de termos específicos do mercado brasileiro
//! - **Regulamentação**: Conformidade com regras da CVM
//!

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
// Silence unused_crate_dependencies for transitional re-export of tucano-profitdll

/// Re-exporta traits principais para conveniência de uso.
///
/// Permite importar facilmente os traits fundamentais do módulo
/// sem precisar especificar o caminho completo de cada submódulo.
pub use asset::{Asset, AssetType};
pub use exchange::{Exchange, ExchangeId};
pub use instrument::{Instrument, InstrumentKind, MarketDataInstrument};
pub use side::Side;

/// Define abstrações de exchanges financeiros.
///
/// Contém traits e tipos para representar diferentes mercados
/// e bolsas onde instrumentos financeiros são negociados.
pub mod exchange;

/// Define abstrações de ativos financeiros.
///
/// Inclui definições para diferentes tipos de ativos como
/// moedas, ações, commodities, etc., com suas características
/// específicas e métodos de identificação.
pub mod asset;

/// Define abstrações de instrumentos financeiros.
///
/// Contém traits e estruturas para representar instrumentos
/// negociáveis como ações, opções, futuros, etc., incluindo
/// metadados de mercado e identificação.
pub mod instrument;

/// Define enumeração de lados de operação.
///
/// Especifica se uma operação é de compra (Buy) ou venda (Sell),
/// fundamental para definição de ordens e análise de fluxo.
pub mod side;

/// Utilitário para valores com chave associada.
///
/// Estrutura genérica que combina uma chave com um valor,
/// útil para mapear dados com identificadores específicos
/// de forma type-safe e eficiente.
///
/// # Exemplo
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

// Module declarations
pub mod b3;
pub mod broker;

// Re-exports
pub use b3::*;
pub use broker::*;
// Re-export commonly used instrument struct
pub use crate::instrument::ConcreteInstrument;

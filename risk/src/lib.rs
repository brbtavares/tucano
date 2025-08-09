//! # 🛡️ Risk - Módulo de Gestão de Risco
//!
//! Framework abrangente para gestão de risco em trading algorítmico,
//! fornecendo validações, limites e controles para proteger o capital
//! e garantir conformidade regulatória.
//!
//! ## 🎯 Objetivos Principais
//!
//! - **Proteção de Capital**: Prevenção de perdas excessivas
//! - **Controle de Exposição**: Limitação de posições por ativo/mercado
//! - **Conformidade**: Aderência a regulamentações financeiras
//! - **Performance**: Validações em tempo real com baixa latência
//!
//! ## 🏗️ Componentes do Sistema
//!
//! ### RiskManager
//! Interface principal para revisão e filtragem de ordens:
//! ```rust,no_run
//! use risk::{RiskManager, RiskApproved, RiskRefused};
//!
//! impl RiskManager for MyRiskManager {
//!     fn check_order(&self, order: &Order) -> Result<RiskApproved<Order>, RiskRefused<Order>> {
//!         // Implementar validações específicas
//!     }
//! }
//! ```
//!
//! ### Tipos de Validação
//! - **Position Limits**: Limites máximos de posição por instrumento
//! - **Exposure Limits**: Limites de exposição total por mercado
//! - **Leverage Control**: Controle de alavancagem máxima
//! - **Concentration Risk**: Prevenção de concentração excessiva
//! - **Market Hours**: Validação de horários de mercado
//! - **Circuit Breakers**: Parada automática em perdas excessivas
//!
//! ## 🔍 Estruturas de Resultado
//!
//! ### RiskApproved<T>
//! Representa uma operação aprovada pelo sistema de risco:
//! ```rust
//! let approved = RiskApproved::new(order);
//! let order = approved.into_item(); // Extrair o item aprovado
//! ```
//!
//! ### RiskRefused<T>
//! Representa uma operação rejeitada com motivo específico:
//! ```rust
//! let refused = RiskRefused::new(order, "Excede limite de posição");
//! println!("Rejeitado: {}", refused.reason);
//! ```
//!
//! ## 🚨 Cenários de Risco Comum
//!
//! ### Limites de Posição
//! ```rust,no_run
//! if position_size > max_position_limit {
//!     return Err(RiskRefused::new(order, "Excede limite máximo de posição"));
//! }
//! ```
//!
//! ### Controle de Exposição
//! ```rust,no_run
//! let total_exposure = calculate_exposure(&portfolio);
//! if total_exposure > exposure_limit {
//!     return Err(RiskRefused::new(order, "Excede limite de exposição"));
//! }
//! ```
//!
//! ### Horário de Mercado
//! ```rust,no_run
//! if !is_market_open(instrument.exchange()) {
//!     return Err(RiskRefused::new(order, "Mercado fechado"));
//! }
//! ```
//!
//! ## 📊 Métricas de Risco
//!
//! - **VaR (Value at Risk)**: Risco de perda em condições normais
//! - **CVaR (Conditional VaR)**: Risco de perda em cenários extremos
//! - **Maximum Drawdown**: Maior perda histórica observada
//! - **Sharpe Ratio**: Retorno ajustado ao risco
//! - **Beta**: Correlação com mercado de referência
//!
//! ## 🔄 Integração com Engine
//!
//! O módulo de risco se integra nativamente com o core engine:
//! ```rust,no_run
//! use core::engine::Engine;
//! use risk::RiskManager;
//!
//! let engine = Engine::new(
//!     clock,
//!     state,
//!     execution_txs,
//!     strategy,
//!     risk_manager // <- Integração automática
//! );
//! ```

/// Módulo contendo implementações de verificações de risco.
///
/// Inclui validadores específicos para diferentes tipos de risco
/// como limites de posição, exposição, horários de mercado, etc.
pub mod check;

pub use check::*;

use derive_more::{Constructor, Display, From};
use execution::{
    order::request::{OrderRequestCancel, OrderRequestOpen},
    ExchangeIndex, InstrumentIndex,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// Resultado aprovado de uma verificação do [`RiskManager`].
///
/// Wrapper que indica que um item (como uma ordem) passou por todas
/// as verificações de risco e foi aprovado para execução.
///
/// # Exemplo
/// ```rust
/// use risk::RiskApproved;
///
/// let approved_order = RiskApproved::new(order);
/// println!("Ordem aprovada: {}", approved_order);
/// ```
#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
    Display,
    From,
    Constructor,
)]
pub struct RiskApproved<T>(pub T);

impl<T> RiskApproved<T> {
    /// Extrai o item aprovado do wrapper.
    pub fn into_item(self) -> T {
        self.0
    }
}

/// Resultado rejeitado de uma verificação do [`RiskManager`].
///
/// Contém o item rejeitado e o motivo específico da rejeição,
/// permitindo logging detalhado e ações corretivas.
///
/// # Exemplo
/// ```rust
/// use risk::RiskRefused;
///
/// let refused = RiskRefused::new(order, "Excede limite de posição");
/// println!("Ordem rejeitada: {}", refused.reason);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct RiskRefused<T, Reason = String> {
    /// O item que foi rejeitado
    pub item: T,
    /// Motivo específico da rejeição
    pub reason: Reason,
}

impl<T> RiskRefused<T> {
    /// Cria uma nova instância de `RiskRefused` com o item e motivo fornecidos.
    pub fn new(item: T, reason: impl Into<String>) -> Self {
        Self {
            item,
            reason: reason.into(),
        }
    }
}

impl<T, Reason> RiskRefused<T, Reason> {
    /// Extrai o item rejeitado do wrapper.
    pub fn into_item(self) -> T {
        self.item
    }
}

/// Interface do RiskManager para revisar e opcionalmente filtrar ordens de
/// cancelamento e abertura geradas por uma [`AlgoStrategy`](strategy::AlgoStrategy).
///
/// ## Responsabilidades Principais
///
/// Um RiskManager pode implementar diversas verificações como:
/// - **Filtro de Exposição**: Rejeitar ordens que resultariam em exposição excessiva
/// - **Limites de Posição**: Verificar se a ordem não excede limites por instrumento
/// - **Validação de Margem**: Garantir margem suficiente para novas posições
/// - **Horários de Mercado**: Validar se o mercado está aberto para negociação
/// - **Circuit Breakers**: Parar operações em caso de perdas excessivas
/// - **Compliance**: Verificar conformidade com regulamentações
///
/// ## Exemplo de Implementação
/// ```rust,no_run
/// use risk::{RiskManager, RiskApproved, RiskRefused};
///
/// struct MyRiskManager {
///     max_position: f64,
///     max_exposure: f64,
/// }
///
/// impl RiskManager for MyRiskManager {
///     fn check_order(&self, order: &Order) -> Result<RiskApproved<Order>, RiskRefused<Order>> {
///         if order.quantity > self.max_position {
///             return Err(RiskRefused::new(order.clone(), "Posição muito grande"));
///         }
///         Ok(RiskApproved::new(order.clone()))
///     }
/// }
/// ```
///
/// For example, a RiskManager implementation may wish to:
/// - Filter out orders that would result in too much exposure.
/// - Filter out orders that have a too high quantity.
/// - Adjust order quantities.
/// - Filter out orders that would cross the OrderBook.
/// - etc.
///
/// # Type Parameters
/// * `ExchangeKey` - Type used to identify an exchange (defaults to [`ExchangeIndex`]).
/// * `InstrumentKey` - Type used to identify an instrument (defaults to [`InstrumentIndex`]).
pub trait RiskManager<ExchangeKey = ExchangeIndex, InstrumentKey = InstrumentIndex> {
    type State;

    fn check(
        &self,
        state: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
    );
}

/// Pass-through risk manager that approves all requests.
#[derive(Debug, Clone, Default)]
pub struct NoRiskManager;

impl<ExchangeKey, InstrumentKey> RiskManager<ExchangeKey, InstrumentKey> for NoRiskManager {
    type State = ();

    fn check(
        &self,
        _state: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
    ) {
        let approved_cancels: Vec<_> = cancels.into_iter().map(RiskApproved::new).collect();
        let approved_opens: Vec<_> = opens.into_iter().map(RiskApproved::new).collect();
        let refused_cancels: Vec<RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>> =
            vec![];
        let refused_opens: Vec<RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>> = vec![];

        (
            approved_cancels,
            approved_opens,
            refused_cancels,
            refused_opens,
        )
    }
}

/// Naive implementation of the [`RiskManager`] interface, approving all orders *without any
/// risk checks*.
///
/// *THIS IS FOR DEMONSTRATION PURPOSES ONLY, NEVER USE FOR REAL TRADING OR IN PRODUCTION*.
#[derive(Debug, Clone)]
pub struct DefaultRiskManager<State> {
    phantom: PhantomData<State>,
}

impl<State> Default for DefaultRiskManager<State> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<State, ExchangeKey, InstrumentKey> RiskManager<ExchangeKey, InstrumentKey>
    for DefaultRiskManager<State>
{
    type State = State;

    fn check(
        &self,
        _: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<ExchangeKey, InstrumentKey>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<ExchangeKey, InstrumentKey>>>,
    ) {
        (
            cancels.into_iter().map(RiskApproved::new),
            opens.into_iter().map(RiskApproved::new),
            std::iter::empty(),
            std::iter::empty(),
        )
    }
}

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

//! DISCLAIMER: Uso experimental/educacional. Não é recomendação de investimento. Veja README e DISCLAIMER.md.
//! # 🧠 Core - Engine Principal do Framework Toucan
//!
//! Framework Rust para construção de sistemas profissionais de trading ao vivo,
//! paper trading e backtesting. O Engine central facilita execução em múltiplos
//! exchanges simultaneamente e oferece flexibilidade para executar a maioria dos
//! tipos de estratégias de trading.
//!
//! ## 🎯 Características Principais
//!
//! - **Multi-Exchange**: Execução simultânea em múltiplos exchanges
//! - **Estratégias Flexíveis**: Suporte a diversos tipos de estratégias algorítmicas
//! - **Controle Dinâmico**: Liga/desliga geração de ordens algorítmicas
//! - **Comandos Externos**: Aceita comandos de processos externos
//! - **Type Safety**: Sistema de tipos Rust para máxima segurança
//!
//! ## 🏗️ Arquitetura do Engine
//!
//! O Engine é o componente central que:
//! - Processa eventos de mercado e conta em tempo real
//! - Executa estratégias algorítmicas configuradas
//! - Gerencia estado global do sistema de trading
//! - Aplica regras de gestão de risco
//! - Mantém auditoria completa de operações
//!
//! ## 🔄 Fluxo de Processamento
//!
//! ```text
//! Eventos de Mercado/Conta
//!           ↓
//!      Engine Central
//!           ↓
//!    Estratégia + Risk
//!           ↓
//!    Ordens Geradas
//!           ↓
//!   Execution Clients
//!           ↓
//!      Exchanges
//! ```
//!
//! ## 💡 Comandos Suportados
//!
//! - `CloseAllPositions`: Fecha todas as posições abertas
//! - `OpenOrders`: Lista ordens abertas
//! - `CancelOrders`: Cancela ordens específicas
//! - `SetTradingState`: Controla estado de trading (enabled/disabled)
//! - `GetPositions`: Consulta posições atuais
//!
//! ## 🧩 Componentes Integrados
//!
//! - **EngineState**: Estado global com dados de mercado e conta
//! - **TradingStrategy**: Interface para estratégias algorítmicas
//! - **RiskManager**: Validação e controle de risco
//! - **ExecutionClients**: Conectividade com exchanges
//! - **AuditTrail**: Rastreamento completo de operações

/// Core é um framework Rust para construção de sistemas profissionais de live-trading,
/// paper-trading e back-testing. O Engine central facilita execução em muitos exchanges
/// simultaneamente, e oferece flexibilidade para executar a maioria dos tipos de
/// estratégias de trading. Permite ligar/desligar geração de ordens algorítmicas e pode
/// executar Comandos emitidos de processos externos (ex: CloseAllPositions, OpenOrders, CancelOrders, etc.)
use crate::{
    engine::{command::Command, state::trading::TradingState},
    execution::AccountStreamEvent,
};
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use shutdown::Shutdown;
use tucano_data::{
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
};
use tucano_execution::{AccountEvent, AssetIndex, ExchangeIndex, InstrumentIndex};
use tucano_integration::Terminal;

// Suppress unused extern crate warnings
use prettytable as _;

/// Algorithmic trading `Engine`, and entry points for processing input `Events`.
///
/// eg/ `Engine`, `run`, `process_with_audit`, etc.
pub mod engine;

/// Defines all possible errors in Core.
pub mod error;

/// Components for initialising multi-exchange execution, routing `ExecutionRequest`s and other
/// execution logic.
pub mod execution;

/// Provides default Core Tracing logging initialisers.
pub mod logging;

/// RiskManager interface for reviewing and optionally filtering algorithmic cancel and open
/// order requests.
pub use tucano_risk as risk;
pub use tucano_trader as strategy; // temporary alias for backward compatibility

/// Statistical algorithms for analysing datasets, financial metrics and financial summaries.
///
/// eg/ `TradingSummary`, `TearSheet`, `SharpeRatio`, etc.
pub use tucano_analytics as analytics; // transitional re-export

// Strategy interfaces foram movidas para a crate `tucano-trader`.
// Importar via: `use tucano_trader::{AlgoStrategy, ClosePositionsStrategy, ...};`

/// Utilities for initialising and interacting with a full trading system.
pub mod system;

/// Backtesting utilities.
pub mod backtest;

/// Traits and types related to component shutdowns.
pub mod shutdown;

/// A timed value.
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Deserialize,
    Serialize,
    Constructor,
)]
pub struct Timed<T> {
    pub value: T,
    pub time: DateTime<Utc>,
}

/// Default [`Engine`](engine::Engine) event that encompasses market events, account/execution
/// events, and `Engine` commands.
///
/// Note that the `Engine` can be configured to process custom events.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, From)]
pub enum EngineEvent<
    MarketKind = DataKind,
    ExchangeKey = ExchangeIndex,
    AssetKey = AssetIndex,
    InstrumentKey = InstrumentIndex,
> {
    Shutdown(Shutdown),
    Command(Command<ExchangeKey, AssetKey, InstrumentKey>),
    TradingStateUpdate(TradingState),
    Account(AccountStreamEvent<ExchangeKey, AssetKey, InstrumentKey>),
    Market(MarketStreamEvent<InstrumentKey, MarketKind>),
}

impl<MarketKind, ExchangeKey, AssetKey, InstrumentKey> Terminal
    for EngineEvent<MarketKind, ExchangeKey, AssetKey, InstrumentKey>
{
    fn is_terminal(&self) -> bool {
        matches!(self, Self::Shutdown(_))
    }
}

impl<MarketKind, ExchangeKey, AssetKey, InstrumentKey>
    EngineEvent<MarketKind, ExchangeKey, AssetKey, InstrumentKey>
{
    pub fn shutdown() -> Self {
        Self::Shutdown(Shutdown)
    }
}

impl<MarketKind, ExchangeKey, AssetKey, InstrumentKey>
    From<AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
    for EngineEvent<MarketKind, ExchangeKey, AssetKey, InstrumentKey>
{
    fn from(value: AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self {
        Self::Account(AccountStreamEvent::Item(value))
    }
}

impl<MarketKind, ExchangeKey, AssetKey, InstrumentKey> From<MarketEvent<InstrumentKey, MarketKind>>
    for EngineEvent<MarketKind, ExchangeKey, AssetKey, InstrumentKey>
{
    fn from(value: MarketEvent<InstrumentKey, MarketKind>) -> Self {
        Self::Market(MarketStreamEvent::Item(value))
    }
}

/// Monotonically increasing event sequence. Used to track `Engine` event processing sequence.
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Constructor,
)]
pub struct Sequence(pub u64);

impl Sequence {
    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn fetch_add(&mut self) -> Sequence {
        let sequence = *self;
        self.0 += 1;
        sequence
    }
}

/// Core test utilities.
pub mod test_utils {
    use crate::{engine::state::asset::AssetState, Timed};
    use tucano_analytics::summary::asset::TearSheetAssetGenerator;
    use tucano_execution::{
        balance::{AssetBalance, Balance},
        order::id::{OrderId, StrategyId},
        trade::{AssetFees, Trade, TradeId},
    };
    use tucano_markets::Side;

    // Placeholder type for integration
    type InstrumentNameInternal = String;
    use chrono::{DateTime, Days, TimeDelta, Utc};
    use rust_decimal::Decimal;

    pub fn f64_is_eq(actual: f64, expected: f64, epsilon: f64) -> bool {
        if actual.is_nan() && expected.is_nan() {
            true
        } else if actual.is_infinite() && expected.is_infinite() {
            actual.is_sign_positive() == expected.is_sign_positive()
        } else if actual.is_nan()
            || expected.is_nan()
            || actual.is_infinite()
            || expected.is_infinite()
        {
            false
        } else {
            (actual - expected).abs() < epsilon
        }
    }

    pub fn time_plus_days(base: DateTime<Utc>, plus: u64) -> DateTime<Utc> {
        base.checked_add_days(Days::new(plus)).unwrap()
    }

    pub fn time_plus_secs(base: DateTime<Utc>, plus: i64) -> DateTime<Utc> {
        base.checked_add_signed(TimeDelta::seconds(plus)).unwrap()
    }

    pub fn time_plus_millis(base: DateTime<Utc>, plus: i64) -> DateTime<Utc> {
        base.checked_add_signed(TimeDelta::milliseconds(plus))
            .unwrap()
    }

    pub fn time_plus_micros(base: DateTime<Utc>, plus: i64) -> DateTime<Utc> {
        base.checked_add_signed(TimeDelta::microseconds(plus))
            .unwrap()
    }

    pub fn trade(
        time_exchange: DateTime<Utc>,
        side: Side,
        price: f64,
        quantity: f64,
        fees: f64,
    ) -> Trade<String, InstrumentNameInternal> {
        Trade {
            id: TradeId::new("trade_id"),
            order_id: OrderId::new("order_id"),
            instrument: "instrument".to_string(), // InstrumentNameInternal is String
            strategy: StrategyId::new("strategy"),
            time_exchange,
            side,
            price: price.try_into().unwrap(),
            quantity: quantity.try_into().unwrap(),
            fees: AssetFees {
                asset: "quote".to_string(), // Normalised quote asset name
                fees: fees.try_into().unwrap(),
            },
        }
    }

    pub fn asset_state(
        symbol: &str,
        balance_total: f64,
        balance_free: f64,
        time_exchange: DateTime<Utc>,
    ) -> AssetState {
        let balance = Timed::new(
            Balance::new(
                Decimal::try_from(balance_total).unwrap(),
                Decimal::try_from(balance_free).unwrap(),
            ),
            time_exchange,
        );

        // Create AssetBalance for the analytics
        let asset_balance = AssetBalance {
            asset: "asset".to_string(), // AssetIndex is String
            balance: balance.value,
            time_exchange: balance.time,
        };

        AssetState {
            asset: symbol.to_string(), // Simplified Asset representation
            balance: Some(balance),
            statistics: TearSheetAssetGenerator::init(&asset_balance),
        }
    }
}

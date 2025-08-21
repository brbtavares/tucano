
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
// ...existing code...
//! # 🧠 Core - Main Engine of the Toucan Framework
//!
//! Rust framework for building professional live trading systems,
//! paper trading, and backtesting. The central Engine facilitates execution on multiple
//! exchanges simultaneously and offers flexibility to run most types of trading strategies.
//!
//! ## 🎯 Main Features
//!
//! - **Multi-Exchange**: Simultaneous execution on multiple exchanges
//! - **Flexible Strategies**: Support for various types of algorithmic strategies
//! - **Dynamic Control**: Enable/disable algorithmic order generation
//! - **External Commands**: Accepts commands from external processes
//! - **Type Safety**: Rust type system for maximum safety
//!
//! ## 🏗️ Engine Architecture
//!
//! The Engine is the central component that:
//! - Processes real-time market and account events
//! - Executes configured algorithmic strategies
//! - Manages global trading system state
//! - Applies risk management rules
//! - Maintains a complete audit of operations
//!
//! ## 🔄 Processing Flow
//!
//! ```text
//! Market/Account Events
//!           ↓
//!      Central Engine
//!           ↓
//!    Strategy + Risk
//!           ↓
//!    Orders Generated
//!           ↓
//!   Execution Clients
//!           ↓
//!      Exchanges
//! ```
//!
//! ## 💡 Supported Commands
//!
//! - `CloseAllPositions`: Closes all open positions
//! - `OpenOrders`: Lists open orders
//! - `CancelOrders`: Cancels specific orders
//! - `SetTradingState`: Controls trading state (enabled/disabled)
//! - `GetPositions`: Queries current positions
//!
//! ## 🛩️ Integrated Components
//!
//! - **EngineState**: Global state with market and account data
//! - **TradingStrategy**: Interface for algorithmic strategies
//! - **RiskManager**: Validation and risk control
//! - **ExecutionClients**: Connectivity with exchanges
//! - **AuditTrail**: Complete operations tracking
//!
//! > Note: Complete usage examples have been temporarily removed from doctests
//! > until public APIs are stabilized. For examples, see the `examples/` directory in the repository.

/// Core is a Rust framework for building professional live-trading systems,
/// paper-trading, and back-testing. The central Engine facilitates execution on many exchanges
/// simultaneously and offers flexibility to run most types of trading strategies.
/// It allows enabling/disabling algorithmic order generation and can execute commands issued from external processes (e.g., CloseAllPositions, OpenOrders, CancelOrders, etc.)
use crate::{
    engine::{command::Command, state::trading::TradingState},
    execution::AccountStreamEvent,
};
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use shutdown::Shutdown;
use toucan_data::{
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
};
use toucan_execution::{AccountEvent, AssetIndex, ExchangeIndex, InstrumentIndex};
use toucan_integration::Terminal;

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
pub use toucan_risk as risk;
pub use toucan_trader as strategy; // temporary alias for backward compatibility

/// Statistical algorithms for analysing datasets, financial metrics and financial summaries.
///
/// eg/ `TradingSummary`, `TearSheet`, `SharpeRatio`, etc.
pub use toucan_analytics as analytics; // transitional re-export

// Strategy interfaces foram movidas para a crate `toucan-trader`.
// Importar via: `use toucan_trader::{AlgoStrategy, ClosePositionsStrategy, ...};`

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
    use toucan_analytics::summary::asset::TearSheetAssetGenerator;
    use toucan_execution::{
        balance::{AssetBalance, Balance},
        order::id::{OrderId, StrategyId},
        trade::{AssetFees, Trade, TradeId},
    };
    use toucan_instrument::Side;

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

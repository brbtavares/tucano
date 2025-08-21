
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
// (moved dummy imports below crate docs to satisfy inner doc comment placement rules)

// ...existing code...
//! # ⚡ Execution - Order Execution Module
//!
//! Private account data streams from financial venues and order execution
//! (live or simulated). Also provides feature-rich MockExchange and MockExecutionClient
//! to assist with backtesting and paper trading.
//!
//! ## 🎯 Main Features
//!
//! * **🚀 Simplicity**: ExecutionClient trait provides a unified
//!   and simple language to interact with exchanges
//! * **🔄 Standardization**: Allows your strategy to communicate with any
//!   real or Mock exchange using the same interface
//! * **🔧 Extensibility**: Highly extensible, making it easy to contribute
//!   new exchange integrations
//!
//! ## 🏗️ Main Components
//!
//! ### ExecutionClient
//! Unified interface for order execution on different exchanges.
//! Below is a sketch (non-compilable) of how a concrete implementation might look:
//! ```rust,ignore
//! use execution::client::ExecutionClient;
//! use markets::ExchangeId;
//!
//! #[derive(Clone)]
//! struct MyClient;
//!
//! impl ExecutionClient for MyClient {
//!     const EXCHANGE: ExchangeId = ExchangeId::B3; // example
//!     type Config = ();
//!     type AccountStream = futures::stream::Empty<execution::UnindexedAccountEvent>;
//!     fn new(_: Self::Config) -> Self { Self }
//!     // Other methods required by the trait must be implemented...
//!     // fn account_snapshot(..) -> ... { }
//!     // fn open_order(..) -> ... { }
//! }
//! ```
//!
//! ### MockExchange
//! Simulated exchange for backtesting and testing:
//! - **Realistic Latency**: Simulates network and processing delays
//! - **Slippage**: Models real price slippage
//! - **Rejections**: Simulates rejections due to risk or liquidity
//!
//! ### Balance Management
//! Robust system for tracking balances and positions:
//! - **Multi-Asset**: Supports multiple assets simultaneously
//! - **Real-Time**: Real-time updates via streams
//! - **Reconciliation**: Automatic consistency validation
//!
//! ## 💡 Usage Example
//!
//! ```rust,ignore
//! // Conceptual flow example (pseudocode):
//! use execution::client::ExecutionClient;
//! use execution::order::request::OrderRequestOpen;
//! use execution::order::{OrderKind, TimeInForce, OrderKey};
//! use execution::order::id::{ClientOrderId, StrategyId};
//! use markets::ExchangeId;
//! use rust_decimal_macros::dec;
//!
//! async fn example(mut client: impl ExecutionClient) {
//!     let instrument = "PETR4".to_string();
//!     let req = OrderRequestOpen {
//!         key: OrderKey { exchange: ExchangeId::B3, instrument: &instrument, strategy: StrategyId("s".into()), cid: ClientOrderId("c1".into()) },
//!         state: execution::order::request::RequestOpen { side: markets::Side::Buy, price: dec!(10), quantity: dec!(5), kind: OrderKind::Limit, time_in_force: TimeInForce::GoodUntilEndOfDay }
//!     };
//!     let _maybe_order = client.open_order(req).await; // returns Option<...>
//! }
//! ```
//!
//! See `README.md` for more information and examples.

// Silence transitional unused deps (must appear after inner crate docs)
#[allow(unused_imports)]
use {serde_json as _, toucan_data as _};

use crate::{
    balance::AssetBalance,
    order::{request::OrderResponseCancel, Order, OrderSnapshot},
    trade::Trade,
};
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From};
use order::state::OrderState;
use serde::{Deserialize, Serialize};
use toucan_integration::snapshot::Snapshot;

// Compatibility module for migration
pub mod compat;
pub use compat::*;

pub mod balance;
pub mod client;
pub mod error;
pub mod exchange;
pub mod indexer;
pub mod map;
pub mod order;
pub mod trade;
pub mod transport; // Phase 2: transport abstraction layer (connectivity/protocol)

/// Convenient type alias for an [`AccountEvent`] keyed with [`ExchangeId`],
/// [`AssetNameExchange`], and [`InstrumentNameExchange`].
pub type UnindexedAccountEvent =
    AccountEvent<ExchangeId, AssetNameExchange, InstrumentNameExchange>;

/// Convenient type alias for an [`AccountSnapshot`] keyed with [`ExchangeId`],
/// [`AssetNameExchange`], and [`InstrumentNameExchange`].
pub type UnindexedAccountSnapshot =
    AccountSnapshot<ExchangeId, AssetNameExchange, InstrumentNameExchange>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AccountEvent<
    ExchangeKey = ExchangeIndex,
    AssetKey = AssetIndex,
    InstrumentKey = InstrumentIndex,
> {
    pub exchange: ExchangeKey,
    // Phase 1: optional introduction of Broker/Account (multi-broker)
    pub broker: Option<BrokerId>,
    pub account: Option<AccountId>,
    pub kind: AccountEventKind<ExchangeKey, AssetKey, InstrumentKey>,
}

impl<ExchangeKey, AssetKey, InstrumentKey> AccountEvent<ExchangeKey, AssetKey, InstrumentKey> {
    pub fn new<K>(exchange: ExchangeKey, kind: K) -> Self
    where
        K: Into<AccountEventKind<ExchangeKey, AssetKey, InstrumentKey>>,
    {
        Self {
            exchange,
            broker: None,
            account: None,
            kind: kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, From)]
pub enum AccountEventKind<ExchangeKey, AssetKey, InstrumentKey> {
    /// Full [`AccountSnapshot`] - replaces all existing state.
    Snapshot(AccountSnapshot<ExchangeKey, AssetKey, InstrumentKey>),

    /// Single [`AssetBalance`] snapshot - replaces existing balance state.
    BalanceSnapshot(Snapshot<AssetBalance<AssetKey>>),

    /// Single [`Order`] snapshot - used to upsert existing order state if it's more recent.
    ///
    /// This variant covers general order updates, and open order responses.
    OrderSnapshot(Snapshot<Order<ExchangeKey, InstrumentKey, OrderState<AssetKey, InstrumentKey>>>),

    /// Response to an [`OrderRequestCancel<ExchangeKey, InstrumentKey>`](order::request::OrderRequestOpen).
    OrderCancelled(OrderResponseCancel<ExchangeKey, AssetKey, InstrumentKey>),

    /// [`Order<ExchangeKey, InstrumentKey, Open>`] partial or full-fill.
    Trade(Trade<QuoteAsset, InstrumentKey>),
}

impl<ExchangeKey, AssetKey, InstrumentKey> AccountEvent<ExchangeKey, AssetKey, InstrumentKey>
where
    AssetKey: Eq,
    InstrumentKey: Eq,
{
    pub fn snapshot(self) -> Option<AccountSnapshot<ExchangeKey, AssetKey, InstrumentKey>> {
        match self.kind {
            AccountEventKind::Snapshot(snapshot) => Some(snapshot),
            _ => None,
        }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Constructor,
)]
pub struct AccountSnapshot<
    ExchangeKey = ExchangeIndex,
    AssetKey = AssetIndex,
    InstrumentKey = InstrumentIndex,
> {
    pub exchange: ExchangeKey,
    pub broker: Option<BrokerId>,
    pub account: Option<AccountId>,
    pub balances: Vec<AssetBalance<AssetKey>>,
    pub instruments: Vec<InstrumentAccountSnapshot<ExchangeKey, AssetKey, InstrumentKey>>,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Constructor,
)]
pub struct InstrumentAccountSnapshot<
    ExchangeKey = ExchangeIndex,
    AssetKey = AssetIndex,
    InstrumentKey = InstrumentIndex,
> {
    pub instrument: InstrumentKey,
    #[serde(default = "Vec::new")]
    pub orders: Vec<OrderSnapshot<ExchangeKey, AssetKey, InstrumentKey>>,
}

impl<ExchangeKey, AssetKey, InstrumentKey> AccountSnapshot<ExchangeKey, AssetKey, InstrumentKey> {
    pub fn time_most_recent(&self) -> Option<DateTime<Utc>> {
        let order_times = self.instruments.iter().flat_map(|instrument| {
            instrument
                .orders
                .iter()
                .filter_map(|order| order.state.time_exchange())
        });
        let balance_times = self.balances.iter().map(|balance| balance.time_exchange);

        order_times.chain(balance_times).max()
    }

    pub fn assets(&self) -> impl Iterator<Item = &AssetKey> {
        self.balances.iter().map(|balance| &balance.asset)
    }

    pub fn instruments(&self) -> impl Iterator<Item = &InstrumentKey> {
        self.instruments.iter().map(|snapshot| &snapshot.instrument)
    }
}

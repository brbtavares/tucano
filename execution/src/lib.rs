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

//! DISCLAIMER: Uso experimental/educacional. Não é recomendação de investimento. Veja README e DISCLAIMER.md.
//! # ⚡ Execution - Módulo de Execução de Ordens
//!
//! Stream de dados privados de conta de venues financeiros e execução de ordens
//! (ao vivo ou simuladas). Também fornece MockExchange e MockExecutionClient
//! ricos em recursos para auxiliar backtesting e paper trading.
//!
//! ## 🎯 Características Principais
//!
//! * **🚀 Simplicidade**: Trait ExecutionClient fornece linguagem unificada
//!   e simples para interagir com exchanges
//! * **🔄 Padronização**: Permite que sua estratégia se comunique com qualquer
//!   exchange real ou Mock usando a mesma interface
//! * **🔧 Extensibilidade**: Altamente extensível, facilitando contribuições
//!   com novas integrações de exchanges
//!
//! ## 🏗️ Componentes Principais
//!
//! ### ExecutionClient
//! Interface unificada para execução de ordens em diferentes exchanges.
//! Abaixo um esboço (não compilável) de como uma implementação concreta poderia ficar:
//! ```rust,ignore
//! use execution::client::ExecutionClient;
//! use markets::ExchangeId;
//!
//! #[derive(Clone)]
//! struct MyClient;
//!
//! impl ExecutionClient for MyClient {
//!     const EXCHANGE: ExchangeId = ExchangeId::B3; // exemplo
//!     type Config = ();
//!     type AccountStream = futures::stream::Empty<execution::UnindexedAccountEvent>;
//!     fn new(_: Self::Config) -> Self { Self }
//!     // Demais métodos exigidos pelo trait devem ser implementados...
//!     // fn account_snapshot(..) -> ... { }
//!     // fn open_order(..) -> ... { }
//! }
//! ```
//!
//! ### MockExchange
//! Exchange simulado para backtesting e testes:
//! - **Latência Realística**: Simula delays de rede e processamento
//! - **Slippage**: Modela escorregamento de preços real
//! - **Rejeições**: Simula rejeições por risco ou liquidez
//!
//! ### Gestão de Saldos
//! Sistema robusto para tracking de saldos e posições:
//! - **Multi-Asset**: Suporte a múltiplos ativos simultaneamente
//! - **Real-Time**: Atualizações em tempo real via streams
//! - **Reconciliação**: Validação automática de consistência
//!
//! ## 📊 Exchanges Suportados
//!
//! - **🇧🇷 B3**: Via ProfitDLL da Nelógica
//! - **🌍 Binance**: Spot e Futures
//! - **🇺🇸 Coinbase**: Exchange americano
//! - **🧪 Mock**: Exchange simulado para testes
//!
//! ## 💡 Exemplo de Uso
//!
//! ```rust,ignore
//! // Exemplo conceitual de fluxo (pseudocódigo):
//! use execution::client::ExecutionClient;
//! use execution::order::request::OrderRequestOpen;
//! use execution::order::{OrderKind, TimeInForce, OrderKey};
//! use execution::order::id::{ClientOrderId, StrategyId};
//! use markets::ExchangeId;
//! use rust_decimal_macros::dec;
//!
//! async fn exemplo(mut client: impl ExecutionClient) {
//!     let instrument = "PETR4".to_string();
//!     let req = OrderRequestOpen {
//!         key: OrderKey { exchange: ExchangeId::B3, instrument: &instrument, strategy: StrategyId("s".into()), cid: ClientOrderId("c1".into()) },
//!         state: execution::order::request::RequestOpen { side: markets::Side::Buy, price: dec!(10), quantity: dec!(5), kind: OrderKind::Limit, time_in_force: TimeInForce::GoodUntilEndOfDay }
//!     };
//!     let _maybe_order = client.open_order(req).await; // retorna Option<...>
//! }
//! ```
//!
//! Veja `README.md` para mais informações e exemplos.

// Silence transitional unused deps (must appear after inner crate docs)
#[allow(unused_imports)]
use {tucano_data as _, serde_json as _};

use crate::{
    balance::AssetBalance,
    order::{request::OrderResponseCancel, Order, OrderSnapshot},
    trade::Trade,
};
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From};
use tucano_integration::snapshot::Snapshot;
use order::state::OrderState;
use serde::{Deserialize, Serialize};

// Módulo de compatibilidade para migração
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
    // Fase 1: introdução opcional de Broker/Account (multi-corretora)
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

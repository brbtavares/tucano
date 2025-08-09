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
//! Interface unificada para execução de ordens em diferentes exchanges:
//! ```rust,no_run
//! use execution::client::ExecutionClient;
//!
//! // Implementação para qualquer exchange
//! impl ExecutionClient for MyExchange {
//!     async fn submit_order(&mut self, order: Order) -> Result<OrderAck> {
//!         // Lógica específica do exchange
//!     }
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
//! ```rust,no_run
//! use execution::{
//!     client::ExecutionClient,
//!     order::{Order, OrderKind},
//!     trade::Trade
//! };
//!
//! async fn execute_strategy(client: &mut impl ExecutionClient) {
//!     // Criar ordem de compra
//!     let order = Order::market_buy("PETR4", 100.0);
//!     
//!     // Enviar ordem
//!     match client.submit_order(order).await {
//!         Ok(ack) => println!("Ordem aceita: {:?}", ack),
//!         Err(e) => println!("Erro: {:?}", e),
//!     }
//! }
//! ```
//!
//! Veja `README.md` para mais informações e exemplos.

use crate::{
    balance::AssetBalance,
    order::{request::OrderResponseCancel, Order, OrderSnapshot},
    trade::Trade,
};
use chrono::{DateTime, Utc};
use derive_more::{Constructor, From};
use integration::snapshot::Snapshot;
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
    pub kind: AccountEventKind<ExchangeKey, AssetKey, InstrumentKey>,
}

impl<ExchangeKey, AssetKey, InstrumentKey> AccountEvent<ExchangeKey, AssetKey, InstrumentKey> {
    pub fn new<K>(exchange: ExchangeKey, kind: K) -> Self
    where
        K: Into<AccountEventKind<ExchangeKey, AssetKey, InstrumentKey>>,
    {
        Self {
            exchange,
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

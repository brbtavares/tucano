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

// (moved dummy use below to allow crate-level inner doc comments `//!` to appear before any items)

//! # 🔄 Integration - Framework de Integração de Alta Performance
//!
//! Framework de baixo nível e alta performance para composição de integrações
//! web flexíveis. Utilizado por outras crates do ecossistema Toucan para
//! construir integrações financeiras robustas, principalmente para coleta de
//! dados públicos e execução de trades.
//!
//! ## 🎯 Características Principais
//!
//! * **🔧 Baixo Nível**: Traduz streams de dados brutos comunicados via web
//!   em qualquer modelo de dados desejado usando transformações arbitrárias
//! * **🚀 Flexibilidade**: Compatível com qualquer protocolo (WebSocket, FIX,
//!   Http, etc.), qualquer modelo input/output, e transformações definidas pelo usuário
//!
//! ## 🏗️ Abstrações Fundamentais
//!
//! ### RestClient
//! Comunicação HTTP configurável e assinada entre cliente e servidor:
//! ```rust,no_run
//! use integration::protocol::http::rest::RestClient;
//!
//! let client = RestClient::new()
//!     .with_auth(api_key, secret)
//!     .with_rate_limit(100); // requests per second
//! ```
//!
//! ### ExchangeStream
//! Comunicação configurável sobre protocolos de stream assíncronos:
//! ```rust,no_run
//! use integration::stream::ExchangeStream;
//!
//! let stream = ExchangeStream::new()
//!     .with_reconnect()
//!     .with_heartbeat(30); // seconds
//! ```
//!
//! ## 🌐 Protocolos Suportados
//!
//! - **WebSocket**: Streaming em tempo real
//! - **HTTP REST**: APIs tradicionais
//! - **FIX Protocol**: Protocolo financeiro padrão
//! - **Extensível**: Fácil adição de novos protocolos
//!
//! ## 📊 Funcionalidades de Integração
//!
//! ### Transformação de Dados
//! - **Parser Flexível**: Converte dados de diferentes formatos
//! - **Normalização**: Padroniza dados de múltiplos exchanges
//! - **Validação**: Verificação de integridade em tempo real
//!
//! ### Gestão de Conectividade
//! - **Auto-Reconnect**: Reconexão automática em falhas
//! - **Heartbeat**: Monitoramento de conectividade
//! - **Circuit Breaker**: Proteção contra falhas em cascata
//!
//! ### Métricas e Monitoramento
//! - **Real-Time Metrics**: Métricas de performance em tempo real
//! - **Health Checks**: Verificações de saúde do sistema
//! - **Alerting**: Sistema de alertas para anomalias
//!
//! ## 💡 Exemplo de Uso
//!
//! ```rust,no_run
//! use integration::{
//!     protocol::websocket::WebSocketClient,
//!     subscription::Subscription,
//!     metric::Metric
//! };
//!
//! async fn setup_integration() {
//!     // Configurar cliente WebSocket
//!     let mut ws_client = WebSocketClient::new("wss://exchange.com/ws")
//!         .with_reconnect()
//!         .connect().await?;
//!
//!     // Subscrever dados de mercado
//!     let subscription = Subscription::new("PETR4", "trades");
//!     ws_client.subscribe(subscription).await?;
//!
//!     // Processar dados em tempo real
//!     while let Some(data) = ws_client.next().await {
//!         process_market_data(data);
//!     }
//! }
//! ```
//!
//! Ambas abstrações fornecem a cola robusta necessária para traduzir
//! convenientemente entre modelos de dados de servidor e cliente.

// Silence transitional unused dependency warnings (must appear after inner crate docs)
#[allow(unused_imports)]
use markets as _;

use crate::error::SocketError;
use serde::{Deserialize, Serialize};

/// All [`Error`](std::error::Error)s generated in Integration.
pub mod error;

/// Contains `StreamParser` implementations for transforming communication protocol specific
/// messages into a generic output data structure.
pub mod protocol;

/// Contains the flexible `Metric` type used for representing real-time metrics generically.
pub mod metric;

/// Utilities to assist deserialisation.
pub mod de;

/// Defines a [`SubscriptionId`](subscription::SubscriptionId) new type representing a unique
/// `SmolStr` identifier for a data stream (market data, account data) that has been
/// subscribed to.
pub mod subscription;

/// Defines a trait [`Tx`](channel::Tx) abstraction over different channel kinds, as well as
/// other channel utilities.
///
/// eg/ `UnboundedTx`, `ChannelTxDroppable`, etc.
pub mod channel;

pub mod collection;

/// Stream utilities.
pub mod stream;

pub mod snapshot;

/// [`Validator`]s are capable of determining if their internal state is satisfactory to fulfill
/// some use case defined by the implementor.
pub trait Validator {
    /// Check if `Self` is valid for some use case.
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized;
}

/// [`Transformer`]s are capable of transforming any `Input` into an iterator of
/// `Result<Self::Output, Self::Error>`s.
pub trait Transformer {
    type Error;
    type Input: for<'de> Deserialize<'de>;
    type Output;
    type OutputIter: IntoIterator<Item = Result<Self::Output, Self::Error>>;
    fn transform(&mut self, input: Self::Input) -> Self::OutputIter;
}

/// Determines if something is considered "unrecoverable", such as an unrecoverable error.
///
/// Note that the meaning of [`Unrecoverable`] may vary depending on the context.
pub trait Unrecoverable {
    fn is_unrecoverable(&self) -> bool;
}

/// Trait that communicates if something is terminal (eg/ requires shutdown or restart).
pub trait Terminal {
    fn is_terminal(&self) -> bool;
}

/// Indicates an `Iterator` or `Stream` has ended.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize, Serialize,
)]
pub struct FeedEnded;

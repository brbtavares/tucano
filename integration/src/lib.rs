// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
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

// ...existing code...
//! # 🔄 Integration - High Performance Integration Framework
//!
//! Low-level, high-performance framework for composing flexible web integrations.
//! Used by other crates in the Toucan ecosystem to build robust financial integrations,
//! mainly for public data collection and trade execution.
//!
//! ## 🎯 Main Features
//!
//! * **🔧 Low Level**: Translates raw data streams communicated via web
//!   into any desired data model using arbitrary transformations
//! * **🚀 Flexibility**: Compatible with any protocol (WebSocket, FIX,
//!   Http, etc.), any input/output model, and user-defined transformations
//!
//! ## 🏗️ Core Abstractions
//!
//! ### RestClient
//! Configurable and signed HTTP communication between client and server:
//! ```rust,ignore
//! use integration::protocol::http::rest::RestClient;
//!
//! let client = RestClient::new()
//!     .with_auth(api_key, secret)
//!     .with_rate_limit(100); // requests per second
//! ```
//!
//! ### ExchangeStream
//! Configurable communication over asynchronous stream protocols:
//! ```rust,ignore
//! use integration::stream::ExchangeStream;
//!
//! let stream = ExchangeStream::new()
//!     .with_reconnect()
//!     .with_heartbeat(30); // seconds
//! ```
//!
//! ## 🌐 Supported Protocols
//!
//! - **WebSocket**: Real-time streaming
//! - **HTTP REST**: Traditional APIs
//! - **FIX Protocol**: Standard financial protocol
//! - **Extensible**: Easy addition of new protocols
//!
//! ## 📊 Integration Features
//!
//! ### Data Transformation
//! - **Flexible Parser**: Converts data from different formats
//! - **Normalization**: Standardizes data from multiple exchanges
//! - **Validation**: Real-time integrity checking
//!
//! ### Connectivity Management
//! - **Auto-Reconnect**: Automatic reconnection on failures
//! - **Heartbeat**: Connectivity monitoring
//! - **Circuit Breaker**: Protection against cascading failures
//!
//! ### Metrics and Monitoring
//! - **Real-Time Metrics**: Real-time performance metrics
//! - **Health Checks**: System health checks
//! - **Alerting**: Alert system for anomalies
//!
//! ## 💡 Usage Example
//!
//! ```rust,ignore
//! use integration::{
//!     protocol::websocket::WebSocketClient,
//!     subscription::Subscription,
//!     metric::Metric
//! };
//!
//! async fn setup_integration() {
//!     // Configure WebSocket client
//!     let mut ws_client = WebSocketClient::new("wss://exchange.com/ws")
//!         .with_reconnect()
//!         .connect().await?;
//!
//!     // Subscribe to market data
//!     let subscription = Subscription::new("PETR4", "trades");
//!     ws_client.subscribe(subscription).await?;
//!
//!     // Process real-time data
//!     while let Some(data) = ws_client.next().await {
//!         process_market_data(data);
//!     }
//! }
//! ```
//!
//! Both abstractions provide the robust glue needed to conveniently translate
//! between server and client data models.

// Silence transitional unused dependency warnings (must appear after inner crate docs)
#[allow(unused_imports)]
use tucano_markets as _;

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

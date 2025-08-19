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
//! # 📊 Data - Market Data Streaming Module
//!
//! High-performance library for WebSocket integration, specialized in streaming
//! public market data from leading exchanges - batteries included. Features:
//!
//! ## 🎯 Main Features
//!
//! * **🚀 Simplicity**: Simple interface with [`StreamBuilder`](streams::builder::StreamBuilder)
//!   and [`DynamicStreams`](streams::builder::dynamic::DynamicStreams) for quick setup
//! * **🔄 Standardization**: Unified interface for consuming WebSocket data with a normalized
//!   data model for all exchanges
//! * **⚡ Real-Time**: Real-time WebSocket integrations allowing consumption of
//!   normalized tick-by-tick data
//! * **🔧 Extensibility**: Highly extensible, making it easy to contribute new
//!   exchange integrations
//!
//! ## 🏗️ User API
//!
//! - [`StreamBuilder`](streams::builder::StreamBuilder) to initialize [`MarketStream`]s
//!   of specific data types
//! - [`DynamicStreams`](streams::builder::dynamic::DynamicStreams) to initialize
//!   [`MarketStream`]s of all supported data types simultaneously
//! - Define which market data you want using the [`Subscription`] type
//! - Pass [`Subscription`]s to the [`StreamBuilder::subscribe`](streams::builder::StreamBuilder::subscribe) methods
//!   or [`DynamicStreams::init`](streams::builder::dynamic::DynamicStreams::init)
//! - Each call to [`StreamBuilder::subscribe`](streams::builder::StreamBuilder::subscribe)
//!   (or batch for [`DynamicStreams::init`](streams::builder::dynamic::DynamicStreams::init))
//!   opens a new WebSocket connection to the exchange - full control
//!
//! ## 📈 Supported Exchanges
//!
//! - **🇧🇷 B3**: Brazilian stock exchange via ProfitDLL

// Silence unused dependency warnings for transitional deps (pending removal)
// ...existing code...
use async_trait::async_trait;
// ...existing code...
use std::collections::VecDeque;
use tokio::sync::mpsc;
use tracing::{error, warn};
// ...existing code...

use tucano_markets::exchange::ExchangeId;
#[allow(unused_imports)]
use {itertools as _, reqwest as _, serde_json as _, vecmap as _};

/// All [`Error`](std::error::Error)s generated in Data.
pub mod error;

/// Defines the generic [`MarketEvent<T>`](MarketEvent) used in every [`MarketStream`].
pub mod event;

/// [`Connector`] implementations for each exchange.
//pub mod exchange;

/// High-level API types used for building [`MarketStream`]s from collections
/// of Toucan [`Subscription`]s.
pub mod streams;

/// [`Subscriber`], [`SubscriptionMapper`](subscriber::mapper::SubscriptionMapper) and
/// [`SubscriptionValidator`](subscriber::validator::SubscriptionValidator) traits that define how a
/// [`Connector`] will subscribe to exchange [`MarketStream`]s.
///
/// Standard implementations for subscribing to WebSocket [`MarketStream`]s are included.
pub mod subscriber;

/// Types that communicate the type of each [`MarketStream`] to initialize, and what normalized
/// Toucan output type the exchange will be transformed into.
pub mod subscription;

/// [`InstrumentData`] trait for instrument describing data.
pub mod instrument;

/// [`OrderBook`](books::OrderBook) related types, and utilities for initializing and maintaining
/// a collection of sorted local Instrument [`OrderBook`](books::OrderBook)s
pub mod books;

/// Generic [`ExchangeTransformer`] implementations used by [`MarketStream`]s to translate exchange
/// specific types to normalized Toucan types.
///
/// A standard [`StatelessTransformer`](transformer::stateless::StatelessTransformer) implementation
/// that works for most `Exchange`-`SubscriptionKind` combinations is included.
///
/// Cases that need custom logic, such as fetching initial [`OrderBooksL2`](subscription::book::OrderBooksL2)
/// and [`OrderBooksL3`](subscription::book::OrderBooksL3) snapshots on startup, may require custom
/// [`ExchangeTransformer`] implementations.
pub mod transformer;

/// Defines a generic identification type for the implementor.
pub trait Identifier<T> {
    fn id(&self) -> T;
}

/// [`Stream`] that yields [`Market<Kind>`](MarketEvent) events. The type of [`Market<Kind>`](MarketEvent)
// ...existing code...
#[async_trait]
pub trait MarketStream {
    // Abstract marker trait for market streams.
}

// TODO: Implementação concreta de schedule_pings_to_exchange deve ser feita na crate exchanges
// pub async fn schedule_pings_to_exchange(...) { unimplemented!() }

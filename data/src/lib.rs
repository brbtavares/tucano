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
//! # 📊 Data - Módulo de Streaming de Dados de Mercado
//!
//! Biblioteca de alta performance para integração via WebSocket, especializada em streaming
//! de dados públicos de mercado de exchanges líderes - bateria incluída. Características:
//!
//! ## 🎯 Características Principais
//!
//! * **🚀 Simplicidade**: Interface simples com [`StreamBuilder`](streams::builder::StreamBuilder)
//!   e [`DynamicStreams`](streams::builder::dynamic::DynamicStreams) para configuração rápida
//! * **🔄 Padronização**: Interface unificada para consumo de dados WebSocket com modelo
//!   de dados normalizado para todos os exchanges
//! * **⚡ Tempo Real**: Integrações WebSocket em tempo real permitindo consumo de dados
//!   tick-by-tick normalizados
//! * **🔧 Extensibilidade**: Altamente extensível, facilitando contribuições com novas
//!   integrações de exchanges
//!
//! ## 🏗️ API do Usuário
//!
//! - [`StreamBuilder`](streams::builder::StreamBuilder) para inicializar [`MarketStream`]s
//!   de tipos específicos de dados
//! - [`DynamicStreams`](streams::builder::dynamic::DynamicStreams) para inicializar
//!   [`MarketStream`]s de todos os tipos de dados suportados simultaneamente
//! - Defina quais dados de mercado deseja usando o tipo [`Subscription`]
//! - Passe [`Subscription`]s para os métodos [`StreamBuilder::subscribe`](streams::builder::StreamBuilder::subscribe)
//!   ou [`DynamicStreams::init`](streams::builder::dynamic::DynamicStreams::init)
//! - Cada chamada para [`StreamBuilder::subscribe`](streams::builder::StreamBuilder::subscribe)
//!   (ou batch para [`DynamicStreams::init`](streams::builder::dynamic::DynamicStreams::init))
//!   abre uma nova conexão WebSocket ao exchange - controle total
//!
//! ## 📈 Exchanges Suportados
//!
//! - **🇧🇷 B3**: Bolsa brasileira via ProfitDLL

// Silence unused dependency warnings for transitional deps (pending removal)
use crate::{
    error::DataError,
    event::MarketEvent,
    exchange::{Connector, PingInterval},
    instrument::InstrumentData,
    subscriber::{Subscribed, Subscriber},
    subscription::{Subscription, SubscriptionKind},
    transformer::ExchangeTransformer,
};
use async_trait::async_trait;
use futures::{SinkExt, Stream, StreamExt};
use std::{collections::VecDeque, future::Future};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use tucano_integration::{
    error::SocketError,
    protocol::{
        websocket::{self, WebSocketParser, WsMessage, WsSink, WsStream},
        StreamParser,
    },
    stream::ExchangeStream,
    Transformer,
};
use tucano_markets::exchange::ExchangeId;
#[allow(unused_imports)]
use {itertools as _, reqwest as _, serde_json as _, vecmap as _};

/// All [`Error`](std::error::Error)s generated in Data.
pub mod error;

/// Defines the generic [`MarketEvent<T>`](MarketEvent) used in every [`MarketStream`].
pub mod event;

/// [`Connector`] implementations for each exchange.
pub mod exchange;

/// High-level API types used for building [`MarketStream`]s from collections
/// of Toucan [`Subscription`]s.
pub mod streams;

/// [`Subscriber`], [`SubscriptionMapper`](subscriber::mapper::SubscriptionMapper) and
/// [`SubscriptionValidator`](subscriber::validator::SubscriptionValidator)  traits that define how a
/// [`Connector`] will subscribe to exchange [`MarketStream`]s.
///
/// Standard implementations for subscribing to WebSocket [`MarketStream`]s are included.
pub mod subscriber;

/// Types that communicate the type of each [`MarketStream`] to initialise, and what normalised
/// Toucan output type the exchange will be transformed into.
pub mod subscription;

/// [`InstrumentData`] trait for instrument describing data.
pub mod instrument;

/// [`OrderBook`](books::OrderBook) related types, and utilities for initialising and maintaining
/// a collection of sorted local Instrument [`OrderBook`](books::OrderBook)s
pub mod books;

/// Generic [`ExchangeTransformer`] implementations used by [`MarketStream`]s to translate exchange
/// specific types to normalised Toucan types.
///
/// A standard [`StatelessTransformer`](transformer::stateless::StatelessTransformer) implementation
/// that works for most `Exchange`-`SubscriptionKind` combinations is included.
///
/// Cases that need custom logic, such as fetching initial [`OrderBooksL2`](subscription::book::OrderBooksL2)
/// and [`OrderBooksL3`](subscription::book::OrderBooksL3) snapshots on startup, may require custom
/// [`ExchangeTransformer`] implementations.
pub mod transformer;

/// Convenient type alias for an [`ExchangeStream`] utilising a tungstenite
/// [`WebSocket`](tucano_integration::protocol::websocket::WebSocket).
pub type ExchangeWsStream<Transformer> = ExchangeStream<WebSocketParser, WsStream, Transformer>;

/// Defines a generic identification type for the implementor.
pub trait Identifier<T> {
    fn id(&self) -> T;
}

/// [`Stream`] that yields [`Market<Kind>`](MarketEvent) events. The type of [`Market<Kind>`](MarketEvent)
/// depends on the provided [`SubscriptionKind`] of the passed [`Subscription`]s.
#[async_trait]
pub trait MarketStream<Exchange, Instrument, Kind>
where
    Self: Stream<Item = Result<MarketEvent<Instrument::Key, Kind::Event>, DataError>>
        + Send
        + Sized
        + Unpin,
    Exchange: Connector,
    Instrument: InstrumentData,
    Kind: SubscriptionKind,
{
    async fn init<SnapFetcher>(
        subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> Result<Self, DataError>
    where
        SnapFetcher: SnapshotFetcher<Exchange, Kind>,
        Subscription<Exchange, Instrument, Kind>:
            Identifier<Exchange::Channel> + Identifier<Exchange::Market>;
}

/// Defines how to fetch market data snapshots for a collection of [`Subscription`]s.
///
/// Useful when a [`MarketStream`] requires an initial snapshot on start-up.
pub trait SnapshotFetcher<Exchange, Kind> {
    fn fetch_snapshots<Instrument>(
        subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> impl Future<Output = Result<Vec<MarketEvent<Instrument::Key, Kind::Event>>, SocketError>> + Send
    where
        Exchange: Connector,
        Instrument: InstrumentData,
        Kind: SubscriptionKind,
        Kind::Event: Send,
        Subscription<Exchange, Instrument, Kind>: Identifier<Exchange::Market>;
}

#[async_trait]
impl<Exchange, Instrument, Kind, Transformer> MarketStream<Exchange, Instrument, Kind>
    for ExchangeWsStream<Transformer>
where
    Exchange: Connector + Send + Sync,
    Instrument: InstrumentData,
    Kind: SubscriptionKind + Send + Sync,
    Transformer: ExchangeTransformer<Exchange, Instrument::Key, Kind> + Send,
    Kind::Event: Send,
{
    async fn init<SnapFetcher>(
        subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> Result<Self, DataError>
    where
        SnapFetcher: SnapshotFetcher<Exchange, Kind>,
        Subscription<Exchange, Instrument, Kind>:
            Identifier<Exchange::Channel> + Identifier<Exchange::Market>,
    {
        // Connect & subscribe
        let Subscribed {
            websocket,
            map: instrument_map,
            buffered_websocket_events,
        } = Exchange::Subscriber::subscribe(subscriptions).await?;

        // Fetch any required initial MarketEvent snapshots
        let initial_snapshots = SnapFetcher::fetch_snapshots(subscriptions).await?;

        // Split WebSocket into WsStream & WsSink components
        let (ws_sink, ws_stream) = websocket.split();

        // Spawn task to distribute Transformer messages (eg/ custom pongs) to the exchange
        let (ws_sink_tx, ws_sink_rx) = mpsc::unbounded_channel();
        tokio::spawn(distribute_messages_to_exchange(
            Exchange::ID,
            ws_sink,
            ws_sink_rx,
        ));

        // Spawn optional task to distribute custom application-level pings to the exchange
        if let Some(ping_interval) = Exchange::ping_interval() {
            tokio::spawn(schedule_pings_to_exchange(
                Exchange::ID,
                ws_sink_tx.clone(),
                ping_interval,
            ));
        }

        // Initialise Transformer associated with this Exchange and SubscriptionKind
        let mut transformer =
            Transformer::init(instrument_map, &initial_snapshots, ws_sink_tx).await?;

        // Process any buffered active subscription events received during Subscription validation
        let mut processed = process_buffered_events::<WebSocketParser, _>(
            &mut transformer,
            buffered_websocket_events,
        );

        // Extend buffered events with any initial snapshot events
        processed.extend(initial_snapshots.into_iter().map(Ok));

        Ok(ExchangeWsStream::new(ws_stream, transformer, processed))
    }
}

/// Implementation of [`SnapshotFetcher`] that does not fetch any initial market data snapshots.
/// Often used for stateless [`MarketStream`]s, such as public trades.
#[derive(Debug)]
pub struct NoInitialSnapshots;

impl<Exchange, Kind> SnapshotFetcher<Exchange, Kind> for NoInitialSnapshots {
    fn fetch_snapshots<Instrument>(
        _: &[Subscription<Exchange, Instrument, Kind>],
    ) -> impl Future<Output = Result<Vec<MarketEvent<Instrument::Key, Kind::Event>>, SocketError>> + Send
    where
        Exchange: Connector,
        Instrument: InstrumentData,
        Kind: SubscriptionKind,
        Kind::Event: Send,
        Subscription<Exchange, Instrument, Kind>: Identifier<Exchange::Market>,
    {
        std::future::ready(Ok(vec![]))
    }
}

pub fn process_buffered_events<Protocol, StreamTransformer>(
    transformer: &mut StreamTransformer,
    events: Vec<Protocol::Message>,
) -> VecDeque<Result<StreamTransformer::Output, StreamTransformer::Error>>
where
    Protocol: StreamParser,
    StreamTransformer: Transformer,
{
    events
        .into_iter()
        .filter_map(|event| {
            Protocol::parse::<StreamTransformer::Input>(Ok(event))?
                .inspect_err(|error| {
                    warn!(
                        ?error,
                        "failed to parse message buffered during Subscription validation"
                    )
                })
                .ok()
        })
        .flat_map(|parsed| transformer.transform(parsed))
        .collect()
}

/// Transmit [`WsMessage`]s sent from the [`ExchangeTransformer`] to the exchange via
/// the [`WsSink`].
///
/// **Note:**
/// ExchangeTransformer is operating in a synchronous trait context so we use this separate task
/// to avoid adding `#[\async_trait\]` to the transformer - this avoids allocations.
pub async fn distribute_messages_to_exchange(
    exchange: ExchangeId,
    mut ws_sink: WsSink,
    mut ws_sink_rx: mpsc::UnboundedReceiver<WsMessage>,
) {
    while let Some(message) = ws_sink_rx.recv().await {
        if let Err(error) = ws_sink.send(message).await {
            if websocket::is_websocket_disconnected(&error) {
                break;
            }

            // Log error only if WsMessage failed to send over a connected WebSocket
            error!(
                %exchange,
                %error,
                "failed to send output message to the exchange via WsSink"
            );
        }
    }
}

/// Schedule the sending of custom application-level ping [`WsMessage`]s to the exchange using
/// the provided [`PingInterval`].
///
/// **Notes:**
///  - This is only used for those exchanges that require custom application-level pings.
///  - This is additional to the protocol-level pings already handled by `tokio_tungstenite`.
pub async fn schedule_pings_to_exchange(
    exchange: ExchangeId,
    ws_sink_tx: mpsc::UnboundedSender<WsMessage>,
    PingInterval { mut interval, ping }: PingInterval,
) {
    loop {
        // Wait for next scheduled ping
        interval.tick().await;

        // Construct exchange custom application-level ping payload
        let payload = ping();
        debug!(%exchange, %payload, "sending custom application-level ping to exchange");

        if ws_sink_tx.send(payload).is_err() {
            break;
        }
    }
}

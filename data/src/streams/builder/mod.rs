// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use super::Streams;
use crate::{
    error::DataError,
    // ...existing code...
    instrument::InstrumentData,
    streams::{
        consumer::{init_market_stream, MarketStreamResult, STREAM_RECONNECTION_POLICY},
        reconnect::stream::ReconnectingStream,
    },
    subscription::{Subscription, SubscriptionKind},
    Identifier,
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
};
use tucano_integration::{channel::Channel, Validator};
use tucano_markets::exchange::ExchangeId;

/// Defines the [`MultiStreamBuilder`](multi::MultiStreamBuilder) API for ergonomically
/// initialising a common [`Streams<Output>`](Streams) from multiple
/// [`StreamBuilder<SubscriptionKind>`](StreamBuilder)s.
pub mod multi;

/// Communicative type alias representing the [`Future`] result of a [`Subscription`] validation
/// call generated whilst executing [`StreamBuilder::subscribe`].
pub type SubscribeFuture = Pin<Box<dyn Future<Output = Result<(), DataError>>>>;

/// Builder to configure and initialise a [`Streams<MarketEvent<SubscriptionKind::Event>`](Streams) instance
/// for a specific [`SubscriptionKind`].
#[derive(Default)]
pub struct StreamBuilder {
    pub channels: HashMap<ExchangeId, ()>,
    pub futures: Vec<SubscribeFuture>,
    // ...existing code...

    // ...existing code...

    // ...existing code...
}

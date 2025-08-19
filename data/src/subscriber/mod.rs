// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use self::{
    mapper::{SubscriptionMapper, WebSocketSubMapper},
    validator::SubscriptionValidator,
};
use crate::{
    // ...existing code...
    instrument::InstrumentData,
    subscription::{Map, Subscription, SubscriptionKind, SubscriptionMeta},
    Identifier,
};
use async_trait::async_trait;
use futures::SinkExt;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::debug;
use tucano_integration::{
    error::SocketError,
    protocol::websocket::{connect, WebSocket, WsMessage},
};

/// [`SubscriptionMapper`] implementations defining how to map a
/// collection of Toucan [`Subscription`]s into exchange specific [`SubscriptionMeta`].
pub mod mapper;

/// [`SubscriptionValidator`] implementations defining how to
/// validate actioned [`Subscription`]s were successful.
pub mod validator;

/// Defines how to connect to a socket and subscribe to market data streams.
#[async_trait]
pub trait Subscriber {
    type SubMapper: SubscriptionMapper;

    async fn subscribe<Exchange, Instrument, Kind>(
        subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> Result<Subscribed, SocketError>;
}

#[derive(Debug)]
pub struct Subscribed;

/// Standard [`Subscriber`] for [`WebSocket`]s suitable for most exchanges.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct WebSocketSubscriber;

#[async_trait]
impl Subscriber for WebSocketSubscriber {
    type SubMapper = WebSocketSubMapper;

    async fn subscribe<Exchange, Instrument, Kind>(
        _subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> Result<Subscribed, SocketError> {
        // TODO: Implementação concreta de Subscriber::subscribe deve ser feita na crate exchanges
        unimplemented!("A implementação concreta de Subscriber::subscribe deve ser feita na crate exchanges.");
    }
}

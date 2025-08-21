
use crate::{
    // ...existing code...
    subscription::{Map, SubscriptionKind},
};
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::debug;
use toucan_integration::{
    error::SocketError,
    protocol::{
        websocket::{WebSocket, WebSocketParser, WsMessage},
        StreamParser,
    },
    Validator,
};

/// Defines how to validate that actioned market data
/// [`Subscription`](crate::subscription::Subscription)s were accepted by the exchange.
#[async_trait]
pub trait SubscriptionValidator {
    type Parser: StreamParser;

    async fn validate<Exchange, Kind>(
        instrument_map: Map<()>,
        websocket: &mut WebSocket,
    ) -> Result<(Map<()>, Vec<WsMessage>), SocketError>;
}

/// Standard [`SubscriptionValidator`] for [`WebSocket`]s suitable for most exchanges.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct WebSocketSubValidator;

#[async_trait]
impl SubscriptionValidator for WebSocketSubValidator {
    type Parser = WebSocketParser;

    async fn validate<Exchange, Kind>(
        _instrument_map: Map<()>,
        _websocket: &mut WebSocket,
    ) -> Result<(Map<()>, Vec<WsMessage>), SocketError> {
    // TODO: Concrete implementation of SubscriptionValidator::validate must be done in the exchanges crate
    unimplemented!("The concrete implementation of SubscriptionValidator::validate must be done in the exchanges crate.");
    }
}

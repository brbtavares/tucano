// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::{
    // ...existing code...
    instrument::InstrumentData,
    subscription::{Map, Subscription, SubscriptionKind, SubscriptionMeta},
    Identifier,
};
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use tucano_integration::subscription::SubscriptionId;

/// Defines how to map a collection of Toucan [`Subscription`]s into exchange specific
/// [`SubscriptionMeta`], containing subscription payloads that are sent to the exchange.
pub trait SubscriptionMapper {
    fn map<Exchange, Instrument, Kind>(
        subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> SubscriptionMeta<()>;
}

/// Standard [`SubscriptionMapper`] for
/// [`WebSocket`](integration::protocol::websocket::WebSocket)s suitable for most exchanges.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct WebSocketSubMapper;

impl SubscriptionMapper for WebSocketSubMapper {
    fn map<Exchange, Instrument, Kind>(
        _subscriptions: &[Subscription<Exchange, Instrument, Kind>],
    ) -> SubscriptionMeta<()> {
        // TODO: Implementação concreta de SubscriptionMapper::map deve ser feita na crate exchanges
        unimplemented!("A implementação concreta de SubscriptionMapper::map deve ser feita na crate exchanges.");
    }
}

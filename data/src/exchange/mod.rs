use self::subscription::ExchangeSub;
use crate::{
    instrument::InstrumentData,
    subscriber::{validator::SubscriptionValidator, Subscriber},
    subscription::{Map, SubscriptionKind},
    MarketStream, SnapshotFetcher,
};
use tucano_integration::{error::SocketError, protocol::websocket::WsMessage, Validator};
use tucano_markets::exchange::ExchangeId;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
use url::Url;

/// `B3` [`Connector`] and [`StreamSelector`] implementations for Brazilian stock exchange.
pub mod b3;

/// Defines the generic [`ExchangeSub`] containing a market and channel combination used by an
/// exchange [`Connector`] to build [`WsMessage`] subscription payloads.
pub mod subscription;

/// Default [`Duration`] the [`Connector::SubValidator`] will wait to receive all success responses to actioned
/// `Subscription` requests.
pub const DEFAULT_SUBSCRIPTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Defines the [`MarketStream`] kind associated with an exchange
/// `Subscription` [`SubscriptionKind`].
///
/// ### Notes
/// Must be implemented by an exchange [`Connector`] if it supports a specific
/// [`SubscriptionKind`].
pub trait StreamSelector<Instrument, Kind>
where
    Self: Connector,
    Instrument: InstrumentData,
    Kind: SubscriptionKind,
{
    type SnapFetcher: SnapshotFetcher<Self, Kind>;
    type Stream: MarketStream<Self, Instrument, Kind>;
}

/// Primary exchange abstraction. Defines how to translate Toucan types into exchange specific
/// types, as well as connecting, subscribing, and interacting with the exchange server.
///
/// ### Notes
/// This must be implemented for a new exchange integration!
pub trait Connector
where
    Self: Clone + Default + Debug + for<'de> Deserialize<'de> + Serialize + Sized,
{
    /// Unique identifier for the exchange server being connected with.
    const ID: ExchangeId;

    /// Type that defines how to translate a Toucan `Subscription` into an exchange specific
    /// channel to be subscribed to.
    type Channel: AsRef<str>;

    /// Type that defines how to translate a Toucan
    /// `Subscription` into an exchange specific market that
    /// can be subscribed to.
    type Market: AsRef<str>;

    /// [`Subscriber`] type that establishes a connection with the exchange server, and actions
    /// `Subscription`s over the socket.
    type Subscriber: Subscriber;

    /// [`SubscriptionValidator`] type that listens to responses from the exchange server and
    /// validates if the actioned `Subscription`s were
    /// successful.
    type SubValidator: SubscriptionValidator;

    /// Deserialisable type that the [`Self::SubValidator`] expects to receive from the exchange server in
    /// response to the `Subscription` [`Self::requests`]
    /// sent over the [`WebSocket`](tucano_integration::protocol::websocket::WebSocket). Implements
    /// [`Validator`] in order to determine if [`Self`]
    /// communicates a successful `Subscription` outcome.
    type SubResponse: Validator + Debug + DeserializeOwned;

    /// Base [`Url`] of the exchange server being connected with.
    fn url() -> Result<Url, SocketError>;

    /// Defines [`PingInterval`] of custom application-level
    /// [`WebSocket`](tucano_integration::protocol::websocket::WebSocket) pings for the exchange
    /// server being connected with.
    ///
    /// Defaults to `None`, meaning that no custom pings are sent.
    fn ping_interval() -> Option<PingInterval> {
        None
    }

    /// Defines how to translate a collection of [`ExchangeSub`]s into the [`WsMessage`]
    /// subscription payloads sent to the exchange server.
    fn requests(exchange_subs: Vec<ExchangeSub<Self::Channel, Self::Market>>) -> Vec<WsMessage>;

    /// Number of `Subscription` responses expected from the
    /// exchange server in responses to the requests send. Used to validate all
    /// `Subscription`s were accepted.
    fn expected_responses<InstrumentKey>(map: &Map<InstrumentKey>) -> usize {
        map.0.len()
    }

    /// Expected [`Duration`] the [`SubscriptionValidator`] will wait to receive all success
    /// responses to actioned `Subscription` requests.
    fn subscription_timeout() -> Duration {
        DEFAULT_SUBSCRIPTION_TIMEOUT
    }
}

/// Used when an exchange has servers different
/// [`InstrumentKind`](toucan_instrument::instrument::kind::InstrumentKind) market data on distinct servers,
/// allowing all the [`Connector`] logic to be identical apart from what this trait provides.
pub trait ExchangeServer: Default + Debug + Clone + Send {
    const ID: ExchangeId;
    fn websocket_url() -> &'static str;
}

/// Defines the frequency and construction function for custom
/// [`WebSocket`](tucano_integration::protocol::websocket::WebSocket) pings - used for exchanges
/// that require additional application-level pings.
#[derive(Debug)]
pub struct PingInterval {
    pub interval: tokio::time::Interval,
    pub ping: fn() -> WsMessage,
}

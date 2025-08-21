
use crate::{
    error::DataError,
    event::MarketEvent,
    subscription::{Map, SubscriptionKind},
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use toucan_integration::{protocol::websocket::WsMessage, Transformer};

/// Generic stateless [`ExchangeTransformer`] often used for transforming
/// [`PublicTrades`](crate::subscription::trade::PublicTrades) streams.
pub mod stateless;

/// Defines how to construct a [`Transformer`] used by [`MarketStream`](super::MarketStream)s to
/// translate exchange specific types to normalised Toucan types.
#[async_trait]
pub trait ExchangeTransformer<Exchange, InstrumentKey, Kind>
where
    Self: Transformer<
            Output = MarketEvent<InstrumentKey, <Kind as SubscriptionKind>::Event>,
            Error = DataError,
        > + Sized,
    Kind: SubscriptionKind,
{
    /// Initialise a new [`Self`], also fetching any market data snapshots required for the
    /// associated Exchange and SubscriptionKind market stream to function.
    ///
    /// The [`mpsc::UnboundedSender`] can be used by [`Self`] to send messages back to the exchange.
    async fn init(
        instrument_map: Map<InstrumentKey>,
        initial_snapshots: &[MarketEvent<InstrumentKey, <Kind as SubscriptionKind>::Event>],
        ws_sink_tx: mpsc::UnboundedSender<WsMessage>,
    ) -> Result<Self, DataError>;
}

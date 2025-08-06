use crate::{Identifier, subscription::Subscription};
use integration::subscription::SubscriptionId;
use serde::Deserialize;

/// Defines an exchange specific market and channel combination used by an exchange
/// [`Connector`](super::Connector) to build the
/// [`WsMessage`](integration::protocol::websocket::WsMessage) subscription payloads to
/// send to the exchange server.
///
/// ### Examples
/// #### Binance OrderBooksL2
/// ```json
/// ExchangeSub {
///     channel: BinanceChannel("@depth@100ms"),
///     market: BinanceMarket("btcusdt"),
/// }
/// ```
/// #### B3 Trades
/// ```rust,ignore
/// ExchangeSub {
///     channel: B3SubKind::Trades,
///     market: "PETR4@BOVESPA"
/// }
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize)]
pub struct ExchangeSub<Channel, Market> {
    /// Type that defines how to translate a Toucan [`Subscription`] into an exchange specific
    /// channel to be subscribed to.
    ///
    /// ### Examples
    /// - [`BinanceChannel("@depth@100ms")`](super::binance::channel::BinanceChannel)
    /// - [`BinanceChannel("trade")`](super::binance::channel::BinanceChannel)
    pub channel: Channel,

    /// Type that defines how to translate a Toucan [`Subscription`] into an exchange specific
    /// market that can be subscribed to.
    ///
    /// ### Examples
    /// - [`BinanceMarket("btcusdt")`](super::binance::market::BinanceMarket)
    /// - [`BinanceMarket("BTCUSDT")`](super::binance::market::BinanceMarket)
    pub market: Market,
}

impl<Channel, Market> Identifier<SubscriptionId> for ExchangeSub<Channel, Market>
where
    Channel: AsRef<str>,
    Market: AsRef<str>,
{
    fn id(&self) -> SubscriptionId {
        SubscriptionId::from(format!(
            "{}|{}",
            self.channel.as_ref(),
            self.market.as_ref()
        ))
    }
}

impl<Channel, Market> ExchangeSub<Channel, Market>
where
    Channel: AsRef<str>,
    Market: AsRef<str>,
{
    /// Construct a new exchange specific [`Self`] with the Toucan [`Subscription`] provided.
    pub fn new<Exchange, Instrument, Kind>(sub: &Subscription<Exchange, Instrument, Kind>) -> Self
    where
        Subscription<Exchange, Instrument, Kind>: Identifier<Channel> + Identifier<Market>,
    {
        Self {
            channel: sub.id(),
            market: sub.id(),
        }
    }
}

impl<Channel, Market> From<(Channel, Market)> for ExchangeSub<Channel, Market>
where
    Channel: AsRef<str>,
    Market: AsRef<str>,
{
    fn from((channel, market): (Channel, Market)) -> Self {
        Self { channel, market }
    }
}

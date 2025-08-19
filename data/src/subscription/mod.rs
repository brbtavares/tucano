// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::instrument::InstrumentData;
use derive_more::Display;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use smol_str::{format_smolstr, ToSmolStr};
use std::{borrow::Borrow, fmt::Debug, hash::Hash};
use tucano_integration::{
    error::SocketError, protocol::websocket::WsMessage, subscription::SubscriptionId, Validator,
};
use tucano_instrument::{exchange::ExchangeId, InstrumentKind, Keyed, MarketDataInstrument};

/// OrderBook [`SubscriptionKind`]s and the associated Toucan output data models.
pub mod book;

/// Candle [`SubscriptionKind`] and the associated Toucan output data model.
pub mod candle;

/// Liquidation [`SubscriptionKind`] and the associated Toucan output data model.
pub mod liquidation;

/// Public trade [`SubscriptionKind`] and the associated Toucan output data model.
pub mod trade;

/// Defines kind of a [`Subscription`], and the output [`Self::Event`] that it yields.
pub trait SubscriptionKind
where
    Self: Debug + Clone,
{
    type Event: Debug;
    fn as_str(&self) -> &'static str;
}

/// Toucan [`Subscription`] used to subscribe to a [`SubscriptionKind`] for a particular exchange
/// [`MarketDataInstrument`].
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct Subscription<Exchange = ExchangeId, Inst = MarketDataInstrument, Kind = SubKind> {
    pub exchange: Exchange,
    #[serde(flatten)]
    pub instrument: Inst,
    #[serde(alias = "type")]
    pub kind: Kind,
}

pub fn display_subscriptions_without_exchange<Exchange, Instrument, Kind>(
    subscriptions: &[Subscription<Exchange, Instrument, Kind>],
) -> String
where
    Instrument: std::fmt::Display,
    Kind: std::fmt::Display,
{
    subscriptions
        .iter()
        .map(
            |Subscription {
                 exchange: _,
                 instrument,
                 kind,
             }| { format_smolstr!("({instrument}, {kind})") },
        )
        .collect::<Vec<_>>()
        .join(",")
}

impl<Exchange, Instrument, Kind> std::fmt::Display for Subscription<Exchange, Instrument, Kind>
where
    Exchange: std::fmt::Display,
    Instrument: std::fmt::Display,
    Kind: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}|{}|{})", self.exchange, self.kind, self.instrument)
    }
}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, Deserialize, Serialize,
)]
pub enum SubKind {
    PublicTrades,
    OrderBooksL1,
    OrderBooksL2,
    OrderBooksL3,
    Liquidations,
    Candles,
}

impl<Exchange, S, Kind> From<(Exchange, S, S, InstrumentKind, Kind)>
    for Subscription<Exchange, MarketDataInstrument, Kind>
where
    S: Into<String>,
{
    fn from(
        (exchange, base, quote, instrument_kind, kind): (Exchange, S, S, InstrumentKind, Kind),
    ) -> Self {
        Subscription {
            exchange,
            instrument: (base, quote, instrument_kind).into(),
            kind,
        }
    }
}

impl<InstrumentKey, Exchange, S, Kind> From<(InstrumentKey, Exchange, S, S, InstrumentKind, Kind)>
    for Subscription<Exchange, Keyed<InstrumentKey, MarketDataInstrument>, Kind>
where
    S: Into<String>,
{
    fn from(
        (instrument_id, exchange, base, quote, instrument_kind, kind): (
            InstrumentKey,
            Exchange,
            S,
            S,
            InstrumentKind,
            Kind,
        ),
    ) -> Self {
        let instrument = Keyed::new(instrument_id, (base, quote, instrument_kind).into());

        Subscription {
            exchange,
            instrument: instrument.into(),
            kind,
        }
    }
}

impl<Exchange, I, Instrument, Kind> From<(Exchange, I, Kind)>
    for Subscription<Exchange, Instrument, Kind>
where
    I: Into<Instrument>,
{
    fn from((exchange, instrument, kind): (Exchange, I, Kind)) -> Self {
        Subscription {
            exchange,
            instrument: instrument.into(),
            kind,
        }
    }
}

/// Determines whether the [`Connector`] associated with this [`ExchangeId`] supports the
/// ingestion of market data for the provided [`InstrumentKind`].
#[allow(clippy::match_like_matches_macro)]
pub fn exchange_supports_instrument_kind(
    exchange: ExchangeId,
    instrument_kind: &InstrumentKind,
) -> bool {
    match (exchange, instrument_kind) {
        // Spot
        (_, InstrumentKind::Spot) => true,

        // Future - futures markets only
        (_, InstrumentKind::Future(_)) => false,

        // Perpetual - only futures exchanges
        (_, InstrumentKind::Perpetual(_)) => false,

        // Option - no supported options exchanges currently
        (_, InstrumentKind::Option(_)) => false,
    }
}

impl<Instrument> Validator for Subscription<ExchangeId, Instrument, SubKind>
where
    Instrument: InstrumentData,
{
    fn validate(self) -> Result<Self, SocketError>
    where
        Self: Sized,
    {
        // Validate the Exchange supports the Subscription InstrumentKind
        if exchange_supports_instrument_kind_sub_kind(
            &self.exchange,
            self.instrument.kind(),
            self.kind,
        ) {
            Ok(self)
        } else {
            Err(SocketError::Unsupported {
                entity: self.exchange.to_string(),
                item: format!("({}, {})", self.instrument.kind(), self.kind),
            })
        }
    }
}

/// Determines whether the [`Connector`] associated with this [`ExchangeId`] supports the
/// ingestion of market data for the provided [`InstrumentKind`] and [`SubKind`] combination.
pub fn exchange_supports_instrument_kind_sub_kind(
    exchange_id: &ExchangeId,
    instrument_kind: &InstrumentKind,
    sub_kind: SubKind,
) -> bool {
    match (exchange_id, instrument_kind, sub_kind) {
        // Spot exchanges
        (ExchangeId::B3, InstrumentKind::Spot, SubKind::PublicTrades | SubKind::OrderBooksL1) => true,

        _ => false,
    }
}

/// Metadata generated from a collection of Toucan [`Subscription`]s, including the exchange
/// specific subscription payloads that are sent to the exchange.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SubscriptionMeta<InstrumentKey> {
    /// `HashMap` containing the mapping between a [`SubscriptionId`] and
    /// it's associated Toucan [`MarketDataInstrument`].
    pub instrument_map: Map<InstrumentKey>,
    /// Collection of [`WsMessage`]s containing exchange specific subscription payloads to be sent.
    pub ws_subscriptions: Vec<WsMessage>,
}

/// New type`HashMap` that maps a [`SubscriptionId`] to some associated type `T`.
///
/// Used by [`ExchangeTransformer`](crate::transformer::ExchangeTransformer)s to identify the
/// Toucan [`MarketDataInstrument`] associated with incoming exchange messages.
#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
pub struct Map<T>(pub FnvHashMap<SubscriptionId, T>);

impl<T> FromIterator<(SubscriptionId, T)> for Map<T> {
    fn from_iter<Iter>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = (SubscriptionId, T)>,
    {
        Self(iter.into_iter().collect::<FnvHashMap<SubscriptionId, T>>())
    }
}

impl<T> Map<T> {
    /// Find the `InstrumentKey` associated with the provided [`SubscriptionId`].
    pub fn find<SubId>(&self, id: &SubId) -> Result<&T, SocketError>
    where
        SubscriptionId: Borrow<SubId>,
        SubId: AsRef<str> + Hash + Eq + ?Sized,
    {
        self.0
            .get(id)
            .ok_or_else(|| SocketError::Unidentifiable(SubscriptionId(id.as_ref().to_smolstr())))
    }

    /// Find the mutable reference to `T` associated with the provided [`SubscriptionId`].
    pub fn find_mut<SubId>(&mut self, id: &SubId) -> Result<&mut T, SocketError>
    where
        SubscriptionId: Borrow<SubId>,
        SubId: AsRef<str> + Hash + Eq + ?Sized,
    {
        self.0
            .get_mut(id)
            .ok_or_else(|| SocketError::Unidentifiable(SubscriptionId(id.as_ref().to_smolstr())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod subscription {
        use super::*; // brings Map, SubscriptionId, InstrumentKind
    use tucano_instrument::MarketDataInstrument;

        // Removed nested module with unused imports (B3Exchange, OrderBooksL2, PublicTrades)

        #[test]
        fn test_find_instrument() {
            // Initialise SubscriptionId-InstrumentKey HashMap
            let ids = Map(FnvHashMap::from_iter([(
                SubscriptionId::from("present"),
                MarketDataInstrument::from(("base", "quote", InstrumentKind::Spot)),
            )]));

            struct TestCase {
                input: SubscriptionId,
                expected: Result<MarketDataInstrument, SocketError>,
            }

            let cases = vec![
                TestCase {
                    // TC0: SubscriptionId (channel) is present in the HashMap
                    input: SubscriptionId::from("present"),
                    expected: Ok(MarketDataInstrument::from((
                        "base",
                        "quote",
                        InstrumentKind::Spot,
                    ))),
                },
                TestCase {
                    // TC1: SubscriptionId (channel) is not present in the HashMap
                    input: SubscriptionId::from("not present"),
                    expected: Err(SocketError::Unidentifiable(SubscriptionId::from(
                        "not present",
                    ))),
                },
            ];

            for (index, test) in cases.into_iter().enumerate() {
                let actual = ids.find(&test.input);
                match (actual, test.expected) {
                    (Ok(actual), Ok(expected)) => {
                        assert_eq!(*actual, expected, "TC{index} failed")
                    }
                    (Err(_), Err(_)) => {
                        // Test passed
                    }
                    (actual, expected) => {
                        // Test failed
                        panic!(
                            "TC{index} failed because actual != expected. \nActual: {actual:?}\nExpected: {expected:?}\n"
                        );
                    }
                }
            }
        }
    }
}

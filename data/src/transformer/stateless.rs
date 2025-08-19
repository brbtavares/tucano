// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use super::ExchangeTransformer;
use crate::{
    error::DataError,
    event::{MarketEvent, MarketIter},
    // ...existing code...
    subscription::{Map, SubscriptionKind},
    Identifier,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::marker::PhantomData;
use tokio::sync::mpsc;
use tucano_instrument::ExchangeId;
use tucano_integration::{
    protocol::websocket::WsMessage, subscription::SubscriptionId, Transformer,
};

/// Standard generic stateless [`ExchangeTransformer`] to translate exchange specific types into
/// normalised Toucan types. Often used with
/// [`PublicTrades`](crate::subscription::trade::PublicTrades) or
/// [`OrderBooksL1`](crate::subscription::book::OrderBooksL1) streams.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StatelessTransformer<Exchange, InstrumentKey, Kind, Input> {
    instrument_map: Map<InstrumentKey>,
    phantom: PhantomData<(Exchange, Kind, Input)>,
}

#[async_trait]
impl<Exchange, InstrumentKey, Kind, Input> ExchangeTransformer<Exchange, InstrumentKey, Kind>
    for StatelessTransformer<Exchange, InstrumentKey, Kind, Input>
where
    InstrumentKey: Clone + Send,
    Kind: SubscriptionKind + Send,
    Input: Identifier<Option<SubscriptionId>> + for<'de> Deserialize<'de>,
    MarketIter<InstrumentKey, <Kind as SubscriptionKind>::Event>:
        From<(ExchangeId, InstrumentKey, Input)>,
{
    async fn init(
        instrument_map: Map<InstrumentKey>,
        _: &[MarketEvent<InstrumentKey, Kind::Event>],
        _: mpsc::UnboundedSender<WsMessage>,
    ) -> Result<Self, DataError> {
        Ok(Self {
            instrument_map,
            phantom: PhantomData,
        })
    }
}

impl<Exchange, InstrumentKey, Kind, Input> Transformer
    for StatelessTransformer<Exchange, InstrumentKey, Kind, Input>
where
    InstrumentKey: Clone,
    Kind: SubscriptionKind,
    Input: Identifier<Option<SubscriptionId>> + for<'de> Deserialize<'de>,
    MarketIter<InstrumentKey, <Kind as SubscriptionKind>::Event>:
        From<(ExchangeId, InstrumentKey, Input)>,
{
    type Error = DataError;
    type Input = Input;
    type Output = MarketEvent<InstrumentKey, <Kind as SubscriptionKind>::Event>;
    type OutputIter = Vec<Result<Self::Output, Self::Error>>;

    fn transform(&mut self, input: Self::Input) -> Self::OutputIter {
        // Determine if the message has an identifiable SubscriptionId
        let subscription_id = match input.id() {
            Some(subscription_id) => subscription_id,
            None => return vec![],
        };

        // Find Instrument associated with Input and transform
        match self.instrument_map.find(&subscription_id) {
            Ok(instrument) => {
                MarketIter::<InstrumentKey, <Kind as SubscriptionKind>::Event>::from((
                    ExchangeId::Other, // Valor fixo temporário, ajuste conforme necessário
                    instrument.clone(),
                    input,
                ))
                .0
            }
            Err(unidentifiable) => vec![Err(DataError::from(unidentifiable))],
        }
    }
}

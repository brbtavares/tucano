// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use super::{StreamBuilder, Streams};
use crate::{
    error::DataError,
    streams::{consumer::MarketStreamResult, reconnect::stream::ReconnectingStream},
    subscription::SubscriptionKind,
};
use futures_util::StreamExt;
use std::{collections::HashMap, fmt::Debug, future::Future, pin::Pin};
use tucano_integration::channel::Channel;
use tucano_instrument::exchange::ExchangeId;

/// Communicative type alias representing the [`Future`] result of a [`StreamBuilder::init`] call
/// generated whilst executing [`MultiStreamBuilder::add`].
pub type BuilderInitFuture = Pin<Box<dyn Future<Output = Result<(), DataError>>>>;

/// Builder to configure and initialise a common [`Streams<Output>`](Streams) instance from
/// multiple [`StreamBuilder<SubscriptionKind>`](StreamBuilder)s.
#[derive(Default)]
pub struct MultiStreamBuilder<Output> {
    pub channels: HashMap<ExchangeId, Channel<Output>>,
    pub futures: Vec<BuilderInitFuture>,
}

impl<Output> Debug for MultiStreamBuilder<Output>
where
    Output: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiStreamBuilder<Output>")
            .field("channels", &self.channels)
            .field("num_futures", &self.futures.len())
            .finish()
    }
}

impl<Output> MultiStreamBuilder<Output> {
    /// Construct a new [`Self`].
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            futures: Vec::new(),
        }
    }

    /// Add a [`StreamBuilder<SubscriptionKind>`](StreamBuilder) to the [`MultiStreamBuilder`]. Creates a
    /// [`Future`] that calls [`StreamBuilder::init`] and maps the [`SubscriptionKind::Event`](SubscriptionKind)
    /// into a common `Output`.
    ///
    /// Note that the created [`Future`] is not awaited until the [`MultiStreamBuilder::init`]
    /// method is invoked.
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, _builder: StreamBuilder) -> Self {
        // TODO: Implementação concreta de add deve ser feita na crate exchanges
        unimplemented!("A implementação concreta de MultiStreamBuilder::add deve ser feita na crate exchanges.");
    }

    /// Initialise each [`StreamBuilder<SubscriptionKind>`](StreamBuilder) that was added to the
    /// [`MultiStreamBuilder`] and map all [`Streams<SubscriptionKind::Event>`](Streams) into a common
    /// [`Streams<Output>`](Streams).
    pub async fn init(self) -> Result<Streams<Output>, DataError> {
        // TODO: Implementação concreta de init deve ser feita na crate exchanges
        unimplemented!("A implementação concreta de MultiStreamBuilder::init deve ser feita na crate exchanges.");
    }
}

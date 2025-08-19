// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use self::builder::{multi::MultiStreamBuilder, StreamBuilder};
use crate::subscription::SubscriptionKind;
use fnv::FnvHashMap;
use futures::Stream;
use tucano_integration::channel::UnboundedRx;
use tucano_instrument::exchange::ExchangeId;

/// Defines the [`StreamBuilder`] and [`MultiStreamBuilder`] APIs for ergonomically initialising
/// [`MarketStream`](super::MarketStream) [`Streams`].
pub mod builder;

/// Central consumer loop functionality used by the [`StreamBuilder`] to
/// drive a re-connecting [`MarketStream`](super::MarketStream).
pub mod consumer;

/// Defines a [`ReconnectingStream`](reconnect::stream::ReconnectingStream) and associated logic
/// for generating an auto reconnecting `Stream`.
pub mod reconnect;

/// Ergonomic collection of exchange market event receivers.
#[derive(Debug)]
pub struct Streams<T> {
    pub streams: FnvHashMap<ExchangeId, UnboundedRx<T>>,
}

impl<T> Streams<T> {
    /// Construct a [`StreamBuilder`] for configuring new market event [`Streams`].
    pub fn builder() -> StreamBuilder {
        StreamBuilder::default()
    }

    /// Construct a [`MultiStreamBuilder`] for configuring new
    /// [`MarketEvent<T>`](crate::event::MarketEvent) [`Streams`].
    pub fn builder_multi() -> MultiStreamBuilder<T> {
        MultiStreamBuilder::<T>::new()
    }

    /// Remove an exchange market event [`Stream`] from the [`Streams`] `HashMap`.
    pub fn select(&mut self, exchange: ExchangeId) -> Option<impl Stream<Item = T> + '_> {
        self.streams.remove(&exchange).map(UnboundedRx::into_stream)
    }

    /// Select and merge every exchange `Stream` using
    /// [`select_all`](futures_util::stream::select_all::select_all).
    pub fn select_all(self) -> impl Stream<Item = T> {
        let all = self.streams.into_values().map(UnboundedRx::into_stream);
        futures_util::stream::select_all::select_all(all)
    }
}

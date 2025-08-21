
use crate::{
    books::{
        map::{OrderBookMap, OrderBookMapMulti},
        OrderBook,
    },
    error::DataError,
    // ...existing code...
    instrument::InstrumentData,
    streams::{consumer::MarketStreamEvent, reconnect::stream::ReconnectingStream, Streams},
    subscription::{
        book::{OrderBookEvent, OrderBooksL2},
        Subscription,
    },
    Identifier,
};
use fnv::FnvHashMap;
use futures::Stream;
use futures_util::StreamExt;
use parking_lot::RwLock;
use std::{
    fmt::{Debug, Display},
    hash::Hash,
    sync::Arc,
};
use tracing::warn;

/// Maintains a set of local L2 [`OrderBook`]s by applying streamed [`OrderBookEvent`]s to the
/// associated [`OrderBook`] in the [`OrderBookMap`].
#[derive(Debug)]
pub struct OrderBookL2Manager<St, BookMap> {
    pub stream: St,
    pub books: BookMap,
}

impl<St, BookMap> OrderBookL2Manager<St, BookMap>
where
    St: Stream<Item = MarketStreamEvent<BookMap::Key, OrderBookEvent>> + Unpin,
    BookMap: OrderBookMap,
    BookMap::Key: Debug,
{
    /// Manage local L2 [`OrderBook`]s.
    pub async fn run(mut self) {
        while let Some(stream_event) = self.stream.next().await {
            // Extract MarketEvent<InstrumentKey, OrderBookEvent>
            let event = match stream_event {
                MarketStreamEvent::Reconnecting(exchange) => {
                    warn!(%exchange, "OrderBook manager input stream disconnected");
                    continue;
                }
                MarketStreamEvent::Item(event) => event,
            };

            // Find OrderBook associated with the MarketEvent InstrumentKey
            let Some(book) = self.books.find(&event.instrument) else {
                warn!(
                    instrument = ?event.instrument,
                    "consumed MarketStreamEvent<_, OrderBookEvent> for non-configured instrument"
                );
                continue;
            };

            let mut book_lock = book.write();
            book_lock.update(&event.kind);
        }
    }
}

/// Initialise a [`OrderBookL2Manager`] using the provided batches of [`OrderBooksL2`]
/// [`Subscription`]s.
///
/// See `examples/order_books_l2_manager` for how to use this initialisation paradigm.
pub async fn init_multi_order_book_l2_manager() {
    unimplemented!("The concrete implementation of init_multi_order_book_l2_manager must be done in the exchanges crate.");
}

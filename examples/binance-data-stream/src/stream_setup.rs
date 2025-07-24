use data::{
    streams::{Streams, consumer::MarketStreamResult, reconnect::Event, reconnect::stream::ReconnectingStream},
    subscription::{book::{OrderBooksL2, OrderBookEvent}},
    exchange::binance::futures::BinanceFuturesUsd,
};
use markets::instrument::market_data::{kind::MarketDataInstrumentKind};
use markets::exchange::ExchangeId;
use integration::channel::{UnboundedTx, Tx};
use crate::types::{OrderBookData, orderbook::OrderedFloat};
use std::collections::BTreeMap;
use futures::StreamExt;
use tracing::{info, warn, error};

pub async fn start_data_streams(orderbook_tx: UnboundedTx<OrderBookData>) -> anyhow::Result<()> {
    info!("🚀 Starting Toucan framework streams for BTCUSDT perpetual futures");

    let mut streams  = Streams::<OrderBooksL2>::builder()
            .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2)])
        .init()
        .await
        .unwrap();

    let mut l2_stream = streams
        .select(ExchangeId::BinanceFuturesUsd)
        .unwrap()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    //l2_stream = l2_stream.reconnect();

    while let Some(market_event) = l2_stream.next().await {
        match market_event {
            Event::Item(market_event) => {
                if let OrderBookEvent::Snapshot(order_book) | OrderBookEvent::Update(order_book) = &market_event.kind {
                    let mut bids = BTreeMap::new();
                    for level in order_book.bids().levels().iter().take(10) {
                        let price = level.price.to_string().parse::<f64>().unwrap_or(0.0);
                        let amount = level.amount.to_string().parse::<f64>().unwrap_or(0.0);
                        bids.insert(OrderedFloat::from(price), amount);
                    }
                    let mut asks = BTreeMap::new();
                    for level in order_book.asks().levels().iter().take(10) {
                        let price = level.price.to_string().parse::<f64>().unwrap_or(0.0);
                        let amount = level.amount.to_string().parse::<f64>().unwrap_or(0.0);
                        asks.insert(OrderedFloat::from(price), amount);
                    }
                    let order_book_data = OrderBookData {
                        symbol: "BTCUSDT".to_string(),
                        timestamp: market_event.time_exchange,
                        bids,
                        asks,
                        last_update_id: order_book.sequence(),
                    };
                    if let Err(e) = orderbook_tx.send(order_book_data) {
                        error!("Failed to send order book data to TUI: {e}");
                        break;
                    }
                }
            }
            Event::Item(e) => {
                error!("Stream error");
                continue;
            }
            Event::Reconnecting(exchange_id) => {
                info!("🔄 Stream reconnecting for exchange: {:?}", exchange_id);
                continue;
            }
            
        }
    }

    Ok(())
}
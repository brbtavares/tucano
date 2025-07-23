/// Real integration implementation with Toucan framework - READY TO USE
/// 
/// This file contains the real implementation of Toucan framework streams.
/// 

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, error};
use data::{
    streams::{Streams, consumer::MarketStreamResult, reconnect::Event},
    event::DataKind,
    subscription::{trade::PublicTrades, book::{OrderBooksL2, OrderBookEvent}},
    exchange::binance::futures::BinanceFuturesUsd,
};
use markets::{
    instrument::market_data::{MarketDataInstrument, kind::MarketDataInstrumentKind},
    Side,
};
use futures::StreamExt;
use crate::types::{OrderBookData, TradeData, orderbook::OrderedFloat};
use std::collections::BTreeMap;

#[allow(dead_code)]
pub async fn start_real_data_streams(
    orderbook_tx: mpsc::UnboundedSender<OrderBookData>,
    trades_tx: mpsc::UnboundedSender<TradeData>,
) -> Result<()> {
    info!("🚀 Starting real Toucan framework streams for BTCUSDT perpetual futures");
    
    // Create multi-type streams (trades + order book L2) using builder_multi
    let streams: Streams<MarketStreamResult<MarketDataInstrument, DataKind>> = Streams::builder_multi()
        // Add PublicTrades Stream
        .add(Streams::<PublicTrades>::builder()
            .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades)])
        )
        // Add OrderBooksL2 Stream  
        .add(Streams::<OrderBooksL2>::builder()
            .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2)])
        )
        .init()
        .await?;
    
    // Use select_all to process all streams
    let mut joined_stream = streams.select_all();
    
    // Process events using while loop like in the examples  
    while let Some(event_result) = joined_stream.next().await {
        match event_result {
            Event::Item(Ok(event)) => {
                // Parse based on the event kind
                match &event.kind {
                    DataKind::Trade(trade) => {
                        
                        // Convert to TUI format
                        let trade_data = TradeData {
                            symbol: "BTCUSDT".to_string(),
                            trade_id: trade.id.parse().unwrap_or(0),
                            price: trade.price,
                            quantity: trade.amount,
                            timestamp: event.time_exchange,
                            is_buyer_maker: matches!(trade.side, Side::Sell), // sell trade means buyer was maker
                        };
                        
                        // Send to TUI
                        if let Err(e) = trades_tx.send(trade_data) {
                            error!("Failed to send trade data to TUI: {e}");
                            break;
                        }
                    }
                    DataKind::OrderBook(order_book_event) => {
                        match order_book_event {
                            OrderBookEvent::Snapshot(order_book) | OrderBookEvent::Update(order_book) => {                                
                                // Convert to TUI format
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
                                    timestamp: event.time_exchange,
                                    bids,
                                    asks,
                                    last_update_id: order_book.sequence(),
                                };
                                
                                // Send to TUI
                                if let Err(e) = orderbook_tx.send(order_book_data) {
                                    error!("Failed to send order book data to TUI: {e}");
                                    break;
                                }
                            }
                        }
                    }
                    _ => {
                        // Other event types we don't handle yet
                        info!("Received other event type: {:?}", event.kind);
                    }
                }
            }
            Event::Item(Err(e)) => {
                error!("Stream error: {e}");
                // Continue processing instead of breaking on errors
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

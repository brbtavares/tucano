/*!
 * Binance BTC Real-time Trading Statistics Example
 * 
 * This example demonstrates a production-ready approach to collecting and analyzing
 * real-time Bitcoin trading data from Binance using Toucan's high-level Data module.
 * 
 * Key Features:
 * - Real-time BTC/USDT trade stream processing
 * - Comprehensive trading statistics calculation
 * - Automatic reconnection handling
 * - Structured logging with tracing
 * - Separate buy/sell volume tracking
 * - Price range monitoring (min/max)
 * - Configurable time-based execution (30 seconds)
 * 
 * Technical Components Used:
 * - Toucan Data module for market data streams
 * - BinanceSpot exchange integration
 * - PublicTrades subscription
 * - ReconnectingStream for fault tolerance
 * - Structured logging with custom statistics
 * 
 * Use Case:
 * This example shows how to build real-world trading analytics systems
 * that can monitor market activity, generate reports, and maintain statistics
 * for trading decision making or market analysis.
 * 
 * Output:
 * - Live trade count updates every 10 trades
 * - Final comprehensive report with volume and price statistics
 * - Buy vs Sell breakdown
 * 
 * Setup:
 * No API keys required - uses public market data only.
 * Run: cargo run --bin binance_btc_realtime_statistics
 */

use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use chrono::Utc;
use futures::StreamExt;

// Toucan ecosystem
use data::{
    exchange::binance::spot::BinanceSpot,
    streams::{Streams, reconnect::{stream::ReconnectingStream, Event}},
    subscription::trade::{PublicTrades, PublicTrade},
    event::MarketEvent,
};
use markets::{
    instrument::market_data::{MarketDataInstrument, kind::MarketDataInstrumentKind},
    Side,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Starting Binance BTCUSDT data stream");

    info!("📊 Setting up stream for BTC/USDT");

    // Configure data stream using the correct pattern
    let streams = Streams::<PublicTrades>::builder()
        .subscribe([
            (BinanceSpot::default(), "btc", "usdt", MarketDataInstrumentKind::Spot, PublicTrades),
        ])
        .init()
        .await?;

    // Create unified stream
    let mut stream = streams
        .select_all()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    info!("📡 Connected to Binance data stream");
    info!("⏰ Running for 30 seconds...");

    // Simple statistics
    let mut stats = TradingStats::new();
    let start_time = Utc::now();

    // Process events for 30 seconds
    tokio::select! {
        _ = async {
            while let Some(event) = stream.next().await {
                match event {
                    Event::Item(market_event) => {
                        stats.process_event(&market_event);
                        
                        // Log every 10 trades
                        if stats.trade_count % 10 == 0 {
                            info!("📈 Trades: {} | Last price: ${:.2} | Volume: {:.4} BTC", 
                                  stats.trade_count, 
                                  stats.last_price.unwrap_or(0.0),
                                  stats.volume_btc);
                        }
                    }
                    Event::Reconnecting(exchange_id) => {
                        warn!("🔄 Reconnecting exchange: {:?}", exchange_id);
                    }
                }
            }
        } => {}
        _ = sleep(Duration::from_secs(30)) => {
            info!("⏱️ 30-second time limit reached");
        }
    }

    // Final report
    let end_time = Utc::now();
    let duration = end_time.signed_duration_since(start_time);
    
    info!("📊 === FINAL REPORT ===");
    info!("⏰ Duration: {} seconds", duration.num_seconds());
    info!("📈 Total trades: {}", stats.trade_count);
    info!("💰 Minimum price: ${:.2}", stats.min_price.unwrap_or(0.0));
    info!("💰 Maximum price: ${:.2}", stats.max_price.unwrap_or(0.0));
    info!("💰 Last price: ${:.2}", stats.last_price.unwrap_or(0.0));
    info!("📊 Total volume: {:.4} BTC", stats.volume_btc);
    info!("📊 Buy volume: {:.4} BTC", stats.buy_volume_btc);
    info!("📊 Sell volume: {:.4} BTC", stats.sell_volume_btc);
    info!("📊 Buy trades: {}", stats.buy_trades);
    info!("📊 Sell trades: {}", stats.sell_trades);
    info!("🏁 Streaming completed successfully!");

    Ok(())
}

/// Structure to maintain trading statistics
#[derive(Debug, Clone)]
struct TradingStats {
    trade_count: u64,
    min_price: Option<f64>,
    max_price: Option<f64>,
    last_price: Option<f64>,
    volume_btc: f64,
    buy_volume_btc: f64,
    sell_volume_btc: f64,
    buy_trades: u64,
    sell_trades: u64,
}

impl TradingStats {
    fn new() -> Self {
        Self {
            trade_count: 0,
            min_price: None,
            max_price: None,
            last_price: None,
            volume_btc: 0.0,
            buy_volume_btc: 0.0,
            sell_volume_btc: 0.0,
            buy_trades: 0,
            sell_trades: 0,
        }
    }

    fn process_event(&mut self, event: &MarketEvent<MarketDataInstrument, PublicTrade>) {
        let trade = &event.kind;
        self.trade_count += 1;
        self.last_price = Some(trade.price);
        self.volume_btc += trade.amount;

        // Update min/max prices
        match self.min_price {
            None => self.min_price = Some(trade.price),
            Some(min) if trade.price < min => self.min_price = Some(trade.price),
            _ => {}
        }

        match self.max_price {
            None => self.max_price = Some(trade.price),
            Some(max) if trade.price > max => self.max_price = Some(trade.price),
            _ => {}
        }

        // Separate by side
        match trade.side {
            Side::Buy => {
                self.buy_trades += 1;
                self.buy_volume_btc += trade.amount;
            }
            Side::Sell => {
                self.sell_trades += 1;
                self.sell_volume_btc += trade.amount;
            }
        }
    }
}

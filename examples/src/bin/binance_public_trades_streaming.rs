/*!
 * Binance Public Trades Streaming Example
 * 
 * This example demonstrates how to stream real-time public trade data from Binance
 * futures using Toucan's high-level streaming capabilities for multiple trading pairs.
 * 
 * Key Features:
 * - Multiple cryptocurrency pair streaming (BTC, ETH, SOL vs USDT)
 * - High-volume data handling with separate WebSocket connections
 * - Automatic reconnection and error recovery
 * - Real-time trade event processing
 * - Structured logging for monitoring
 * 
 * Technical Components:
 * - BinanceFuturesUsd exchange integration
 * - PublicTrades subscription type
 * - ReconnectingStream for fault tolerance
 * - Concurrent stream processing
 * 
 * Use Case:
 * Perfect for building market monitoring systems, price alerts, arbitrage
 * detection, or general market data analysis across multiple trading pairs.
 * 
 * Setup:
 * No API keys required - uses public market data only.
 * Run: cargo run --bin binance_public_trades_streaming
 */

use data::{
    exchange::binance::futures::BinanceFuturesUsd,
    streams::{Streams, reconnect::stream::ReconnectingStream},
    subscription::trade::PublicTrades,
};
use markets::instrument::market_data::kind::MarketDataInstrumentKind;
use futures_util::StreamExt;
use tracing::{info, warn};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Initialise PublicTrades Streams for BinanceFuturesUsd only
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let streams = Streams::<PublicTrades>::builder()

        // Separate WebSocket connection for BTC_USDT stream since it's very high volume
        .subscribe([
            (BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades),
        ])

        // Separate WebSocket connection for ETH_USDT stream since it's very high volume
        .subscribe([
            (BinanceFuturesUsd::default(), "eth", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades),
        ])

        // Lower volume Instruments can share a WebSocket connection
        .subscribe([
            (BinanceFuturesUsd::default(), "xrp", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades),
            (BinanceFuturesUsd::default(), "sol", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades),
            (BinanceFuturesUsd::default(), "avax", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades),
            (BinanceFuturesUsd::default(), "ltc", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades),
        ])
        .init()
        .await
        .unwrap();

    // Select and merge every exchange Stream using futures_util::stream::select_all
    // Note: use `Streams.select(ExchangeId)` to interact with individual exchange streams!
    let mut joined_stream = streams
        .select_all()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    while let Some(event) = joined_stream.next().await {
        info!("{event:?}");
    }
}

// Initialise an INFO `Subscriber` for `Tracing` Json logs and install it as the global default.
fn init_logging() {
    tracing_subscriber::fmt()
        // Filter messages based on the INFO
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // Disable colours on release builds
        .with_ansi(cfg!(debug_assertions))
        // Enable Json formatting
        .json()
        // Install this Tracing subscriber as global default
        .init()
}

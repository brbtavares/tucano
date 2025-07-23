/// Level 2 order book streaming example
/// Demonstrates full order book depth streaming from Binance

use data::{
    exchange::binance::futures::BinanceFuturesUsd,
    streams::{Streams, reconnect::stream::ReconnectingStream},
    subscription::book::OrderBooksL2,
};
use markets::{
    exchange::ExchangeId, instrument::market_data::kind::MarketDataInstrumentKind,
};
use futures_util::StreamExt;
use tracing::{info, warn};

#[rustfmt::skip]
#[tokio::main]
async fn main() {
    // Initialise INFO Tracing log subscriber
    init_logging();

    // Initialise OrderBooksL2 Streams for BinanceSpot only
    // '--> each call to StreamBuilder::subscribe() creates a separate WebSocket connection
    let mut streams = Streams::<OrderBooksL2>::builder()

        // Separate WebSocket connection for BTC_USDT stream since it's very high volume
        .subscribe([
            (BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2),
        ])

        // Separate WebSocket connection for ETH_USDT stream since it's very high volume
        //.subscribe([
        //    (BinanceFuturesUsd::default(), "eth", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2),
        //])

        // Lower volume Instruments can share a WebSocket connection
        //.subscribe([
        //    (BinanceFuturesUsd::default(), "xrp", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2),
        //    (BinanceFuturesUsd::default(), "sol", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2),
        //    (BinanceFuturesUsd::default(), "avax", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2),
        //    (BinanceFuturesUsd::default(), "ltc", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL2),
        //])
        .init()
        .await
        .unwrap();

    // Select the ExchangeId::BinanceFuturesUsd stream
    // Note: use `Streams.select(ExchangeId)` to interact with individual exchange streams!
    let mut l2_stream = streams
        .select(ExchangeId::BinanceFuturesUsd)
        .unwrap()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    while let Some(event) = l2_stream.next().await {
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

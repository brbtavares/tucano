use serde::{Deserialize, Serialize};
use markets::instrument::market_data::{kind::MarketDataInstrumentKind, MarketDataInstrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub binance: BinanceConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    pub instrument: MarketDataInstrument,
    pub reconnect_interval_secs: u64,
    pub max_reconnect_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub orderbook_depth: usize,
    pub trades_history_size: usize,
    pub refresh_rate_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            binance: BinanceConfig::default(),
            display: DisplayConfig::default(),
        }
    }
}

impl Default for BinanceConfig {
    fn default() -> Self {
        Self {
            instrument: MarketDataInstrument::new("btc", "usdt", MarketDataInstrumentKind::Perpetual),
            reconnect_interval_secs: 5,
            max_reconnect_attempts: 10,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            orderbook_depth: 20,
            trades_history_size: 1000,
            refresh_rate_ms: 100,
        }
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub binance: BinanceConfig,
    pub display: DisplayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceConfig {
    pub base_url: String,
    pub ws_url: String,
    pub symbol: String,
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
            base_url: "https://fapi.binance.com".to_string(),
            ws_url: "wss://fstream.binance.com".to_string(),
            symbol: "BTCUSDT".to_string(),
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

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

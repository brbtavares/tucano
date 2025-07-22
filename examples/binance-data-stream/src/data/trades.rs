use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub symbol: String,
    pub trade_id: u64,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: DateTime<Utc>,
    pub is_buyer_maker: bool, // true if buyer is market maker (sell trade)
}

impl TradeData {
    pub fn new(
        symbol: String,
        trade_id: u64,
        price: f64,
        quantity: f64,
        is_buyer_maker: bool,
    ) -> Self {
        Self {
            symbol,
            trade_id,
            price,
            quantity,
            timestamp: Utc::now(),
            is_buyer_maker,
        }
    }

    pub fn mock_data() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static TRADE_ID: AtomicU64 = AtomicU64::new(1);
        
        let trade_id = TRADE_ID.fetch_add(1, Ordering::SeqCst);
        let base_price = 45005.0;
        let price_variation = (rand::random::<f64>() - 0.5) * 100.0; // ±50 price variation
        let price = base_price + price_variation;
        let quantity = 0.001 + rand::random::<f64>() * 0.999; // 0.001 to 1.0 BTC
        let is_buyer_maker = rand::random::<f64>() > 0.5;

        Self::new("BTCUSDT".to_string(), trade_id, price, quantity, is_buyer_maker)
    }

    pub fn side_str(&self) -> &'static str {
        if self.is_buyer_maker {
            "SELL" // buyer is maker means it was a sell order filled
        } else {
            "BUY" // buyer is taker means it was a buy order
        }
    }

    pub fn notional_value(&self) -> f64 {
        self.price * self.quantity
    }
}

// For storing recent trades
#[derive(Debug, Clone)]
pub struct TradesHistory {
    pub trades: Vec<TradeData>,
    pub max_size: usize,
}

impl TradesHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            trades: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add_trade(&mut self, trade: TradeData) {
        self.trades.push(trade);
        if self.trades.len() > self.max_size {
            self.trades.remove(0);
        }
    }

    pub fn get_recent_trades(&self, count: usize) -> &[TradeData] {
        let start = if self.trades.len() > count {
            self.trades.len() - count
        } else {
            0
        };
        &self.trades[start..]
    }

    pub fn get_volume_in_period(&self, seconds: i64) -> f64 {
        let cutoff = Utc::now() - chrono::Duration::seconds(seconds);
        self.trades
            .iter()
            .filter(|trade| trade.timestamp > cutoff)
            .map(|trade| trade.quantity)
            .sum()
    }

    pub fn get_avg_price_in_period(&self, seconds: i64) -> Option<f64> {
        let cutoff = Utc::now() - chrono::Duration::seconds(seconds);
        let recent_trades: Vec<_> = self.trades
            .iter()
            .filter(|trade| trade.timestamp > cutoff)
            .collect();

        if recent_trades.is_empty() {
            return None;
        }

        let total_notional: f64 = recent_trades
            .iter()
            .map(|trade| trade.notional_value())
            .sum();
        let total_volume: f64 = recent_trades
            .iter()
            .map(|trade| trade.quantity)
            .sum();

        if total_volume > 0.0 {
            Some(total_notional / total_volume)
        } else {
            None
        }
    }
}

// Simple random number generator for mock data
mod rand {
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEED: AtomicU64 = AtomicU64::new(1);

    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        let seed = SEED.fetch_add(1, Ordering::SeqCst);
        let a = 1664525_u64;
        let c = 1013904223_u64;
        let m = 2_u64.pow(32);
        
        let next = (a.wrapping_mul(seed).wrapping_add(c)) % m;
        SEED.store(next, Ordering::SeqCst);
        
        T::from(next as f64 / m as f64)
    }
}

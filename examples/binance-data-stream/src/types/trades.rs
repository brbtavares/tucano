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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookData {
    pub symbol: String,
    pub timestamp: DateTime<Utc>,
    pub bids: BTreeMap<OrderedFloat, f64>, // price -> quantity
    pub asks: BTreeMap<OrderedFloat, f64>, // price -> quantity
    pub last_update_id: u64,
}

// Wrapper for f64 to make it Ord for BTreeMap
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OrderedFloat(pub f64);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl From<f64> for OrderedFloat {
    fn from(f: f64) -> Self {
        OrderedFloat(f)
    }
}

impl std::fmt::Display for OrderedFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl OrderBookData {
    #[allow(dead_code)]
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            timestamp: Utc::now(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
        }
    }

    pub fn get_best_bid(&self) -> Option<(OrderedFloat, f64)> {
        self.bids.iter().rev().next().map(|(&price, &qty)| (price, qty))
    }

    pub fn get_best_ask(&self) -> Option<(OrderedFloat, f64)> {
        self.asks.iter().next().map(|(&price, &qty)| (price, qty))
    }

    pub fn get_spread(&self) -> Option<f64> {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some((bid_price, _)), Some((ask_price, _))) => {
                Some(ask_price.0 - bid_price.0)
            }
            _ => None,
        }
    }

    pub fn get_mid_price(&self) -> Option<f64> {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some((bid_price, _)), Some((ask_price, _))) => {
                Some((bid_price.0 + ask_price.0) / 2.0)
            }
            _ => None,
        }
    }
}

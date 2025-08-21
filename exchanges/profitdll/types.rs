
//! B3-specific types and structures
//!
//! This module contains all B3-specific data types that implement
//! the markets abstractions, making them compatible with the framework
//! while maintaining Brazilian market terminology.

use super::profitdll_types::OrderSide;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::fmt::{self, Display};
use toucan_instrument::{Asset, AssetType};

/// B3 Exchange identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3ExchangeId;

impl Display for B3ExchangeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B3")
    }
}

/// B3 Instrument identifier (ticker symbol)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Instrument {
    /// Ticker symbol (e.g., "PETR4", "VALE3", "ITUB4")
    pub symbol: SmolStr,
    /// Market segment (e.g., "BOVESPA", "BMF")
    pub market: SmolStr,
}

impl B3Instrument {
    pub fn new(symbol: impl Into<SmolStr>, market: impl Into<SmolStr>) -> Self {
        Self {
            symbol: symbol.into(),
            market: market.into(),
        }
    }

    pub fn bovespa(symbol: impl Into<SmolStr>) -> Self {
        Self::new(symbol, "BOVESPA")
    }

    pub fn bmf(symbol: impl Into<SmolStr>) -> Self {
        Self::new(symbol, "BMF")
    }
}

impl Display for B3Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.symbol, self.market)
    }
}

/// B3 Asset types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3Asset {
    /// Brazilian Real
    BRL,
    /// Stocks (ações)
    Stock(SmolStr),
    /// Funds (fundos)
    Fund(SmolStr),
    /// Futures contracts
    Future(SmolStr),
    /// Options
    Option(SmolStr),
    /// Other asset types
    Other(SmolStr),
}

impl Display for B3Asset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            B3Asset::BRL => write!(f, "BRL"),
            B3Asset::Stock(s) => write!(f, "STOCK:{s}"),
            B3Asset::Fund(s) => write!(f, "FUND:{s}"),
            B3Asset::Future(s) => write!(f, "FUTURE:{s}"),
            B3Asset::Option(s) => write!(f, "OPTION:{s}"),
            B3Asset::Other(s) => write!(f, "OTHER:{s}"),
        }
    }
}

/// Order side for B3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3Side {
    Buy,
    Sell,
}

impl Display for B3Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            B3Side::Buy => write!(f, "BUY"),
            B3Side::Sell => write!(f, "SELL"),
        }
    }
}

/// B3 Order types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3OrderType {
    Market,
    Limit,
    StopLoss,
    StopLimit,
}

/// B3 Order status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3OrderStatus {
    Pending,
    PartiallyFilled { filled_quantity: Decimal },
    Filled,
    Cancelled,
    Rejected { reason: SmolStr },
}

/// B3 Order structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3Order {
    pub id: SmolStr,
    pub instrument: B3Instrument,
    pub side: B3Side,
    pub order_type: B3OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub status: B3OrderStatus,
    pub timestamp: DateTime<Utc>,
}

/// B3 Trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3Trade {
    pub id: SmolStr,
    pub instrument: B3Instrument,
    pub side: B3Side,
    pub quantity: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub buyer_agent: Option<SmolStr>,
    pub seller_agent: Option<SmolStr>,
}

/// B3 Balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3Balance {
    pub asset: B3Asset,
    pub available: Decimal,
    pub locked: Decimal,
}

impl B3Balance {
    pub fn total(&self) -> Decimal {
        self.available + self.locked
    }
}

/// B3 Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3Account {
    pub account_id: SmolStr,
    pub account_holder: SmolStr,
    pub broker_name: SmolStr,
    pub broker_id: i32,
    pub balances: Vec<B3Balance>,
}

/// B3 Market data tick
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3MarketTick {
    pub instrument: B3Instrument,
    pub price: Decimal,
    pub volume: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// B3 Order book level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3BookLevel {
    pub price: Decimal,
    pub quantity: Decimal,
    pub position: i32,
}

/// B3 Order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B3OrderBook {
    pub instrument: B3Instrument,
    pub bids: Vec<B3BookLevel>,
    pub asks: Vec<B3BookLevel>,
    pub timestamp: DateTime<Utc>,
}

/// B3 Order book side
#[derive(Debug, Clone)]
pub enum B3BookSide {
    Bid,
    Ask,
}

/// Implement markets::Asset trait for B3Asset
impl Asset for B3Asset {
    fn symbol(&self) -> &str {
        match self {
            B3Asset::BRL => "BRL",
            B3Asset::Stock(s) => s.as_str(),
            B3Asset::Fund(s) => s.as_str(),
            B3Asset::Future(s) => s.as_str(),
            B3Asset::Option(s) => s.as_str(),
            B3Asset::Other(s) => s.as_str(),
        }
    }

    fn asset_type(&self) -> AssetType {
        match self {
            B3Asset::BRL => AssetType::Currency,
            B3Asset::Stock(_) => AssetType::Stock,
            B3Asset::Fund(_) => AssetType::Fund,
            B3Asset::Future(_) => AssetType::Future,
            B3Asset::Option(_) => AssetType::Option,
            B3Asset::Other(_) => AssetType::Other,
        }
    }
}

/// Conversion utilities from ProfitDLL types
impl From<OrderSide> for B3Side {
    fn from(side: OrderSide) -> Self {
        match side {
            OrderSide::Buy => B3Side::Buy,
            OrderSide::Sell => B3Side::Sell,
        }
    }
}

impl From<B3Side> for OrderSide {
    fn from(side: B3Side) -> Self {
        match side {
            B3Side::Buy => OrderSide::Buy,
            B3Side::Sell => OrderSide::Sell,
        }
    }
}

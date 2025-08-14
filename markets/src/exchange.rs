//! Exchange abstractions and identifiers for the Toucan trading framework
//!
//! This module provides core exchange definitions and the ExchangeId enum
//! used throughout the system to identify different trading venues.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Identifies different exchanges/trading venues supported by Toucan
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum ExchangeId {
    /// Brazilian Stock Exchange (B3)
    B3,
    /// Mock exchange for testing
    Mock,
    /// Simulated exchange for backtesting
    Simulated,
    /// Binance cryptocurrency exchange
    Binance,
    /// Coinbase cryptocurrency exchange
    Coinbase,
    /// FTX cryptocurrency exchange (legacy)
    Ftx,
    /// OKX cryptocurrency exchange
    Okx,
    /// Bybit cryptocurrency exchange
    Bybit,
    /// BitMEX cryptocurrency exchange
    Bitmex,
    /// Kraken cryptocurrency exchange
    Kraken,
    /// Huobi cryptocurrency exchange
    Huobi,
    /// KuCoin cryptocurrency exchange
    Kucoin,
    /// Gate.io cryptocurrency exchange
    GateIo,
    /// Bitfinex cryptocurrency exchange
    Bitfinex,
}

impl ExchangeId {
    /// Returns true if this is a Brazilian exchange
    pub fn is_brazilian(&self) -> bool {
        matches!(self, ExchangeId::B3)
    }

    /// Returns true if this is a cryptocurrency exchange
    pub fn is_crypto(&self) -> bool {
        matches!(
            self,
            ExchangeId::Binance
                | ExchangeId::Coinbase
                | ExchangeId::Ftx
                | ExchangeId::Okx
                | ExchangeId::Bybit
                | ExchangeId::Bitmex
                | ExchangeId::Kraken
                | ExchangeId::Huobi
                | ExchangeId::Kucoin
                | ExchangeId::GateIo
                | ExchangeId::Bitfinex
        )
    }

    /// Returns true if this is a testing/simulation exchange
    pub fn is_test(&self) -> bool {
        matches!(self, ExchangeId::Mock | ExchangeId::Simulated)
    }

    /// Returns the canonical string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeId::B3 => "B3",
            ExchangeId::Mock => "MOCK",
            ExchangeId::Simulated => "SIMULATED",
            ExchangeId::Binance => "BINANCE",
            ExchangeId::Coinbase => "COINBASE",
            ExchangeId::Ftx => "FTX",
            ExchangeId::Okx => "OKX",
            ExchangeId::Bybit => "BYBIT",
            ExchangeId::Bitmex => "BITMEX",
            ExchangeId::Kraken => "KRAKEN",
            ExchangeId::Huobi => "HUOBI",
            ExchangeId::Kucoin => "KUCOIN",
            ExchangeId::GateIo => "GATEIO",
            ExchangeId::Bitfinex => "BITFINEX",
        }
    }

    // kept for backward compatibility (deprecated)
    #[deprecated(note = "Use std::str::FromStr implementation instead")]
    pub fn parse(s: &str) -> Option<Self> {
        <Self as FromStr>::from_str(s).ok()
    }
}

impl FromStr for ExchangeId {
    type Err = (); // simple error for now
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "B3" => Ok(ExchangeId::B3),
            "MOCK" => Ok(ExchangeId::Mock),
            "SIMULATED" => Ok(ExchangeId::Simulated),
            "BINANCE" => Ok(ExchangeId::Binance),
            "COINBASE" => Ok(ExchangeId::Coinbase),
            "FTX" => Ok(ExchangeId::Ftx),
            "OKX" => Ok(ExchangeId::Okx),
            "BYBIT" => Ok(ExchangeId::Bybit),
            "BITMEX" => Ok(ExchangeId::Bitmex),
            "KRAKEN" => Ok(ExchangeId::Kraken),
            "HUOBI" => Ok(ExchangeId::Huobi),
            "KUCOIN" => Ok(ExchangeId::Kucoin),
            "GATEIO" | "GATE.IO" => Ok(ExchangeId::GateIo),
            "BITFINEX" => Ok(ExchangeId::Bitfinex),
            _ => Err(()),
        }
    }
}

impl Display for ExchangeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for ExchangeId {
    fn from(s: &str) -> Self {
        s.parse().unwrap_or(ExchangeId::Mock)
    }
}

/// Core exchange trait providing metadata and capabilities
pub trait Exchange {
    type ExchangeId;

    /// Returns the unique identifier for this exchange
    fn id(&self) -> Self::ExchangeId;

    /// Returns the human-readable name of the exchange
    fn name(&self) -> &'static str;

    /// Returns true if the exchange supports real-time market data
    fn supports_market_data(&self) -> bool {
        true
    }

    /// Returns true if the exchange supports order execution
    fn supports_trading(&self) -> bool {
        true
    }

    /// Returns true if the exchange is currently operational
    fn is_operational(&self) -> bool {
        true
    }
}

/// Brazilian Stock Exchange (B3) implementation
#[derive(Debug, Clone)]
pub struct B3Exchange;

impl Exchange for B3Exchange {
    type ExchangeId = ExchangeId;

    fn id(&self) -> Self::ExchangeId {
        ExchangeId::B3
    }

    fn name(&self) -> &'static str {
        "Brasil Bolsa Balcão"
    }

    fn supports_market_data(&self) -> bool {
        true
    }

    fn supports_trading(&self) -> bool {
        true
    }

    fn is_operational(&self) -> bool {
        true
    }
}

/// Mock exchange for testing
#[derive(Debug, Clone)]
pub struct MockExchange;

impl Exchange for MockExchange {
    type ExchangeId = ExchangeId;

    fn id(&self) -> Self::ExchangeId {
        ExchangeId::Mock
    }

    fn name(&self) -> &'static str {
        "Mock Exchange"
    }

    fn supports_market_data(&self) -> bool {
        true
    }

    fn supports_trading(&self) -> bool {
        true
    }

    fn is_operational(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_id_classification() {
        assert!(ExchangeId::B3.is_brazilian());
        assert!(!ExchangeId::B3.is_crypto());
        assert!(!ExchangeId::B3.is_test());

        assert!(!ExchangeId::Binance.is_brazilian());
        assert!(ExchangeId::Binance.is_crypto());
        assert!(!ExchangeId::Binance.is_test());

        assert!(!ExchangeId::Mock.is_brazilian());
        assert!(!ExchangeId::Mock.is_crypto());
        assert!(ExchangeId::Mock.is_test());
    }

    #[test]
    fn test_exchange_id_string_conversion() {
        assert_eq!(ExchangeId::B3.as_str(), "B3");
        assert_eq!(ExchangeId::B3.to_string(), "B3");

    assert_eq!(ExchangeId::from_str("b3").unwrap(), ExchangeId::B3);
    assert_eq!(ExchangeId::from_str("BINANCE").unwrap(), ExchangeId::Binance);
    assert!(ExchangeId::from_str("invalid").is_err());
    }

    #[test]
    fn test_exchange_implementations() {
        let b3 = B3Exchange;
        assert_eq!(b3.id(), ExchangeId::B3);
        assert_eq!(b3.name(), "Brasil Bolsa Balcão");
        assert!(b3.supports_market_data());
        assert!(b3.supports_trading());

        let mock = MockExchange;
        assert_eq!(mock.id(), ExchangeId::Mock);
        assert_eq!(mock.name(), "Mock Exchange");
    }
}

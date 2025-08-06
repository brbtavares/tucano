//! B3 instrument definitions and utilities

use crate::instrument::InstrumentData;
use markets::{
    exchange::ExchangeId, 
    instrument::market_data::kind::MarketDataInstrumentKind
};
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// B3 instrument data structure
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Instrument {
    pub ticker: String,
    pub exchange: String,
    pub security_type: B3SecurityType,
    pub isin: Option<String>,
    pub description: Option<String>,
    pub lot_size: Option<i32>,
    pub tick_size: Option<Decimal>,
}

/// Types of securities available on B3
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3SecurityType {
    /// Common stocks (Ações)
    Stock,
    /// Exchange Traded Funds
    Etf,
    /// Real Estate Investment Trusts (FIIs)
    Reit,
    /// Brazilian Depositary Receipts
    Bdr,
    /// Options on stocks
    Option,
    /// Futures contracts
    Future,
    /// Forward contracts
    Forward,
    /// Government bonds
    Bond,
    /// Corporate debentures
    Debenture,
    /// Other instrument types
    Other,
}

impl From<&str> for B3SecurityType {
    fn from(security_type: &str) -> Self {
        match security_type.to_uppercase().as_str() {
            "STOCK" | "AÇÃO" => B3SecurityType::Stock,
            "ETF" => B3SecurityType::Etf,
            "REIT" | "FII" => B3SecurityType::Reit,
            "BDR" => B3SecurityType::Bdr,
            "OPTION" | "OPÇÃO" => B3SecurityType::Option,
            "FUTURE" | "FUTURO" => B3SecurityType::Future,
            "FORWARD" | "TERMO" => B3SecurityType::Forward,
            "BOND" | "TÍTULO" => B3SecurityType::Bond,
            "DEBENTURE" | "DEBÊNTURE" => B3SecurityType::Debenture,
            _ => B3SecurityType::Other,
        }
    }
}

impl From<B3SecurityType> for MarketDataInstrumentKind {
    fn from(b3_type: B3SecurityType) -> Self {
        match b3_type {
            B3SecurityType::Stock | B3SecurityType::Etf | B3SecurityType::Reit | B3SecurityType::Bdr => {
                MarketDataInstrumentKind::Spot
            }
            B3SecurityType::Option => {
                // For now, return Spot since we don't have option contract details
                MarketDataInstrumentKind::Spot
            }
            B3SecurityType::Future | B3SecurityType::Forward => {
                // For now, return Perpetual since we don't have expiry details
                MarketDataInstrumentKind::Perpetual
            }
            B3SecurityType::Bond | B3SecurityType::Debenture => MarketDataInstrumentKind::Spot,
            B3SecurityType::Other => MarketDataInstrumentKind::Spot,
        }
    }
}

impl B3Instrument {
    /// Create a new B3 instrument
    pub fn new(
        ticker: String,
        exchange: String,
        security_type: B3SecurityType,
    ) -> Self {
        Self {
            ticker,
            exchange,
            security_type,
            isin: None,
            description: None,
            lot_size: None,
            tick_size: None,
        }
    }

    /// Get the full symbol including exchange
    pub fn symbol(&self) -> String {
        format!("{}@{}", self.ticker, self.exchange)
    }

    /// Check if this is a stock instrument
    pub fn is_stock(&self) -> bool {
        matches!(self.security_type, B3SecurityType::Stock)
    }

    /// Check if this is a derivative instrument
    pub fn is_derivative(&self) -> bool {
        matches!(
            self.security_type,
            B3SecurityType::Option | B3SecurityType::Future | B3SecurityType::Forward
        )
    }

    /// Check if this is a fixed income instrument
    pub fn is_fixed_income(&self) -> bool {
        matches!(
            self.security_type,
            B3SecurityType::Bond | B3SecurityType::Debenture
        )
    }
}

impl InstrumentData for B3Instrument {
    type Key = String;
    
    fn key(&self) -> &Self::Key {
        &self.ticker
    }
    
    fn kind(&self) -> &MarketDataInstrumentKind {
        // For now, we'll store the kind in the instrument
        // This is a simplified implementation
        match self.security_type {
            B3SecurityType::Stock | B3SecurityType::Etf | B3SecurityType::Reit | B3SecurityType::Bdr => {
                &MarketDataInstrumentKind::Spot
            }
            _ => &MarketDataInstrumentKind::Spot
        }
    }
}

/// Utility functions for B3 instrument handling
pub mod utils {
    use super::*;

    /// Parse a ticker symbol to extract underlying asset for derivatives
    pub fn parse_underlying(ticker: &str) -> Option<String> {
        // For options: PETR4P250 -> PETR4
        // For futures: WINV24 -> WIN
        if ticker.len() > 5 {
            // Check if it's an option (ends with call/put indicator)
            if ticker.chars().nth_back(2).map_or(false, |c| c.is_alphabetic()) {
                return Some(ticker[..ticker.len()-3].to_string());
            }
            
            // Check if it's a future (month/year code)
            if ticker.chars().nth_back(1).map_or(false, |c| c.is_numeric()) &&
               ticker.chars().nth_back(2).map_or(false, |c| c.is_alphabetic()) {
                // Find where the underlying ends and month code begins
                let mut underlying_len = ticker.len() - 3;
                for (i, c) in ticker.char_indices().rev() {
                    if c.is_alphabetic() && i < ticker.len() - 3 {
                        underlying_len = i + 1;
                        break;
                    }
                }
                return Some(ticker[..underlying_len].to_string());
            }
        }
        
        None
    }

    /// Determine security type from ticker pattern
    pub fn infer_security_type(ticker: &str) -> B3SecurityType {
        if ticker.ends_with("11") {
            B3SecurityType::Reit
        } else if ticker.len() > 5 && ticker.chars().nth_back(0).map_or(false, |c| c.is_numeric()) {
            B3SecurityType::Option
        } else if ticker.len() <= 4 || (ticker.len() == 5 && ticker.chars().last() == Some('4')) {
            B3SecurityType::Stock
        } else {
            B3SecurityType::Other
        }
    }

    /// Validate B3 ticker format
    pub fn is_valid_ticker(ticker: &str) -> bool {
        if ticker.is_empty() || ticker.len() > 12 {
            return false;
        }

        // Must start with alphabetic characters
        let first_char = ticker.chars().next().unwrap_or(' ');
        if !first_char.is_alphabetic() {
            return false;
        }

        // Can contain only alphanumeric characters
        ticker.chars().all(|c| c.is_alphanumeric())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_type_inference() {
        assert_eq!(utils::infer_security_type("PETR4"), B3SecurityType::Stock);
        assert_eq!(utils::infer_security_type("BBDC11"), B3SecurityType::Reit);
        assert_eq!(utils::infer_security_type("PETR4P250"), B3SecurityType::Option);
    }

    #[test]
    fn test_underlying_parsing() {
        assert_eq!(utils::parse_underlying("PETR4P250"), Some("PETR4".to_string()));
        assert_eq!(utils::parse_underlying("WINV24"), Some("WIN".to_string()));
        assert_eq!(utils::parse_underlying("PETR4"), None);
    }

    #[test]
    fn test_ticker_validation() {
        assert!(utils::is_valid_ticker("PETR4"));
        assert!(utils::is_valid_ticker("BBDC11"));
        assert!(!utils::is_valid_ticker(""));
        assert!(!utils::is_valid_ticker("123PETR"));
        assert!(!utils::is_valid_ticker("PETR-4"));
    }
}

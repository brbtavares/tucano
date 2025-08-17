// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
//! B3 (Brazilian Stock Exchange) asset definitions
//!
//! Provides specialized asset types for different instruments traded on B3:
//! - Stocks (Ações)
//! - Options (Opções)
//! - Futures (Futuros)
//! - ETFs
//! - Real Estate Investment Trusts (FIIs)

use crate::{Asset, AssetType};
use serde::{Deserialize, Serialize};
use std::fmt;

/// B3-specific asset categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum B3AssetCategory {
    /// Stocks (Ações)
    Stock,
    /// Options (Opções)
    Option,
    /// Futures (Futuros)
    Future,
    /// ETFs (Exchange Traded Funds)
    ETF,
    /// Real Estate Investment Trusts (Fundos Imobiliários)
    REIT,
    /// Certificates of Deposit (CDBs)
    CDB,
    /// Debentures
    Debenture,
}

impl fmt::Display for B3AssetCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            B3AssetCategory::Stock => write!(f, "Stock"),
            B3AssetCategory::Option => write!(f, "Option"),
            B3AssetCategory::Future => write!(f, "Future"),
            B3AssetCategory::ETF => write!(f, "ETF"),
            B3AssetCategory::REIT => write!(f, "REIT"),
            B3AssetCategory::CDB => write!(f, "CDB"),
            B3AssetCategory::Debenture => write!(f, "Debenture"),
        }
    }
}

/// Brazilian stock asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Stock {
    pub symbol: String,
    pub company_name: String,
    pub isin: Option<String>,
    pub trading_lot: u32,
    pub sector: Option<String>,
    pub subsector: Option<String>,
    pub segment: Option<String>,
}

impl Asset for B3Stock {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Stock
    }
}

impl B3Stock {
    pub fn new(symbol: String, company_name: String) -> Self {
        Self {
            symbol,
            company_name,
            isin: None,
            trading_lot: 1,
            sector: None,
            subsector: None,
            segment: None,
        }
    }

    pub fn with_details(
        symbol: String,
        company_name: String,
        isin: String,
        trading_lot: u32,
        sector: String,
        subsector: String,
        segment: String,
    ) -> Self {
        Self {
            symbol,
            company_name,
            isin: Some(isin),
            trading_lot,
            sector: Some(sector),
            subsector: Some(subsector),
            segment: Some(segment),
        }
    }

    pub fn category(&self) -> B3AssetCategory {
        B3AssetCategory::Stock
    }
}

/// Brazilian option asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Option {
    pub symbol: String,
    pub underlying_symbol: String,
    pub option_type: OptionType,
    pub strike_price: rust_decimal::Decimal,
    pub expiry_date: chrono::NaiveDate,
    pub exercise_style: ExerciseStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExerciseStyle {
    European,
    American,
}

impl Asset for B3Option {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Option
    }
}

impl B3Option {
    pub fn new(
        symbol: String,
        underlying_symbol: String,
        option_type: OptionType,
        strike_price: rust_decimal::Decimal,
        expiry_date: chrono::NaiveDate,
    ) -> Self {
        Self {
            symbol,
            underlying_symbol,
            option_type,
            strike_price,
            expiry_date,
            exercise_style: ExerciseStyle::European, // Default for B3
        }
    }

    pub fn category(&self) -> B3AssetCategory {
        B3AssetCategory::Option
    }

    pub fn is_call(&self) -> bool {
        matches!(self.option_type, OptionType::Call)
    }

    pub fn is_put(&self) -> bool {
        matches!(self.option_type, OptionType::Put)
    }

    pub fn days_to_expiry(&self) -> i64 {
        let today = chrono::Utc::now().date_naive();
        (self.expiry_date - today).num_days()
    }

    pub fn is_expired(&self) -> bool {
        let today = chrono::Utc::now().date_naive();
        self.expiry_date <= today
    }
}

/// Brazilian futures asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Future {
    pub symbol: String,
    pub underlying: String,
    pub contract_month: chrono::NaiveDate,
    pub contract_size: rust_decimal::Decimal,
    pub tick_size: rust_decimal::Decimal,
    pub settlement_type: SettlementType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementType {
    Physical,
    Cash,
}

impl Asset for B3Future {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Future
    }
}

impl B3Future {
    pub fn new(
        symbol: String,
        underlying: String,
        contract_month: chrono::NaiveDate,
        contract_size: rust_decimal::Decimal,
        tick_size: rust_decimal::Decimal,
    ) -> Self {
        Self {
            symbol,
            underlying,
            contract_month,
            contract_size,
            tick_size,
            settlement_type: SettlementType::Cash, // Default for most B3 futures
        }
    }

    pub fn category(&self) -> B3AssetCategory {
        B3AssetCategory::Future
    }

    pub fn days_to_expiry(&self) -> i64 {
        let today = chrono::Utc::now().date_naive();
        (self.contract_month - today).num_days()
    }

    pub fn is_expired(&self) -> bool {
        let today = chrono::Utc::now().date_naive();
        self.contract_month <= today
    }
}

/// Brazilian ETF asset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3ETF {
    pub symbol: String,
    pub fund_name: String,
    pub isin: Option<String>,
    pub benchmark: Option<String>,
    pub management_fee: Option<rust_decimal::Decimal>,
}

impl Asset for B3ETF {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn asset_type(&self) -> AssetType {
        AssetType::ETF
    }
}

impl B3ETF {
    pub fn new(symbol: String, fund_name: String) -> Self {
        Self {
            symbol,
            fund_name,
            isin: None,
            benchmark: None,
            management_fee: None,
        }
    }

    pub fn category(&self) -> B3AssetCategory {
        B3AssetCategory::ETF
    }
}

/// Brazilian Real Estate Investment Trust (FII)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3REIT {
    pub symbol: String,
    pub fund_name: String,
    pub isin: Option<String>,
    pub property_type: Option<String>,
    pub management_fee: Option<rust_decimal::Decimal>,
    pub dividend_yield: Option<rust_decimal::Decimal>,
}

impl Asset for B3REIT {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn asset_type(&self) -> AssetType {
        AssetType::REIT
    }
}

impl B3REIT {
    pub fn new(symbol: String, fund_name: String) -> Self {
        Self {
            symbol,
            fund_name,
            isin: None,
            property_type: None,
            management_fee: None,
            dividend_yield: None,
        }
    }

    pub fn category(&self) -> B3AssetCategory {
        B3AssetCategory::REIT
    }
}

/// Factory for creating B3 assets from symbols
#[derive(Debug)]
pub struct B3AssetFactory;

impl B3AssetFactory {
    /// Create a B3 asset from a symbol
    ///
    /// B3 symbol conventions:
    /// - Stocks: 4 letters + 1-2 digits (e.g., PETR4, VALE3)
    /// - Options: Complex format with underlying + strike + expiry
    /// - Futures: Underlying + month code (e.g., WINM23)
    /// - ETFs: Usually end with 11 (e.g., BOVA11)
    /// - REITs: Usually end with 11B (e.g., HGLG11)
    pub fn from_symbol(symbol: &str) -> Result<Box<dyn Asset>, String> {
        let symbol = symbol.to_uppercase();

        // ETF detection (ends with 11)
        if symbol.len() >= 6 && symbol.ends_with("11") && !symbol.ends_with("11B") {
            return Ok(Box::new(B3ETF::new(
                symbol.clone(),
                format!("ETF {symbol}"),
            )));
        }

        // REIT detection (ends with 11B or just 11 with B pattern)
        if symbol.ends_with("11B") || (symbol.len() >= 6 && symbol.ends_with("11")) {
            return Ok(Box::new(B3REIT::new(
                symbol.clone(),
                format!("REIT {symbol}"),
            )));
        }

        // Stock detection (4 letters + 1-2 digits)
        if symbol.len() >= 5 && symbol.len() <= 6 {
            let (letters, numbers) = symbol.split_at(4);
            if letters.chars().all(|c| c.is_alphabetic()) && numbers.chars().all(|c| c.is_numeric())
            {
                return Ok(Box::new(B3Stock::new(
                    symbol.clone(),
                    format!("Company {letters}"),
                )));
            }
        }

        // Default to stock if pattern doesn't match
        Ok(Box::new(B3Stock::new(
            symbol.clone(),
            format!("Asset {symbol}"),
        )))
    }

    /// Create a B3 option from symbol and details
    pub fn create_option(
        symbol: String,
        underlying_symbol: String,
        option_type: OptionType,
        strike_price: rust_decimal::Decimal,
        expiry_date: chrono::NaiveDate,
    ) -> B3Option {
        B3Option::new(
            symbol,
            underlying_symbol,
            option_type,
            strike_price,
            expiry_date,
        )
    }

    /// Create a B3 future from symbol and details
    pub fn create_future(
        symbol: String,
        underlying: String,
        contract_month: chrono::NaiveDate,
        contract_size: rust_decimal::Decimal,
        tick_size: rust_decimal::Decimal,
    ) -> B3Future {
        B3Future::new(symbol, underlying, contract_month, contract_size, tick_size)
    }
}

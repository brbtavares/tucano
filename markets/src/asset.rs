//! Core asset abstractions

use serde::{Deserialize, Serialize};
use std::fmt;

/// Core trait for financial assets
pub trait Asset {
    fn symbol(&self) -> &str;
    fn asset_type(&self) -> AssetType;
}

/// Basic asset types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Currency,
    Stock,
    Future,
    Option,
    Fund,
    Bond,
    ETF,
    REIT,
    Other,
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetType::Currency => write!(f, "Currency"),
            AssetType::Stock => write!(f, "Stock"),
            AssetType::Future => write!(f, "Future"),
            AssetType::Option => write!(f, "Option"),
            AssetType::Fund => write!(f, "Fund"),
            AssetType::Bond => write!(f, "Bond"),
            AssetType::ETF => write!(f, "ETF"),
            AssetType::REIT => write!(f, "REIT"),
            AssetType::Other => write!(f, "Other"),
        }
    }
}

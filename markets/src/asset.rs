//! Core asset abstractions

use serde::{Deserialize, Serialize};

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
    Other,
}

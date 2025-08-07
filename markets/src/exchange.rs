use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Constructor,
)]
pub struct ExchangeIndex(pub usize);

impl ExchangeIndex {
    pub fn index(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for ExchangeIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ExchangeIndex({})", self.0)
    }
}

/// Core trait for exchanges - simplified abstraction
pub trait Exchange {
    type ExchangeId: Copy + Eq + std::hash::Hash + std::fmt::Debug;
    
    fn id(&self) -> Self::ExchangeId;
    fn name(&self) -> &'static str;
}

/// Simplified exchange ID enum - focused on essential exchanges
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Display,
)]
#[serde(rename = "execution", rename_all = "snake_case")]
pub enum ExchangeId {
    Other,
    Simulated,
    Mock,
    /// Brazilian Stock Exchange (B3)
    B3,
}

impl ExchangeId {
    /// Return the &str representation of this [`ExchangeId`]
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeId::Other => "other",
            ExchangeId::Simulated => "simulated", 
            ExchangeId::Mock => "mock",
            ExchangeId::B3 => "b3",
        }
    }
}

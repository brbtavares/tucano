//! Temporary compatibility types for the hybrid architecture

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Temporary type to replace complex market data references
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarketDataInstrument {
    pub symbol: String,
    pub kind: MarketDataInstrumentKind,
}

// Implementação única que funciona com strings e AssetNameInternal
impl<S> From<(S, S, MarketDataInstrumentKind)> for MarketDataInstrument 
where 
    S: Into<String> + Into<AssetNameInternal>
{
    fn from((base, _quote, kind): (S, S, MarketDataInstrumentKind)) -> Self {
        Self {
            symbol: base.into(),
            kind,
        }
    }
}

/// Temporary type to replace complex asset name references  
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AssetNameInternal(pub String);

impl AsRef<str> for AssetNameInternal {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for AssetNameInternal {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AssetNameInternal {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<&String> for AssetNameInternal {
    fn from(s: &String) -> Self {
        Self(s.clone())
    }
}

/// Temporary type to replace complex instrument name references
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstrumentNameExchange(pub String);

impl AsRef<str> for InstrumentNameExchange {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Temporary market data instrument kind
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MarketDataInstrumentKind {
    Spot,
    Future,
    Option,
    Perpetual,
}

impl Display for MarketDataInstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketDataInstrumentKind::Spot => write!(f, "spot"),
            MarketDataInstrumentKind::Future => write!(f, "future"),
            MarketDataInstrumentKind::Option => write!(f, "option"),
            MarketDataInstrumentKind::Perpetual => write!(f, "perpetual"),
        }
    }
}

/// Temporary type for asset types during migration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetTypeTemp {
    Currency,
    Stock,
    Future,
    Option,
}

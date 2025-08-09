//! Core instrument abstractions

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Core trait for financial instruments
pub trait Instrument {
    type Symbol: Display + Clone;

    fn symbol(&self) -> &Self::Symbol;
    fn market(&self) -> &str;
}

/// Basic instrument types
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InstrumentKind {
    Spot,
    Future,
    Option,
    Perpetual,
}

impl Display for InstrumentKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrumentKind::Spot => write!(f, "spot"),
            InstrumentKind::Future => write!(f, "future"),
            InstrumentKind::Option => write!(f, "option"),
            InstrumentKind::Perpetual => write!(f, "perpetual"),
        }
    }
}

/// Simple market data instrument struct
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MarketDataInstrument {
    pub symbol: String,
    pub kind: InstrumentKind,
}

impl MarketDataInstrument {
    pub fn new(symbol: String, kind: InstrumentKind) -> Self {
        Self { symbol, kind }
    }
}

impl<S> From<(S, S, InstrumentKind)> for MarketDataInstrument
where
    S: Into<String>,
{
    fn from((base, _quote, kind): (S, S, InstrumentKind)) -> Self {
        Self {
            symbol: base.into(),
            kind,
        }
    }
}

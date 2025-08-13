//! B3 Exchange implementation

use serde::{Deserialize, Serialize};
use tucano_markets::{Exchange, ExchangeId};

/// B3 Exchange struct
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct B3Exchange;

/// Implement markets::Exchange trait for B3Exchange
impl Exchange for B3Exchange {
    type ExchangeId = ExchangeId;

    fn id(&self) -> Self::ExchangeId {
        ExchangeId::B3
    }

    fn name(&self) -> &'static str {
        "B3"
    }
}

impl Default for B3Exchange {
    fn default() -> Self {
        B3Exchange
    }
}

impl std::fmt::Display for B3Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B3")
    }
}

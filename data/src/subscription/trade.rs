
use super::SubscriptionKind;
use serde::{Deserialize, Serialize};
// use toucan_macros::{DeSubKind, SerSubKind}; // removed: crate no longer exists
use toucan_instrument::Side;

/// Toucan [`Subscription`](super::Subscription) [`SubscriptionKind`] that yields [`PublicTrade`]
/// [`MarketEvent<T>`](crate::event::MarketEvent) events.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct PublicTrades;

impl SubscriptionKind for PublicTrades {
    type Event = PublicTrade;

    fn as_str(&self) -> &'static str {
        "public_trades"
    }
}

impl std::fmt::Display for PublicTrades {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Normalised Toucan [`PublicTrade`] model.
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct PublicTrade {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: Side,
}

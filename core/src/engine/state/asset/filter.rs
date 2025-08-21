
use serde::{Deserialize, Serialize};
use toucan_integration::collection::one_or_many::OneOrMany;
use toucan_instrument::exchange::ExchangeId;

/// Asset filter.
///
/// Used to filter asset-centric data structures such as `AssetStates`.
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum AssetFilter {
    None,
    Exchanges(OneOrMany<ExchangeId>),
}

impl AssetFilter {
    pub fn exchanges(exchanges: impl IntoIterator<Item = ExchangeId>) -> Self {
        Self::Exchanges(OneOrMany::from_iter(exchanges))
    }
}

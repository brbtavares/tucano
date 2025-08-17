// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Constructor,
)]
pub struct AssetBalance<AssetKey> {
    pub asset: AssetKey,
    pub balance: Balance,
    pub time_exchange: DateTime<Utc>,
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Deserialize,
    Serialize,
    Constructor,
)]
pub struct Balance {
    pub total: Decimal,
    pub free: Decimal,
}

impl Balance {
    pub fn used(&self) -> Decimal {
        self.total - self.free
    }
}

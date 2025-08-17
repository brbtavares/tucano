// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::engine::Processor;
use serde::{Deserialize, Serialize};
use tucano_data::event::MarketEvent;
use tucano_execution::AccountEvent;

/// Empty `GlobalData` that can be used for `Strategy` and `RiskManager` combinations that
/// require no specific global data state.
#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Deserialize, Serialize,
)]
pub struct DefaultGlobalData;

impl<ExchangeKey, AssetKey, InstrumentKey>
    Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for DefaultGlobalData
{
    type Audit = ();
    fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
}

impl<InstrumentKey, Kind> Processor<&MarketEvent<InstrumentKey, Kind>> for DefaultGlobalData {
    type Audit = ();
    fn process(&mut self, _: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {}
}

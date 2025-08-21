
use crate::{compat::*, error::KeyError};
use fnv::FnvHashMap;
use toucan_instrument::Keyed;
use toucan_integration::collection::{FnvIndexMap, FnvIndexSet};

// Use the core representation of IndexedInstruments (Vec<Keyed<InstrumentIndex, MarketDataInstrument>>)
// without creating a hard compile-time dependency (keep lightweight placeholder for now).
// We'll accept any slice of Keyed instrument indices from the caller.
use toucan_instrument::MarketDataInstrument;
pub type IndexedInstruments = Vec<Keyed<InstrumentIndex, MarketDataInstrument>>;

/// Indexed instrument map used to associate the internal Toucan representation of instruments and
/// assets with the [`ExecutionClient`](super::client::ExecutionClient) representation.
///
/// Similarly, when the execution manager received an [`AccountEvent`](super::AccountEvent)
/// from the execution API, it needs to determine the internal representation of the associated
/// assets and instruments.
///
/// eg/ `InstrumentNameExchange("XBT-USDT")` <--> `InstrumentIndex(1)` <br>
/// eg/ `AssetNameExchange("XBT")` <--> `AssetIndex(1)`
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ExecutionInstrumentMap {
    pub exchange: Keyed<ExchangeIndex, ExchangeId>,
    pub assets: FnvIndexSet<AssetNameExchange>,
    pub instruments: FnvIndexSet<InstrumentNameExchange>,
    pub asset_names: FnvHashMap<AssetNameExchange, TAssetIndex>,
    pub instrument_names: FnvHashMap<InstrumentNameExchange, TInstrumentIndex>,
}

// MIGRATION STEP: if the `typed_indices` feature is active, reexports the newtypes
// and defines local aliases for internal use in this module.
#[cfg(feature = "typed_indices")]
use crate::compat::typed as typed_indices;
#[cfg(feature = "typed_indices")]
type TAssetIndex = typed_indices::AssetIndex;
#[cfg(not(feature = "typed_indices"))]
type TAssetIndex = AssetIndex;
#[cfg(feature = "typed_indices")]
type TInstrumentIndex = typed_indices::InstrumentIndex;
#[cfg(not(feature = "typed_indices"))]
type TInstrumentIndex = InstrumentIndex;

impl ExecutionInstrumentMap {
    /// Construct a new [`Self`] using the provided indexed assets and instruments.
    pub fn new(
        exchange: Keyed<ExchangeIndex, ExchangeId>,
        assets: FnvIndexMap<TAssetIndex, AssetNameExchange>,
        instruments: FnvIndexMap<TInstrumentIndex, InstrumentNameExchange>,
    ) -> Self {
        Self {
            exchange,
            asset_names: assets
                .iter()
                .map(|(key, value)| (value.clone(), key.clone()))
                .collect(),
            instrument_names: instruments
                .iter()
                .map(|(key, value)| (value.clone(), key.clone()))
                .collect(),
            assets: assets.into_values().collect(),
            instruments: instruments.into_values().collect(),
        }
    }

    pub fn exchange_assets(&self) -> impl Iterator<Item = &AssetNameExchange> {
        self.assets.iter()
    }

    pub fn exchange_instruments(&self) -> impl Iterator<Item = &InstrumentNameExchange> {
        self.instruments.iter()
    }

    pub fn find_exchange_id(&self, exchange: ExchangeIndex) -> Result<ExchangeId, KeyError> {
        Ok(ExchangeId::Other)
    }
    pub fn find_exchange_index(&self, exchange: ExchangeId) -> Result<ExchangeIndex, IndexError> {
        Ok(exchange.to_string())
    }

    pub fn find_asset_name_exchange(
        &self,
        asset: AssetIndex,
    ) -> Result<&AssetNameExchange, KeyError> {
    // When the typed_indices feature is active, we need to look up the corresponding name
    // by the stored value (index). Since it's a HashMap name -> index, we do a linear search.
    // Acceptable cost while indices are small; can be optimized (reverse map) if needed.
        #[cfg(feature = "typed_indices")]
        {
            self.asset_names
                .iter()
                .find(|(_, v)| v.as_str() == asset)
                .map(|(k, _)| k)
                .ok_or_else(|| {
                    KeyError::AssetKey(format!("ExecutionInstrumentMap does not contain: {asset}"))
                })
        }
        #[cfg(not(feature = "typed_indices"))]
        {
            // Sem typed_indices, AssetIndex == String e TAssetIndex == String
            self.asset_names.get(&asset).ok_or_else(|| {
                KeyError::AssetKey(format!("ExecutionInstrumentMap does not contain: {asset}"))
            })
        }
    }

    pub fn find_asset_index(&self, asset: &AssetNameExchange) -> Result<AssetIndex, IndexError> {
        #[cfg(feature = "typed_indices")]
        {
            self.asset_names
                .get(asset)
                .map(|v| v.to_string())
                .ok_or_else(|| {
                    IndexError::AssetIndex(format!(
                        "ExecutionInstrumentMap does not contain: {asset}"
                    ))
                })
        }
        #[cfg(not(feature = "typed_indices"))]
        {
            self.asset_names.get(asset).cloned().ok_or_else(|| {
                IndexError::AssetIndex(format!("ExecutionInstrumentMap does not contain: {asset}"))
            })
        }
    }

    pub fn find_instrument_name_exchange(
        &self,
        instrument: InstrumentIndex,
    ) -> Result<&InstrumentNameExchange, KeyError> {
        #[cfg(feature = "typed_indices")]
        {
            self.instrument_names
                .iter()
                .find(|(_, v)| v.as_str() == instrument)
                .map(|(k, _)| k)
                .ok_or_else(|| {
                    KeyError::InstrumentKey(format!(
                        "ExecutionInstrumentMap does not contain: {instrument}"
                    ))
                })
        }
        #[cfg(not(feature = "typed_indices"))]
        {
            self.instrument_names.get(&instrument).ok_or_else(|| {
                KeyError::InstrumentKey(format!(
                    "ExecutionInstrumentMap does not contain: {instrument}"
                ))
            })
        }
    }

    pub fn find_instrument_index(
        &self,
        instrument: &InstrumentNameExchange,
    ) -> Result<InstrumentIndex, IndexError> {
        #[cfg(feature = "typed_indices")]
        {
            self.instrument_names
                .get(instrument)
                .map(|v| v.to_string())
                .ok_or_else(|| {
                    IndexError::InstrumentIndex(format!(
                        "ExecutionInstrumentMap does not contain: {instrument}"
                    ))
                })
        }
        #[cfg(not(feature = "typed_indices"))]
        {
            self.instrument_names
                .get(instrument)
                .cloned()
                .ok_or_else(|| {
                    IndexError::InstrumentIndex(format!(
                        "ExecutionInstrumentMap does not contain: {instrument}"
                    ))
                })
        }
    }
}

pub fn generate_execution_instrument_map(
    _instruments: &IndexedInstruments,
    exchange: ExchangeId,
) -> Result<ExecutionInstrumentMap, IndexError> {
    // TODO: Build real mapping from provided instruments. For now return empty map for compile.
    Ok(ExecutionInstrumentMap::new(
        Keyed::new(exchange.to_string(), exchange),
        FnvIndexMap::default(),
        FnvIndexMap::default(),
    ))
}

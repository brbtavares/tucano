use crate::{compat::*, error::KeyError};
use fnv::FnvHashMap;
use integration::collection::{FnvIndexMap, FnvIndexSet};
use markets::Keyed;

// Use the core representation of IndexedInstruments (Vec<Keyed<InstrumentIndex, ConcreteInstrument>>)
// without creating a hard compile-time dependency (keep lightweight placeholder for now).
// We'll accept any slice of Keyed instrument indices from the caller.
use markets::ConcreteInstrument;
pub type IndexedInstruments = Vec<Keyed<InstrumentIndex, ConcreteInstrument>>;

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
    pub asset_names: FnvHashMap<AssetNameExchange, AssetIndex>,
    pub instrument_names: FnvHashMap<InstrumentNameExchange, InstrumentIndex>,
}

impl ExecutionInstrumentMap {
    /// Construct a new [`Self`] using the provided indexed assets and instruments.
    pub fn new(
    exchange: Keyed<ExchangeIndex, ExchangeId>,
        assets: FnvIndexMap<AssetIndex, AssetNameExchange>,
        instruments: FnvIndexMap<InstrumentIndex, InstrumentNameExchange>,
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
        Ok(ExchangeId::from(exchange.as_str()))
    }
    pub fn find_exchange_index(&self, exchange: ExchangeId) -> Result<ExchangeIndex, IndexError> {
        Ok(exchange.to_string())
    }

    pub fn find_asset_name_exchange(
        &self,
        asset: AssetIndex,
    ) -> Result<&AssetNameExchange, KeyError> {
        // Como AssetIndex é String agora, podemos procurar diretamente
        self.asset_names.get(&asset).ok_or_else(|| {
            KeyError::AssetKey(format!("ExecutionInstrumentMap does not contain: {asset}"))
        })
    }

    pub fn find_asset_index(&self, asset: &AssetNameExchange) -> Result<AssetIndex, IndexError> {
        self.asset_names.get(asset).cloned().ok_or_else(|| {
            IndexError::AssetIndex(format!("ExecutionInstrumentMap does not contain: {asset}"))
        })
    }

    pub fn find_instrument_name_exchange(
        &self,
        instrument: InstrumentIndex,
    ) -> Result<&InstrumentNameExchange, KeyError> {
        // Como InstrumentIndex é String agora, podemos procurar diretamente
        self.instrument_names.get(&instrument).ok_or_else(|| {
            KeyError::InstrumentKey(format!(
                "ExecutionInstrumentMap does not contain: {instrument}"
            ))
        })
    }

    pub fn find_instrument_index(
        &self,
        instrument: &InstrumentNameExchange,
    ) -> Result<InstrumentIndex, IndexError> {
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

// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::{compat::*, error::KeyError};
use fnv::FnvHashMap;
use tucano_integration::collection::{FnvIndexMap, FnvIndexSet};
use tucano_markets::Keyed;

// Use the core representation of IndexedInstruments (Vec<Keyed<InstrumentIndex, ConcreteInstrument>>)
// without creating a hard compile-time dependency (keep lightweight placeholder for now).
// We'll accept any slice of Keyed instrument indices from the caller.
use tucano_markets::ConcreteInstrument;
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
    pub asset_names: FnvHashMap<AssetNameExchange, TAssetIndex>,
    pub instrument_names: FnvHashMap<InstrumentNameExchange, TInstrumentIndex>,
}

// MIGRATION STEP: se a feature `typed_indices` estiver ativa, reexporta os newtypes
// e define aliases locais para uso interno deste módulo.
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
        Ok(ExchangeId::from(exchange.as_str()))
    }
    pub fn find_exchange_index(&self, exchange: ExchangeId) -> Result<ExchangeIndex, IndexError> {
        Ok(exchange.to_string())
    }

    pub fn find_asset_name_exchange(
        &self,
        asset: AssetIndex,
    ) -> Result<&AssetNameExchange, KeyError> {
        // Quando a feature typed_indices está ativa, precisamos procurar o nome correspondente
        // pelo valor (índice) armazenado. Como é um HashMap name -> index, fazemos uma busca linear.
        // Custo aceitável enquanto os índices são pequenos; pode ser otimizado (mapa reverso) se necessário.
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

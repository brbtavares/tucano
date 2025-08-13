use crate::engine::state::{
    asset::generate_empty_indexed_asset_states,
    connectivity::generate_empty_indexed_connectivity_states,
    instrument::generate_indexed_instrument_states, order::Orders, position::PositionManager,
    trading::TradingState, EngineState,
};
use chrono::{DateTime, Utc};
use fnv::FnvHashMap;
use tracing::debug;
use tucano_execution::{
    balance::{AssetBalance, Balance},
    InstrumentIndex,
};
use tucano_integration::snapshot::Snapshot;
use tucano_markets::{ConcreteInstrument, Keyed};

/// Placeholder types
use super::asset::AssetNameInternal;
use super::IndexedInstruments;

/// Builder utility for an [`EngineState`] instance.
#[derive(Debug, Clone)]
pub struct EngineStateBuilder<'a, GlobalData, FnInstrumentData> {
    instruments: &'a IndexedInstruments,
    trading_state: Option<TradingState>,
    time_engine_start: Option<DateTime<Utc>>,
    global: GlobalData,
    balances: FnvHashMap<AssetNameInternal, Balance>,
    instrument_data_init: FnInstrumentData,
}

impl<'a, GlobalData, FnInstrumentData> EngineStateBuilder<'a, GlobalData, FnInstrumentData> {
    /// Construct a new `EngineStateBuilder` with a layout derived from [`IndexedInstruments`].
    ///
    /// Note that the rest of the [`EngineState`] data can be generated from defaults if that
    /// is all that is needed.
    ///
    /// Note that `ConnectivityStates` will be generated with
    /// [`generate_empty_indexed_connectivity_states`], defaulting to `Health::Reconnecting`.
    pub fn new(
        instruments: &'a IndexedInstruments,
        global: GlobalData,
        instrument_data_init: FnInstrumentData,
    ) -> Self {
        Self {
            instruments,
            time_engine_start: None,
            trading_state: None,
            global,
            balances: FnvHashMap::default(),
            instrument_data_init,
        }
    }

    /// Optionally provide the initial `TradingState`.
    ///
    /// Defaults to `TradingState::Disabled`.
    pub fn trading_state(self, value: TradingState) -> Self {
        Self {
            trading_state: Some(value),
            ..self
        }
    }

    /// Optionally provide the `time_engine_start`.
    ///
    /// Providing this is useful for back-test scenarios where the time should be seeded with a
    /// "historical" clock time (eg/ from first historical `MarketEvent`).
    ///
    /// Defaults to `Utc::now`
    pub fn time_engine_start(self, value: DateTime<Utc>) -> Self {
        Self {
            time_engine_start: Some(value),
            ..self
        }
    }

    /// Optionally provide initial exchange asset `Balance`s.
    ///
    /// Useful for back-test scenarios where seeding EngineState with initial `Balance`s is
    /// required.
    ///
    /// Note the internal implementation uses a `HashMap`, so duplicate
    /// `ExchangeAsset<AssetNameInternal>` keys are overwritten.
    pub fn balances<BalanceIter>(mut self, balances: BalanceIter) -> Self
    where
        BalanceIter: IntoIterator<Item = (AssetNameInternal, Balance)>,
    {
        self.balances.extend(balances);
        self
    }

    /// Use the builder data to generate the associated [`EngineState`].
    ///
    /// If optional data is not provided (eg/ Balances), default values are used (eg/ zero Balance).
    pub fn build<InstrumentData>(self) -> EngineState<GlobalData, InstrumentData>
    where
        FnInstrumentData: Fn(&'a Keyed<InstrumentIndex, ConcreteInstrument>) -> InstrumentData,
    {
        let Self {
            instruments,
            time_engine_start,
            trading_state,
            global,
            balances,
            instrument_data_init,
        } = self;

        // Default if not provided
        let time_engine_start = time_engine_start.unwrap_or_else(|| {
            debug!("EngineStateBuilder using Utc::now as time_engine_start default");
            Utc::now()
        });
        let trading = trading_state.unwrap_or_default();

        // Construct empty ConnectivityStates
        let connectivity = generate_empty_indexed_connectivity_states(instruments);

        // Update empty AssetStates from provided exchange asset Balances
        let mut assets = generate_empty_indexed_asset_states(instruments);
        for (asset_name, balance) in balances {
            if !assets.0.contains_key(&asset_name) {
                assets.0.insert(
                    asset_name.clone(),
                    crate::engine::state::asset::AssetState::new(
                        asset_name.clone(),
                        tucano_analytics::summary::asset::TearSheetAssetGenerator::default(),
                        None,
                    ),
                );
            }
            assets
                .asset_mut(&asset_name)
                .update_from_balance(Snapshot(&AssetBalance {
                    asset: asset_name.clone(),
                    balance,
                    time_exchange: time_engine_start,
                }))
        }

        // Generate empty InstrumentStates using provided FnInstrumentData etc.
        let instruments = generate_indexed_instrument_states(
            instruments,
            time_engine_start,
            PositionManager::default,
            Orders::default,
            instrument_data_init,
        );

        EngineState {
            trading,
            global,
            connectivity,
            assets,
            instruments,
        }
    }
}

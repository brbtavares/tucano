use crate::{
    summary::{
        asset::{TearSheetAsset, TearSheetAssetGenerator},
        instrument::{TearSheet, TearSheetGenerator},
    },
    time::TimeInterval,
};
use execution::{balance::AssetBalance, AssetIndex, InstrumentIndex};
use integration::collection::FnvIndexMap;

// Placeholder name types for integration - these will be properly defined during full integration
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AssetNameInternal(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct InstrumentNameInternal(pub String);

impl AssetNameInternal {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AssetNameInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl InstrumentNameInternal {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InstrumentNameInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Allow lookups by &str in IndexMap
impl std::borrow::Borrow<str> for InstrumentNameInternal {
    fn borrow(&self) -> &str { &self.0 }
}

// Placeholder for ExchangeAsset - simplified for integration
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ExchangeAsset<T> {
    pub asset: T,
    pub exchange: String,
}

impl<T> ExchangeAsset<T> {
    pub fn new(asset: T, exchange: String) -> Self {
        Self { asset, exchange }
    }
}
use chrono::{DateTime, TimeDelta, Utc};
use derive_more::Constructor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Placeholder for AssetStates - this will be properly resolved when integrating
pub type AssetStates<AssetKey> = FnvIndexMap<AssetIndex, AssetBalance<AssetKey>>;

/// Placeholder for InstrumentStates - this will be properly resolved when integrating
pub type InstrumentStates = FnvIndexMap<InstrumentIndex, ()>;

/// Placeholder for LocalSnapshot - this will be properly resolved when integrating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSnapshot<T>(pub T);

impl<T> LocalSnapshot<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::Deref for LocalSnapshot<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Placeholder for PositionExited - this will be properly resolved when integrating
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionExited {
    pub timestamp: DateTime<Utc>,
    pub pnl_realised: Decimal,
    pub time_exit: DateTime<Utc>,
    pub instrument: InstrumentIndex,
    pub price_entry_average: Decimal,
    pub quantity_abs_max: Decimal,
}

pub mod asset;
pub mod dataset;
pub mod display;
pub mod instrument;
pub mod pnl;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct TradingSummary<Interval> {
    /// Trading session start time defined by the [`Engine`](crate::engine::Engine) clock.
    pub time_engine_start: DateTime<Utc>,

    /// Trading session end time defined by the [`Engine`](crate::engine::Engine) clock.
    pub time_engine_end: DateTime<Utc>,

    /// Instrument [`TearSheet`]s.
    ///
    /// Note that an Instrument is unique to an exchange, so, for example, Binance btc_usdt_spot
    /// and B3 petr4_brl_spot will be summarised by distinct [`TearSheet`]s.
    pub instruments: FnvIndexMap<InstrumentNameInternal, TearSheet<Interval>>,

    /// [`ExchangeAsset`] [`TearSheet`]s.
    pub assets: FnvIndexMap<ExchangeAsset<AssetNameInternal>, TearSheetAsset>,
}

impl<Interval> TradingSummary<Interval> {
    /// Duration of trading that the `TradingSummary` covers.
    pub fn trading_duration(&self) -> TimeDelta {
        self.time_engine_end
            .signed_duration_since(self.time_engine_start)
    }
}

/// Generator for a [`TradingSummary`].
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct TradingSummaryGenerator {
    /// Theoretical rate of return of an investment with zero risk.
    ///
    /// See docs: <https://www.investopedia.com/terms/r/risk-freerate.asp>
    pub risk_free_return: Decimal,

    /// Trading session summary start time defined by the [`Engine`](crate::engine::Engine) clock.
    pub time_engine_start: DateTime<Utc>,

    /// Trading session summary most recent update time defined by the
    /// [`Engine`](crate::engine::Engine) clock.
    pub time_engine_now: DateTime<Utc>,

    /// Instrument [`TearSheetGenerator`]s.
    ///
    /// Note that an Instrument is unique to an exchange, so, for example, Binance btc_usdt_spot
    /// and B3 petr4_brl_spot will be summarised by distinct [`TearSheet`]s.
    pub instruments: FnvIndexMap<InstrumentNameInternal, TearSheetGenerator>,

    /// [`ExchangeAsset`] [`TearSheetAssetGenerator`]s.
    pub assets: FnvIndexMap<ExchangeAsset<AssetNameInternal>, TearSheetAssetGenerator>,
}

impl TradingSummaryGenerator {
    /// Initialise a [`TradingSummaryGenerator`] from a `risk_free_return` value, and initial
    /// indexed state.
    pub fn init<InstrumentData, AssetKey>(
        risk_free_return: Decimal,
        time_engine_start: DateTime<Utc>,
        time_engine_now: DateTime<Utc>,
        _instruments: &InstrumentStates,
        _assets: &AssetStates<AssetKey>,
    ) -> Self {
        Self {
            risk_free_return,
            time_engine_start,
            time_engine_now,
            instruments: FnvIndexMap::default(), // Simplified placeholder
            assets: FnvIndexMap::default(),      // Simplified placeholder
        }
    }

    /// Update the [`TradingSummaryGenerator`] `time_now`.
    pub fn update_time_now(&mut self, time_now: DateTime<Utc>) {
        self.time_engine_now = time_now;
    }

    /// Update the [`TradingSummaryGenerator`] from the next [`PositionExited`].
    pub fn update_from_position<AssetKey, InstrumentKey>(&mut self, position: &PositionExited)
    where
        Self: InstrumentTearSheetManager<InstrumentIndex>,
    {
        if self.time_engine_now < position.time_exit {
            self.time_engine_now = position.time_exit;
        }

        self.instrument_mut(&position.instrument)
            .update_from_position::<AssetKey, InstrumentKey>(position)
    }

    /// Update the [`TradingSummaryGenerator`] from the next [`LocalSnapshot`] [`AssetBalance`].
    pub fn update_from_balance<AssetKey>(&mut self, balance: LocalSnapshot<&AssetBalance<AssetKey>>)
    where
        Self: AssetTearSheetManager<AssetKey>,
    {
        if self.time_engine_now < balance.0.time_exchange {
            self.time_engine_now = balance.0.time_exchange;
        }

        // For simplicity, we'll just update the time since we don't have the full asset manager
        // This is a placeholder implementation
    }

    /// Generate the latest [`TradingSummary`] at the specific [`TimeInterval`].
    ///
    /// For example, pass [`Annual365`](super::time::Annual365) to generate a crypto-centric
    /// (24/7 trading) annualised [`TradingSummary`].
    pub fn generate<Interval>(&mut self, interval: Interval) -> TradingSummary<Interval>
    where
        Interval: TimeInterval + Copy,
    {
        let instruments = self
            .instruments
            .iter_mut()
            .map(|(instrument, tear_sheet)| (instrument.clone(), tear_sheet.generate(interval)))
            .collect();

        let assets = self
            .assets
            .iter_mut()
            .map(|(asset, tear_sheet)| (asset.clone(), tear_sheet.generate()))
            .collect();

        TradingSummary {
            time_engine_start: self.time_engine_start,
            time_engine_end: self.time_engine_now,
            instruments,
            assets,
        }
    }
}

pub trait InstrumentTearSheetManager<InstrumentKey> {
    fn instrument(&self, key: &InstrumentKey) -> &TearSheetGenerator;
    fn instrument_mut(&mut self, key: &InstrumentKey) -> &mut TearSheetGenerator;
}

impl InstrumentTearSheetManager<InstrumentNameInternal> for TradingSummaryGenerator {
    fn instrument(&self, key: &InstrumentNameInternal) -> &TearSheetGenerator {
        self.instruments
            .get(key)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator does not contain: {key}"))
    }

    fn instrument_mut(&mut self, key: &InstrumentNameInternal) -> &mut TearSheetGenerator {
        self.instruments
            .get_mut(key)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator does not contain: {key}"))
    }
}

impl InstrumentTearSheetManager<InstrumentIndex> for TradingSummaryGenerator {
    fn instrument(&self, _key: &InstrumentIndex) -> &TearSheetGenerator {
        // For simplicity in integration mode, return a default value
        // This will be properly implemented when integrating with real instrument management
        self.instruments
            .iter()
            .next()
            .map(|(_k, v)| v)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator has no instruments available"))
    }

    fn instrument_mut(&mut self, _key: &InstrumentIndex) -> &mut TearSheetGenerator {
        // For simplicity in integration mode, return a default value
        // This will be properly implemented when integrating with real instrument management
        self.instruments
            .iter_mut()
            .next()
            .map(|(_k, v)| v)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator has no instruments available"))
    }
}

pub trait AssetTearSheetManager<AssetKey> {
    fn asset(&self, key: &AssetKey) -> &TearSheetAssetGenerator;
    fn asset_mut(&mut self, key: &AssetKey) -> &mut TearSheetAssetGenerator;
}

impl AssetTearSheetManager<AssetIndex> for TradingSummaryGenerator {
    fn asset(&self, _key: &AssetIndex) -> &TearSheetAssetGenerator {
        // For simplicity in integration mode, return a default value
        // This will be properly implemented when integrating with real asset management
        self.assets
            .iter()
            .next()
            .map(|(_k, v)| v)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator has no assets available"))
    }

    fn asset_mut(&mut self, _key: &AssetIndex) -> &mut TearSheetAssetGenerator {
        // For simplicity in integration mode, return a default value
        // This will be properly implemented when integrating with real asset management
        self.assets
            .iter_mut()
            .next()
            .map(|(_k, v)| v)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator has no assets available"))
    }
}

impl AssetTearSheetManager<ExchangeAsset<AssetNameInternal>> for TradingSummaryGenerator {
    fn asset(&self, key: &ExchangeAsset<AssetNameInternal>) -> &TearSheetAssetGenerator {
        self.assets
            .get(key)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator does not contain: {key:?}"))
    }

    fn asset_mut(
        &mut self,
        key: &ExchangeAsset<AssetNameInternal>,
    ) -> &mut TearSheetAssetGenerator {
        self.assets
            .get_mut(key)
            .unwrap_or_else(|| panic!("TradingSummaryGenerator does not contain: {key:?}"))
    }
}

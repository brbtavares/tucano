// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::engine::state::{
    instrument::{data::InstrumentDataState, filter::InstrumentFilter},
    order::{manager::OrderManager, Orders},
    position::{PositionExited, PositionManager},
};
use chrono::{DateTime, Utc};
use derive_more::Constructor;
use itertools::Either;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tucano_analytics::summary::{self, instrument::TearSheetGenerator};
use tucano_data::event::MarketEvent;
use tucano_execution::{
    order::{
        request::OrderResponseCancel,
        state::{ActiveOrderState, OrderState},
        Order, OrderKey,
    },
    trade::Trade,
    AssetIndex, ExchangeIndex, InstrumentAccountSnapshot, InstrumentIndex, QuoteAsset,
};
use tucano_integration::{collection::FnvIndexMap, snapshot::Snapshot};
use tucano_markets::{exchange::ExchangeId, ConcreteInstrument, Keyed};

// ConcreteInstrument now defined in markets crate

/// Placeholder types
pub type AssetNameExchange = String;
pub type InstrumentNameExchange = String;
pub type InstrumentNameInternal = String;

/// Placeholder for IndexedInstruments - reused from parent module
use super::IndexedInstruments;

/// Defines the state interface [`InstrumentDataState`] that can be implemented for custom
/// instrument level data state.
pub mod data;

/// Defines an `InstrumentFilter`, used to filter instrument-centric data structures.
pub mod filter;

/// Collection of [`InstrumentState`]s indexed by [`InstrumentIndex`].
///
/// Note that the same instruments with the same [`InstrumentNameExchange`] (eg/ "btc_usdt") but
/// on different exchanges will have their own [`InstrumentState`].
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InstrumentStates<
    InstrumentData,
    ExchangeKey = ExchangeIndex,
    InstrumentKey = InstrumentIndex,
>(
    pub  FnvIndexMap<
        InstrumentNameInternal,
        InstrumentState<InstrumentData, ExchangeKey, InstrumentKey>,
    >,
);

impl<InstrumentData> InstrumentStates<InstrumentData> {
    /// Return a reference to the `InstrumentState` associated with an `InstrumentIndex`.
    ///
    /// Panics if `InstrumentState` associated with the `InstrumentIndex` does not exist.
    pub fn instrument_index(&self, key: &InstrumentIndex) -> &InstrumentState<InstrumentData> {
        self.0
            .get(key)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    /// Return a mutable reference to the `InstrumentState` associated with an `InstrumentIndex`.
    ///
    /// Panics if `InstrumentState` associated with the `InstrumentIndex` does not exist.
    pub fn instrument_index_mut(
        &mut self,
        key: &InstrumentIndex,
    ) -> &mut InstrumentState<InstrumentData> {
        self.0
            .get_mut(key)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    /// Return a reference to the `InstrumentState` associated with an `InstrumentNameInternal`.
    ///
    /// Panics if `InstrumentState` associated with the `InstrumentNameInternal` does not exist.
    pub fn instrument(&self, key: &InstrumentNameInternal) -> &InstrumentState<InstrumentData> {
        self.0
            .get(key)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    /// Return a mutable reference to the `InstrumentState` associated with an
    /// `InstrumentNameInternal`.
    ///
    /// Panics if `InstrumentState` associated with the `InstrumentNameInternal` does not exist.
    pub fn instrument_mut(
        &mut self,
        key: &InstrumentNameInternal,
    ) -> &mut InstrumentState<InstrumentData> {
        self.0
            .get_mut(key)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    /// Return an `Iterator` of references to `InstrumentState`s being tracked, optionally filtered
    /// by the provided `InstrumentFilter`.
    pub fn instruments<'a>(
        &'a self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a InstrumentState<InstrumentData>> {
        self.filtered(filter)
    }

    /// Return an `Iterator` of mutable references to `InstrumentState`s being tracked, optionally
    /// filtered by the provided `InstrumentFilter`.
    pub fn instruments_mut<'a>(
        &'a mut self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a mut InstrumentState<InstrumentData>> {
        self.filtered_mut(filter)
    }

    /// Return an `Iterator` of references to instrument `TearSheetGenerator`s, optionally
    /// filtered by the provided `InstrumentFilter`.
    pub fn tear_sheets<'a>(
        &'a self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a TearSheetGenerator>
    where
        InstrumentData: 'a,
    {
        self.filtered(filter).map(|state| &state.tear_sheet)
    }

    /// Return an `Iterator` of references to instrument `PositionManager`s, optionally
    /// filtered by the provided `InstrumentFilter`.
    pub fn positions<'a>(
        &'a self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a PositionManager>
    where
        InstrumentData: 'a,
    {
        self.filtered(filter).map(|state| &state.position)
    }

    /// Return an `Iterator` of references to instrument `Orders`, optionally filtered by the
    /// provided `InstrumentFilter`.
    pub fn orders<'a>(&'a self, filter: &'a InstrumentFilter) -> impl Iterator<Item = &'a Orders>
    where
        InstrumentData: 'a,
    {
        self.filtered(filter).map(|state| &state.orders)
    }

    /// Return an `Iterator` of references to custom instrument level data state, optionally
    /// filtered by the provided `InstrumentFilter`.
    pub fn instrument_datas<'a>(
        &'a self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a InstrumentData>
    where
        InstrumentData: 'a,
    {
        self.filtered(filter).map(|state| &state.data)
    }

    /// Return an `Iterator` of mutable references to custom instrument level data state,
    /// optionally filtered by the provided `InstrumentFilter`.
    pub fn instrument_datas_mut<'a>(
        &'a mut self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a mut InstrumentData>
    where
        InstrumentData: 'a,
    {
        self.filtered_mut(filter).map(|state| &mut state.data)
    }

    /// Return a filtered `Iterator` of `InstrumentState`s based on the provided `InstrumentFilter`.
    fn filtered<'a>(
        &'a self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a InstrumentState<InstrumentData>>
    where
        InstrumentData: 'a,
    {
        use filter::InstrumentFilter::*;
        match filter {
            None => Either::Left(Either::Left(self.0.values())),
            Exchanges(exchanges) => {
                // exchanges is OneOrMany<ExchangeIndex> (String) in compatibility layer; compare stringified
                Either::Left(Either::Right(self.0.values().filter(|state| {
                    exchanges.contains(&state.instrument.exchange.to_string())
                })))
            }
            Instruments(instruments) => Either::Right(Either::Right(
                self.0
                    .values()
                    .filter(|state| instruments.contains(&state.key)),
            )),
            Underlyings(_underlying) => {
                // Temporary: ConcreteInstrument.underlying is Option<String>. Skip filtering.
                Either::Right(Either::Left(self.0.values().filter(|_| false)))
            }
        }
    }

    /// Return a filtered `Iterator` of mutable `InstrumentState`s based on the
    /// provided `InstrumentFilter`.
    fn filtered_mut<'a>(
        &'a mut self,
        filter: &'a InstrumentFilter,
    ) -> impl Iterator<Item = &'a mut InstrumentState<InstrumentData>>
    where
        InstrumentData: 'a,
    {
        use filter::InstrumentFilter::*;
        match filter {
            None => Either::Left(Either::Left(self.0.values_mut())),
            Exchanges(exchanges) => {
                Either::Left(Either::Right(self.0.values_mut().filter(|state| {
                    exchanges.contains(&state.instrument.exchange.to_string())
                })))
            }
            Instruments(instruments) => Either::Right(Either::Right(
                self.0
                    .values_mut()
                    .filter(|state| instruments.contains(&state.key)),
            )),
            Underlyings(_underlying) => Either::Right(Either::Left(
                self.0.values_mut().filter(|_| false), // temporarily disabled underlying filter
            )),
        }
    }
}

/// Represents the current state of an instrument, including its [`Position`](super::position::Position), [`Orders`], and
/// user provided instrument data.
///
/// This aggregates all the state and data for a single instrument, providing a comprehensive
/// view of the instrument.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct InstrumentState<
    InstrumentData,
    ExchangeKey = ExchangeIndex,
    InstrumentKey = InstrumentIndex,
> {
    /// Unique `InstrumentKey` identifier for the instrument this state is associated with.
    pub key: InstrumentKey,

    /// Complete instrument definition.
    pub instrument: ConcreteInstrument, // Using concrete instrument type

    /// TearSheet generator for summarising the trading performance associated with an Instrument.
    pub tear_sheet: TearSheetGenerator,

    /// Current `PositionManager`.
    pub position: PositionManager<InstrumentKey>,

    /// Active orders and associated order management.
    pub orders: Orders<ExchangeKey, InstrumentKey>,

    /// User provided instrument level data state. This can include market data, strategy data,
    /// risk data, option pricing data, or any other instrument-specific information.
    pub data: InstrumentData,
}

impl<InstrumentData, ExchangeKey, InstrumentKey>
    InstrumentState<InstrumentData, ExchangeKey, InstrumentKey>
{
    /// Updates the instrument state using an account snapshot from the exchange.
    ///
    /// This updates active orders for the instrument, using timestamps where relevant to ensure
    /// the most recent order state is applied.
    pub fn update_from_account_snapshot(
        &mut self,
        snapshot: &InstrumentAccountSnapshot<ExchangeKey, AssetIndex, InstrumentKey>,
    ) where
        ExchangeKey: Debug + Clone,
        InstrumentKey: Debug + Clone,
    {
        for order in &snapshot.orders {
            self.update_from_order_snapshot(Snapshot(order))
        }
    }

    /// Updates the instrument state from an [`Order`] snapshot.
    pub fn update_from_order_snapshot(
        &mut self,
        order: Snapshot<&Order<ExchangeKey, InstrumentKey, OrderState<AssetIndex, InstrumentKey>>>,
    ) where
        ExchangeKey: Debug + Clone,
        InstrumentKey: Debug + Clone,
    {
        self.orders.update_from_order_snapshot(order);
    }

    /// Updates the instrument state from an
    /// [`OrderRequestCancel`](toucan_execution::order::request::OrderRequestCancel) response.
    pub fn update_from_cancel_response(
        &mut self,
        response: &OrderResponseCancel<ExchangeKey, AssetIndex, InstrumentKey>,
    ) where
        ExchangeKey: Debug + Clone,
        InstrumentKey: Debug + Clone,
    {
        self.orders
            .update_from_cancel_response::<AssetIndex>(response);
    }

    /// Updates the instrument state based on a new trade.
    ///
    /// This method handles:
    /// - Opening/updating the current position state based on a new trade.
    /// - Updating the internal [`TearSheetGenerator`] if a position is exited.
    pub fn update_from_trade(
        &mut self,
        trade: &Trade<QuoteAsset, InstrumentKey>,
    ) -> Option<PositionExited<QuoteAsset, InstrumentKey>>
    where
        InstrumentKey: Debug + Clone + PartialEq,
    {
        self.position.update_from_trade(trade).inspect(|closed| {
            // Convert core PositionExited to analytics PositionExited
            let analytics_position = summary::PositionExited {
                timestamp: closed.time_exit,
                pnl_realised: closed.pnl_realised,
                time_exit: closed.time_exit,
                instrument: "0".to_string(), // Placeholder conversion
                price_entry_average: closed.price_entry_average,
                quantity_abs_max: closed.quantity_abs_max,
            };
            self.tear_sheet
                .update_from_position::<AssetIndex, InstrumentKey>(&analytics_position);
        })
    }

    /// Updates the instrument state based on a new market event.
    ///
    /// If the market event has a price associated with it (eg/ `PublicTrade`, `OrderBookL1`), any
    /// open [`Position`](super::position::Position) `pnl_unrealised` is re-calculated.
    pub fn update_from_market(
        &mut self,
        event: &MarketEvent<InstrumentKey, InstrumentData::MarketEventKind>,
    ) where
        InstrumentData: InstrumentDataState<ExchangeKey, AssetIndex, InstrumentKey>,
    {
        self.data.process(event);

        let Some(position) = &mut self.position.current else {
            return;
        };

        let Some(price) = self.data.price() else {
            return;
        };

        position.update_pnl_unrealised(price);
    }
}

pub fn generate_unindexed_instrument_account_snapshot<InstrumentData, ExchangeKey, InstrumentKey>(
    exchange: ExchangeId,
    state: &InstrumentState<InstrumentData, ExchangeKey, InstrumentKey>,
) -> InstrumentAccountSnapshot<ExchangeId, AssetNameExchange, InstrumentNameExchange>
where
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    let InstrumentState {
        key: _,
        instrument,
        tear_sheet: _,
        position: _,
        orders,
        data: _,
    } = state;

    InstrumentAccountSnapshot {
        instrument: instrument.name_exchange.clone(),
        orders: orders
            .orders()
            .filter_map(|order| {
                let Order {
                    key,
                    side,
                    price,
                    quantity,
                    kind,
                    time_in_force,
                    state: ActiveOrderState::Open(open),
                } = order
                else {
                    return None;
                };

                Some(Order {
                    key: OrderKey {
                        exchange,
                        instrument: instrument.name_exchange.clone(),
                        strategy: key.strategy.clone(),
                        cid: key.cid.clone(),
                    },
                    side: *side,
                    price: *price,
                    quantity: *quantity,
                    kind: *kind,
                    time_in_force: *time_in_force,
                    state: OrderState::active(open.clone()),
                })
            })
            .collect(),
    }
}

/// Generates an indexed [`InstrumentStates`]. Uses default values for
pub fn generate_indexed_instrument_states<'a, FnPosMan, FnOrders, FnInsData, InstrumentData>(
    instruments: &'a IndexedInstruments,
    time_engine_start: DateTime<Utc>,
    position_manager_init: FnPosMan,
    orders_init: FnOrders,
    instrument_data_init: FnInsData,
) -> InstrumentStates<InstrumentData>
where
    FnPosMan: Fn() -> PositionManager,
    FnOrders: Fn() -> Orders,
    FnInsData: Fn(&'a Keyed<InstrumentIndex, ConcreteInstrument>) -> InstrumentData,
{
    InstrumentStates(
        instruments
            .iter()
            .map(|instrument| {
                // Use the provided instrument.key (already a String alias) as the internal key
                (
                    instrument.key.clone(),
                    InstrumentState::new(
                        instrument.key.clone(),
                        instrument.value.clone(),
                        TearSheetGenerator::init(time_engine_start),
                        position_manager_init(),
                        orders_init(),
                        instrument_data_init(instrument),
                    ),
                )
            })
            .collect(),
    )
}

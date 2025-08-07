use crate::{
    algo::AlgoStrategy,
    close_positions::ClosePositionsStrategy,
    on_disconnect::OnDisconnectStrategy,
    on_trading_disabled::OnTradingDisabled,
};
use execution::{
    order::{
        id::StrategyId,
        request::{OrderRequestCancel, OrderRequestOpen},
    },
    AssetIndex,
    ExchangeIndex,
    InstrumentIndex,
};
use markets::exchange::ExchangeId;
use std::marker::PhantomData;

/// Naive implementation of all strategy interfaces.
///
/// *THIS IS FOR DEMONSTRATION PURPOSES ONLY, NEVER USE FOR REAL TRADING OR IN PRODUCTION*.
///
/// This strategy:
/// - Generates no algorithmic orders (AlgoStrategy).
/// - Does not close positions (ClosePositionsStrategy).
/// - Does nothing when an exchange disconnects (OnDisconnectStrategy).
/// - Does nothing when trading state is set to disabled (OnTradingDisabled).
#[derive(Debug, Clone)]
pub struct DefaultStrategy<State> {
    pub id: StrategyId,
    phantom: PhantomData<State>,
}

impl<State> Default for DefaultStrategy<State> {
    fn default() -> Self {
        Self {
            id: StrategyId::new("default"),
            phantom: PhantomData,
        }
    }
}

impl<State, ExchangeKey, InstrumentKey> AlgoStrategy<ExchangeKey, InstrumentKey>
    for DefaultStrategy<State>
{
    type State = State;

    fn generate_algo_orders(
        &self,
        _: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    ) {
        (std::iter::empty(), std::iter::empty())
    }
}

impl<State> ClosePositionsStrategy for DefaultStrategy<State> {
    type State = State;

    fn close_positions_requests<'a>(
        &'a self,
        _state: &'a Self::State,
        _filter: &'a impl std::fmt::Debug,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>> + 'a,
    )
    where
        ExchangeIndex: 'a,
        AssetIndex: 'a,
        InstrumentIndex: 'a,
    {
        // Default implementation: no position closing orders
        (std::iter::empty(), std::iter::empty())
    }
}

impl<Clock, State, ExecutionTxs, Risk> OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
    for DefaultStrategy<State>
{
    type OnDisconnect = ();

    fn on_disconnect(_exchange: ExchangeId) -> Self::OnDisconnect {
        // Default implementation: do nothing
    }
}

impl<Clock, State, ExecutionTxs, Risk> OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
    for DefaultStrategy<State>
{
    type OnTradingDisabled = ();

    fn on_trading_disabled() -> Self::OnTradingDisabled {
        // Default implementation: do nothing
    }
}

/// Alias for DefaultStrategy for backward compatibility.
pub type NoStrategy<State> = DefaultStrategy<State>;

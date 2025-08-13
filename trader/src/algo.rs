//! Ported from former `strategy` crate: AlgoStrategy trait.
use tucano_execution::{
    order::request::{OrderRequestCancel, OrderRequestOpen},
    ExchangeIndex, InstrumentIndex,
};

pub trait AlgoStrategy<ExchangeKey = ExchangeIndex, InstrumentKey = InstrumentIndex> {
    type State;
    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>>,
    );
}

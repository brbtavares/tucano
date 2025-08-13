//! Ported from former `strategy` crate: ClosePositionsStrategy and helpers.
use execution::{
    order::{
        id::{ClientOrderId, StrategyId},
        request::{OrderRequestCancel, OrderRequestOpen, RequestOpen},
        OrderKey, OrderKind, TimeInForce,
    },
    AssetIndex, ExchangeIndex, InstrumentIndex,
};
use markets::Side;
use rust_decimal::Decimal;

pub trait ClosePositionsStrategy<
    ExchangeKey = ExchangeIndex,
    AssetKey = AssetIndex,
    InstrumentKey = InstrumentIndex,
>
{
    type State;
    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a impl std::fmt::Debug,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>> + 'a,
    )
    where
        ExchangeKey: 'a,
        AssetKey: 'a,
        InstrumentKey: 'a;
}

pub fn build_ioc_market_order_to_close_position<ExchangeKey, InstrumentKey>(
    exchange: ExchangeKey,
    instrument: InstrumentKey,
    strategy_id: StrategyId,
    side: Side,
    quantity: Decimal,
    price: Decimal,
    gen_cid: impl Fn() -> ClientOrderId,
) -> OrderRequestOpen<ExchangeKey, InstrumentKey>
where
    ExchangeKey: Clone,
    InstrumentKey: Clone,
{
    OrderRequestOpen {
        key: OrderKey {
            exchange: exchange.clone(),
            instrument: instrument.clone(),
            strategy: strategy_id,
            cid: gen_cid(),
        },
        state: RequestOpen {
            side: match side {
                Side::Buy => Side::Sell,
                Side::Sell => Side::Buy,
            },
            price,
            quantity,
            kind: OrderKind::Market,
            time_in_force: TimeInForce::ImmediateOrCancel,
        },
    }
}

pub fn close_open_positions_with_market_orders<ExchangeKey, InstrumentKey>(
    exchange: ExchangeKey,
    instrument: InstrumentKey,
    strategy_id: StrategyId,
    side: Side,
    quantity: Decimal,
    price: Decimal,
    gen_cid: impl Fn() -> ClientOrderId,
) -> Vec<OrderRequestOpen<ExchangeKey, InstrumentKey>>
where
    ExchangeKey: Clone,
    InstrumentKey: Clone,
{
    if quantity.is_zero() {
        return vec![];
    }

    vec![build_ioc_market_order_to_close_position(
        exchange,
        instrument,
        strategy_id,
        side,
        quantity,
        price,
        gen_cid,
    )]
}

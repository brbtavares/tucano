use toucan_execution::order::{
    OrderKey, OrderKind, TimeInForce,
    id::{ClientOrderId, StrategyId},
    request::{OrderRequestCancel, OrderRequestOpen, RequestOpen},
};
use toucan_instrument::{
    Side, asset::AssetIndex, exchange::ExchangeIndex, instrument::InstrumentIndex,
};
use rust_decimal::Decimal;

/// Strategy interface for generating open and cancel order requests that close open positions.
///
/// This allows full customisation of how a strategy will close a position.
///
/// Different strategies may:
/// - Use different order types (Market, Limit, etc.).
/// - Prioritise certain exchanges.
/// - Increase the position of an inversely correlated instrument in order to neutralise exposure.
/// - etc.
///
/// # Type Parameters
/// * `ExchangeKey` - Type used to identify an exchange (defaults to [`ExchangeIndex`]).
/// * `AssetKey` - Type used to identify an asset (defaults to [`AssetIndex`]).
/// * `InstrumentKey` - Type used to identify an instrument (defaults to [`InstrumentIndex`]).
pub trait ClosePositionsStrategy<
    ExchangeKey = ExchangeIndex,
    AssetKey = AssetIndex,
    InstrumentKey = InstrumentIndex,
>
{
    /// State used by the `ClosePositionsStrategy` to determine what open and cancel requests
    /// to generate.
    ///
    /// For Toucan ecosystem strategies, this is typically the full `EngineState` of the trading system.
    type State;

    /// Generate orders based on current system `State`.
    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a impl std::fmt::Debug, // Use a generic Debug bound instead of Any
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeKey, InstrumentKey>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeKey, InstrumentKey>> + 'a,
    )
    where
        ExchangeKey: 'a,
        AssetKey: 'a,
        InstrumentKey: 'a;
}

/// Build an equal but opposite `Side` `ImmediateOrCancel` `Market` order that neutralises a position.
///
/// For example, if position is LONG by 100, build a market order request to sell 100.
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

/// Generate market orders to close open positions.
/// 
/// This is a utility function that generates market orders to close all open positions
/// for a given instrument and strategy.
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

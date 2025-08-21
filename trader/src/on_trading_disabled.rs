
pub trait OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnTradingDisabled;
    fn on_trading_disabled() -> Self::OnTradingDisabled;
}

// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
pub trait OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnTradingDisabled;
    fn on_trading_disabled() -> Self::OnTradingDisabled;
}

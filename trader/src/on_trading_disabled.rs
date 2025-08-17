// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
pub trait OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnTradingDisabled;
    fn on_trading_disabled() -> Self::OnTradingDisabled;
}

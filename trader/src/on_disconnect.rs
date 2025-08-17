// Mini-Disclaimer: For educational/experimental use only; no investment advice or affiliation; no third-party compensation; Profit/ProfitDLL © Nelógica; see README & DISCLAIMER.
use tucano_markets::exchange::ExchangeId;

pub trait OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnDisconnect;
    fn on_disconnect(exchange: ExchangeId) -> Self::OnDisconnect;
}

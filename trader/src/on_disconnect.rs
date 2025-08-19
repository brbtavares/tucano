// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use tucano_instrument::exchange::ExchangeId;

pub trait OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnDisconnect;
    fn on_disconnect(exchange: ExchangeId) -> Self::OnDisconnect;
}

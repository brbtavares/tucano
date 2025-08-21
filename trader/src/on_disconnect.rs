
use toucan_instrument::exchange::ExchangeId;

pub trait OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnDisconnect;
    fn on_disconnect(exchange: ExchangeId) -> Self::OnDisconnect;
}

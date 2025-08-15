// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
use tucano_markets::exchange::ExchangeId;

pub trait OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    type OnDisconnect;
    fn on_disconnect(exchange: ExchangeId) -> Self::OnDisconnect;
}

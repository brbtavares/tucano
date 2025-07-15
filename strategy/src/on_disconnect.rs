use markets::exchange::ExchangeId;

/// Strategy interface that defines what actions should be performed after an
/// [`ExchangeId`] connection disconnects.
///
/// For example, some strategies may wish to cancel all orders, close all positions, set
/// `TradingState::Disabled`, etc.
pub trait OnDisconnectStrategy<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    /// Output of the `OnDisconnectStrategy` that is forwarded to the `AuditStream`.
    ///
    /// For example, this could include any order requests generated.
    type OnDisconnect;

    /// Perform actions after receiving an [`ExchangeId`] disconnection event.
    ///
    /// This method is called with the exchange that disconnected.
    /// Implementations should not assume access to the engine internals.
    fn on_disconnect(exchange: ExchangeId) -> Self::OnDisconnect;
}

/// Strategy interface that defines what actions should be performed after the
/// `TradingState` is set to `TradingState::Disabled`.
///
/// For example, some strategies may wish to cancel all orders, close all positions, etc.
pub trait OnTradingDisabled<Clock, State, ExecutionTxs, Risk>
where
    Self: Sized,
{
    /// Output of the `OnTradingDisabled` that is forwarded to the `AuditStream`.
    ///
    /// For example, this could include any order requests generated.
    type OnTradingDisabled;

    /// Perform actions after the `TradingState` is set to `TradingState::Disabled`.
    ///
    /// This method is called when trading is disabled.
    /// Implementations should not assume access to the engine internals.
    fn on_trading_disabled() -> Self::OnTradingDisabled;
}

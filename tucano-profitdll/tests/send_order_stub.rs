//! Teste que garante comportamento previsível do wrapper send_order quando símbolo ausente.

use rust_decimal::Decimal;
use tucano_profitdll::*;

#[tokio::test]
async fn send_order_symbol_missing_is_error_or_ok() {
    // Em plataformas não-Windows ou sem feature real_dll, construção deve funcionar (mock).
    let connector = ProfitConnector::new(None).expect("new connector");
    let _rx = connector.initialize_login("activation", "user", "pass").await.expect("init");
    let order = SendOrder::new_market_order(
        AssetIdentifier::new("PETR4".into(), "BVMF".into()),
        AccountIdentifier::new("ACC123".into(), "BRK".into()),
        OrderSide::Buy,
        Decimal::from(100),
    );
    // No ambiente mock, não há erro (função não faz nada). Em FFI real sem símbolo, erro MissingSymbol.
    let _ = connector.send_order(&order); // Aceitamos qualquer resultado (não panic) neste estágio.
}

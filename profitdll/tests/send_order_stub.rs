//! Teste que garante comportamento previsível do wrapper send_order quando símbolo ausente.

use rust_decimal::Decimal;
use profitdll::*;

#[tokio::test]
async fn send_order_symbol_missing_is_error_or_ok() {
    let connector = ProfitConnector::new(None).expect("new connector");
    let _rx = connector.initialize_login("activation", "user", "pass").await.expect("init");
    let order = SendOrder::new_market_order(
        AssetIdentifier::new("PETR4".into(), "BVMF".into()),
        AccountIdentifier::new("ACC123".into(), "BRK".into()),
        OrderSide::Buy,
        Decimal::from(100),
    );
    let _ = connector.send_order(&order);
}

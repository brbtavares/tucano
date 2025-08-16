// Mini-Disclaimer.
//! Exemplo mínimo que roda em mock ou live (se não forçado mock).
use profitdll::{
    new_backend, AccountIdentifier, AssetIdentifier, Credentials, OrderSide, SendOrder,
};
use rust_decimal::Decimal;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    if std::env::var("PROFITDLL_FORCE_MOCK").is_err() {
        // Força mock implicitamente se variáveis de credencial não existirem
        if std::env::var("PROFIT_USER").is_err() {
            std::env::set_var("PROFITDLL_FORCE_MOCK", "1");
        }
    }
    // Usa credenciais reais se disponíveis; caso contrário fallback hardcoded mock
    let creds = Credentials::from_env().unwrap_or(Credentials {
        user: "demo_user".into(),
        password: "demo_pass".into(),
        activation_key: "demo_key".into(),
    });
    let backend = new_backend()?;
    let mut rx = backend.initialize_login(&creds).await?;
    backend.subscribe_ticker("PETR4", "B").ok();
    let order = SendOrder::new_market_order(
        AssetIdentifier::new("PETR4".into(), "B".into()),
        AccountIdentifier::new("ACC123".into(), "BROKER".into()),
        OrderSide::Buy,
        Decimal::from(100),
    );
    let _ = backend.send_order(&order);
    println!("[mock_minimal] Esperando 5 eventos...");
    for i in 0..5 {
        if let Some(evt) = rx.recv().await {
            println!("evt[{i}]: {evt:?}");
        } else {
            break;
        }
    }
    backend.shutdown();
    Ok(())
}

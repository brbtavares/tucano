

//! Minimal example that runs in mock or live mode (unless mock is forced).
use rust_decimal::Decimal;
use toucan_profitdll::{
    new_backend, AccountIdentifier, AssetIdentifier, Credentials, OrderSide, SendOrder,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    if std::env::var("PROFITDLL_FORCE_MOCK").is_err() {
        // Implicitly force mock if credential variables do not exist
        if std::env::var("PROFIT_USER").is_err() {
            std::env::set_var("PROFITDLL_FORCE_MOCK", "1");
        }
    }
    // Use real credentials if available; otherwise fallback to hardcoded mock
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
    println!("[mock_minimal] Waiting for 5 events...");
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

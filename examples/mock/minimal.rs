// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Exemplo mínimo (mock) de uso do backend ProfitDLL unificado.
use profitdll::{new_backend, Credentials, AssetIdentifier, AccountIdentifier, SendOrder, OrderSide};
use rust_decimal::Decimal;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("PROFITDLL_FORCE_MOCK", "1");
    let _ = dotenvy::from_filename(".env");
    let creds = Credentials { user: "demo_user".into(), password: "demo_pass".into(), activation_key: "demo_key".into() };
    let backend = new_backend()?;
    let mut rx = backend.initialize_login(&creds).await?;
    backend.subscribe_ticker("PETR4", "BVMF")?;
    let order = SendOrder::new_market_order(
        AssetIdentifier::new("PETR4".into(), "BVMF".into()),
        AccountIdentifier::new("ACC123".into(), "BROKER".into()),
        OrderSide::Buy,
        Decimal::from(100),
    );
    let _ = backend.send_order(&order);
    println!("[mock_minimal] Aguardando eventos...");
    for i in 0..5 { if let Some(evt) = rx.recv().await { println!("evento[{i}]: {evt:?}"); } else { break; } }
    println!("[mock_minimal] Encerrado."); Ok(()) }

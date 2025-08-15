//! Exemplo mínimo (live) que usa credenciais reais via variáveis de ambiente.
use profitdll::{new_backend, Credentials};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    let creds = match Credentials::from_env() { Ok(c)=>c, Err(e)=>{ eprintln!("[live_minimal] Credenciais ausentes: {e}"); return Ok(()); } };
    let backend = new_backend()?; let mut rx = backend.initialize_login(&creds).await?;
    let ticker = std::env::var("LIVE_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("LIVE_EXCHANGE").unwrap_or_else(|_| "BVMF".into());
    backend.subscribe_ticker(&ticker, &exchange)?;
    println!("[live_minimal] Subscrito em {ticker}@{exchange}. Lendo eventos (até 10s)...");
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(10)); tokio::pin!(timeout); let mut count=0usize;
    loop { tokio::select! { evt = rx.recv() => { match evt { Some(e)=>{ println!("evt[{count}]: {e:?}"); count+=1; if count>=20 { break; }}, None=>break } } _ = &mut timeout => { println!("[live_minimal] Timeout"); break; } }}
    println!("[live_minimal] Encerrado (eventos: {count})."); Ok(()) }

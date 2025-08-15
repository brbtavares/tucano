// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! # Exemplo: Login + 1 Ticker (Live)
//!
//! Demonstra somente:
//! 1. Carregamento de variáveis de ambiente (arquivo `.env` opcional)
//! 2. Leitura de credenciais (`PROFIT_USER`, `PROFIT_PASSWORD`, opcional `PROFIT_ACTIVATION_KEY`)
//! 3. Seleção automática de backend (`mock` ou DLL real em Windows + feature `real_dll`)
//! 4. Login inicial (`initialize_login`) retornando um canal de eventos
//! 5. Subscrição de 1 ticker (`LIVE_TICKER`, default `PETR4`) em uma bolsa (`LIVE_EXCHANGE`, default `B`)
//! 6. Loop de leitura de eventos por **10 segundos** ou até **30 eventos** (o que ocorrer primeiro), contabilizando total recebido
//!
//! Não envia ordens, não persiste estado, não faz lógica de estratégia.
//!
//! ## Variáveis de Ambiente Principais
//! - `PROFIT_USER` / `PROFIT_PASSWORD` (obrigatórias)
//! - `PROFIT_ACTIVATION_KEY` (opcional)
//! - `LIVE_TICKER` (default: `PETR4`)
//! - `LIVE_EXCHANGE` (default: `B`)  // "B"=Bovespa, "F"=BM&F
//! - `PROFITDLL_FORCE_MOCK=1` força backend mock independentemente de SO/feature
//! - (Windows + feature `real_dll`): `PROFITDLL_PATH` para caminho customizado da DLL
//!
//! ## Execução
//! ```bash
//! cargo run -p tucano-examples --bin live_login_single_ticker
//! ```
//! (Adicione as variáveis de ambiente antes ou um `.env` na raiz.)
//!
//! Saída típica (mock): eventos sintéticos de market data / callbacks.
use profitdll::{new_backend, backend_kind, Credentials};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carrega variáveis do .env (ignora erro se arquivo não existir)
    let _ = dotenvy::from_filename(".env");

    // Credenciais obrigatórias via ambiente
    let creds = match Credentials::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[live_login_single_ticker] Credenciais ausentes: {e}");
            return Ok(()); // não tratamos como erro fatal para manter exemplo simples
        }
    };

    // Seleciona backend (mock forçado via PROFITDLL_FORCE_MOCK=1; caso contrário tenta real em Windows + feature)
    let backend = new_backend()?;
    let kind = backend_kind(&*backend);
    println!("[live_login_single_ticker] Backend selecionado: {kind}");
    let mut rx = backend.initialize_login(&creds).await?;

    // Parâmetros de subscrição
    let ticker = std::env::var("LIVE_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("LIVE_EXCHANGE").unwrap_or_else(|_| "B".into());
    backend.subscribe_ticker(&ticker, &exchange)?;
    const MAX_EVENTS: usize = 30;
    println!("[live_login_single_ticker] Subscrito em {ticker}@{exchange}. Lendo eventos por 10s ou até {MAX_EVENTS} eventos...");
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(10));
    tokio::pin!(timeout);
    let mut count: usize = 0;
    loop {
        tokio::select! {
            evt = rx.recv() => {
                match evt {
                    Some(e) => {
                        println!("evt[{count}]: {e:?}");
                        count += 1;
                        if count >= MAX_EVENTS {
                            println!("[live_login_single_ticker] Limite de {MAX_EVENTS} eventos atingido.");
                            break;
                        }
                    }
                    None => {
                        println!("[live_login_single_ticker] Canal de eventos fechado.");
                        break;
                    }
                }
            }
            _ = &mut timeout => {
                println!("[live_login_single_ticker] Timeout (10s). Encerrando loop.");
                break;
            }
        }
    }
    // Shutdown limpo (encerra geradores mock, se aplicável)
    backend.shutdown();
    println!("[live_login_single_ticker] Encerrado. Total de eventos recebidos: {count}.");
    Ok(())
}

// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! # Example 1: Live Login (DLL real obrigatória)
//!
//! Este exemplo exige a ProfitDLL real carregada (Windows nativo ou Wine) + feature `real_dll`.
//! Não há fallback para mock aqui.
//!
//! Passos:
//! 1. Definir variáveis de ambiente `PROFIT_USER`, `PROFIT_PASSWORD` (e opcional `PROFIT_ACTIVATION_KEY`).
//! 2. Garantir que a DLL esteja acessível: caminho padrão `profitdll/ProfitDLL.dll` ou definir `PROFITDLL_PATH`.
//! 3. Executar com `--features real_dll`.
//!
//! Variáveis adicionais:
//! - `LIVE_TICKER` (default PETR4)
//! - `LIVE_EXCHANGE` (default B)
//! - `PROFITDLL_PATH` se a DLL não estiver no diretório padrão.
//!
//! Execução:
//! ```bash
//! cargo run -p tucano-examples --bin example_1_live_login --features real_dll
//! ```
//! Erro genérico será exibido se a DLL não puder ser carregada / símbolos ausentes / credenciais inválidas.

use profitdll::{backend_kind, new_backend, Credentials};

fn fatal(msg: &str) -> ! {
    eprintln!("[example_1_live_login][ERRO] {msg}");
    std::process::exit(1)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carrega .env (ignora ausência)
    let _ = dotenvy::from_filename(".env");

    // Credenciais
    let creds =
        Credentials::from_env().unwrap_or_else(|e| fatal(&format!("Falha lendo credenciais: {e}")));

    // Instancia backend real. Se mock for retornado (ambiente sem Windows+feature), aborta.
    let backend =
        new_backend().unwrap_or_else(|e| fatal(&format!("Falha inicializando backend: {e}")));
    let kind = backend_kind(&*backend);
    if kind != "real" {
        fatal("Backend não é real (DLL). Causas possíveis: 1) não compilou com --features real_dll 2) não está em Windows/Wine 3) faltou DLL no caminho indicado.");
    }

    let mut rx = backend.initialize_login(&creds).await.unwrap_or_else(|e| {
        fatal(&format!("Falha no login inicial: {e}. Causas possíveis: credenciais inválidas, DLL incompatível, símbolo faltante."));
    });

    let ticker = std::env::var("LIVE_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("LIVE_EXCHANGE").unwrap_or_else(|_| "B".into());

    backend.subscribe_ticker(&ticker, &exchange).unwrap_or_else(|e| {
        fatal(&format!("Falha ao subscrever {ticker}@{exchange}: {e}. Possíveis causas: ticker inválido, licença insuficiente."));
    });

    println!(
        "[example_1_live_login] Conectado (backend real). Lendo eventos (10s ou 30 eventos)..."
    );
    const MAX_EVENTS: usize = 30;
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(10));
    tokio::pin!(timeout);
    let mut count = 0usize;
    loop {
        tokio::select! {
            evt = rx.recv() => {
                match evt {
                    Some(e) => {
                        println!("evt[{count}]: {e:?}");
                        count += 1;
                        if count >= MAX_EVENTS { println!("[example_1_live_login] Limite atingido."); break; }
                    }
                    None => { println!("[example_1_live_login] Canal fechado."); break; }
                }
            }
            _ = &mut timeout => { println!("[example_1_live_login] Timeout."); break; }
        }
    }
    backend.shutdown();
    println!("[example_1_live_login] Encerrado. Eventos: {count}");
    Ok(())
}

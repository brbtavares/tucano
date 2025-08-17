// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! # Example 1: Login & Subscribe (Live ou Mock)
//!
//! Demonstra login + subscribe de um ticker e impressão de eventos iniciais.
//! Funciona em dois modos:
//! - Live: Windows + `--features real_dll` + DLL acessível (backend_kind = "real_dll")
//! - Mock: Qualquer SO (backend_kind = "mock") ou forçado via `PROFITDLL_FORCE_MOCK=1`
//!
//! Variáveis de ambiente (.env opcional):
//! ```env
//! PROFIT_USER=seu_usuario            # Obrigatório (mock pode usar qualquer valor)
//! PROFIT_PASSWORD=sua_senha          # Obrigatório (mock pode usar qualquer valor)
//! PROFIT_ACTIVATION_KEY=opcional
//! PROFITDLL_PATH=C:\\caminho\\ProfitDLL.dll   # Para modo live se necessário
//! EX1_TICKER=PETR4
//! EX1_EXCHANGE=B
//! PROFITDLL_FORCE_MOCK=0            # Defina 1 para forçar mock
//! PROFITDLL_DIAG=1                  # Logs internos
//! PROFITDLL_STRICT=1                # Falha se não conseguir backend real
//! ```
//!
//! Executar (live se possível, senão mock):
//! ```bash
//! cargo run -p tucano-examples --bin example_1_live_login --features real_dll
//! ```
//! ou (garantir mock):
//! ```bash
//! PROFITDLL_FORCE_MOCK=1 cargo run -p tucano-examples --bin example_1_live_login
//! ```
//!
//! Comportamento:
//! 1. Carrega `.env`.
//! 2. Cria backend (`new_backend`).
//! 3. Login.
//! 4. Subscribe.
//! 5. Loop até timeout ou limite de eventos.
//!
//! Licença: Apache-2.0 OR MIT.

use tucano_profitdll::{backend_kind, new_backend, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_1_live_login][DEBUG] .env carregado (se existia)");

    let creds = Credentials::from_env().map_err(|e| {
        eprintln!("[example_1_live_login][ERRO] Falha lendo credenciais: {e}");
        e
    })?;
    let backend = new_backend().map_err(|e| {
        eprintln!("[example_1_live_login][ERRO] Falha criando backend: {e}");
        e
    })?;
    let kind = backend_kind(&*backend);
    eprintln!("[example_1_live_login][INFO] backend_kind={kind}");
    // Diagnóstico explícito para modo live
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        let strict = std::env::var("PROFITDLL_STRICT").unwrap_or_default() == "1";
        let force_mock = std::env::var("PROFITDLL_FORCE_MOCK").unwrap_or_default() == "1";
        if kind != "real_dll" && !force_mock {
            eprintln!("[example_1_live_login][AVISO] Não foi possível usar o modo live (real_dll). Rodando em mock.\n  Dicas:\n  - Verifique se a DLL está acessível e PROFITDLL_PATH está correto.\n  - Ative logs com PROFITDLL_DIAG=1.\n  - Use PROFITDLL_STRICT=1 para forçar erro se não conseguir live.\n  - Veja README/MANUAL.md para requisitos do modo real.");
            if strict {
                eprintln!("[example_1_live_login][FALHA] PROFITDLL_STRICT=1: abortando por não conseguir backend real.");
                return Err("Backend real_dll não disponível".into());
            }
        }
    }

    let mut rx = backend.initialize_login(&creds).await.map_err(|e| {
        eprintln!("[example_1_live_login][ERRO] Login falhou: {e}");
        e
    })?;

    let ticker = std::env::var("EX1_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("EX1_EXCHANGE").unwrap_or_else(|_| "B".into());

    if let Err(e) = backend.subscribe_ticker(&ticker, &exchange) {
        eprintln!("[example_1_live_login][ERRO] subscribe_ticker falhou: {e}");
    }

    println!(
        "[example_1_live_login] Conectado (kind={kind}). Lendo eventos (10s ou 30 eventos)..."
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

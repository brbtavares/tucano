// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! # Example 1: Live Login & Subscribe
//!
//! Demonstra login na DLL real + assinatura de um ticker e impressão de eventos iniciais.
//!
//! Requisitos:
//! - SO: Windows (exemplos live são Windows-only; stubs em outras plataformas)
//! - Feature: `--features real_dll`
//! - DLL: Disponível e acessível (definir `PROFITDLL_PATH` se não estiver no diretório padrão)
//! - Credenciais: `PROFIT_USER`, `PROFIT_PASSWORD` (opcional `PROFIT_ACTIVATION_KEY`)
//!
//! Ambiente (.env opcional na raiz):
//! ```env
//! PROFIT_USER=seu_usuario
//! PROFIT_PASSWORD=sua_senha
//! PROFITDLL_PATH=C:\\caminho\\para\\ProfitDLL.dll
//! LIVE_TICKER=PETR4
//! LIVE_EXCHANGE=B
//! PROFITDLL_DIAG=1             # opcional, logs FFI
//! PROFITDLL_STRICT=1           # opcional, falha se não conseguir backend real
//! ```
//!
//! Execução:
//! ```bash
//! cargo run -p tucano-examples --features real_dll --bin example_1_live_login
//! ```
//!
//! Fluxo Interno:
//! 1. Carrega `.env` se existir.
//! 2. Lê credenciais (`Credentials::from_env`).
//! 3. `new_backend()` -> instancia backend real (falha se mock).
//! 4. `initialize_login()` -> recebe canal de eventos.
//! 5. `subscribe_ticker()` para `LIVE_TICKER@LIVE_EXCHANGE`.
//! 6. Loop (10s ou 30 eventos) imprimindo `CallbackEvent`.
//!
//! Eventos impressos podem incluir: StateChanged, Book/Trade (se logo após login), etc.
//!
//! Erros Comuns:
//! - "Backend não é real_dll": faltou feature ou DLL inacessível.
//! - Falha no login: credenciais inválidas ou símbolos incompatíveis.
//! - MissingSymbol: versão da DLL não bate com binding atual.
//!
//! Ajustes Rápidos:
//! - Alterar ticker: exportar `LIVE_TICKER=...`.
//! - Mais eventos: aumentar `MAX_EVENTS` no código.
//! - Diagnóstico: `PROFITDLL_DIAG=1` (mostra carga e registro de callbacks).
//!
//! Saída Esperada (exemplo sintetizado):
//! ```text
//! [example_1_live_login] Conectado (backend real). Lendo eventos (10s ou 30 eventos)...
//! evt[0]: StateChanged { ... }
//! evt[1]: Trade { ... }
//! ...
//! [example_1_live_login] Timeout.
//! ```
//!
//! Segurança / Produção:
//! - Não fazer log de senha.
//! - Tratar reconexões e resubscribes (não coberto neste exemplo inicial).
//!
//! Próximas Extensões Planejadas:
//! - Envio de ordens
//! - Book nível 2 + agregação
//! - Métricas (VWAP) em streaming
//! - Gestão de reconexão automática
//!
//! Licença: Apache-2.0 OR MIT.

#[cfg(all(target_os = "windows", feature = "real_dll"))]
use profitdll::{backend_kind, new_backend, Credentials};

#[cfg(all(target_os = "windows", feature = "real_dll"))]
fn fatal(msg: &str) -> ! {
    eprintln!("[example_1_live_login][ERRO] {msg}");
    std::process::exit(1)
}

// Stub para plataformas que não são Windows ou sem feature real_dll: apenas avisa e sai.
#[cfg(not(all(target_os = "windows", feature = "real_dll")))]
fn main() {
    eprintln!("[example_1_live_login] Este exemplo live só é suportado em Windows com --features real_dll.");
}

#[cfg(all(target_os = "windows", feature = "real_dll"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Carrega .env (ignora ausência)
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_1_live_login][DEBUG] .env carregado (se existia)");

    // Credenciais
    eprintln!("[example_1_live_login][DEBUG] Lendo credenciais...");
    let creds =
        Credentials::from_env().unwrap_or_else(|e| fatal(&format!("Falha lendo credenciais: {e}")));
    eprintln!(
        "[example_1_live_login][DEBUG] Credenciais obtidas para usuário='{}'",
        creds.user
    );

    // Instancia backend real. Se mock for retornado (ambiente sem Windows+feature), aborta.
    eprintln!("[example_1_live_login][DEBUG] Criando backend...");
    let backend =
        new_backend().unwrap_or_else(|e| fatal(&format!("Falha inicializando backend: {e}")));
    eprintln!("[example_1_live_login][DEBUG] Backend criado.");
    let kind = backend_kind(&*backend);
    eprintln!("[example_1_live_login][DEBUG] backend_kind={kind}");
    if kind != "real_dll" {
        fatal("Backend não é real_dll. Certifique-se de rodar em Windows e compilar com --features real_dll e que a DLL esteja acessível.");
    }

    eprintln!("[example_1_live_login][DEBUG] Chamando initialize_login...");
    let mut rx = backend.initialize_login(&creds).await.unwrap_or_else(|e| {
        fatal(&format!("Falha no login inicial: {e}. Causas possíveis: credenciais inválidas, DLL incompatível, símbolo faltante."));
    });
    eprintln!("[example_1_live_login][DEBUG] initialize_login retornou canal.");

    let ticker = std::env::var("LIVE_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("LIVE_EXCHANGE").unwrap_or_else(|_| "B".into());

    eprintln!("[example_1_live_login][DEBUG] Subscribing {ticker}@{exchange}...");
    backend.subscribe_ticker(&ticker, &exchange).unwrap_or_else(|e| {
        fatal(&format!("Falha ao subscrever {ticker}@{exchange}: {e}. Possíveis causas: ticker inválido, licença insuficiente."));
    });
    eprintln!("[example_1_live_login][DEBUG] Subscribe ok.");

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

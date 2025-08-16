// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica.
//! # Example 2: History Trades Interval (Pull + Incremental opcional)
//!
//! Suporta backend real (Windows + feature `real_dll`) ou mock. No mock os eventos são
//! gerados sinteticamente no intervalo solicitado.
//!
//! Variáveis de ambiente:
//! - HIST_TICKER / HIST_EXCHANGE (defaults PETR4 / B)
//! - PROFIT_USER / PROFIT_PASSWORD (credenciais)
//! - PROFITDLL_FORCE_MOCK=1 força mock
//! - PROFITDLL_PATH / PROFITDLL_DIAG / PROFITDLL_STRICT conforme exemplo 1
//! - INTERVAL_START=YYYY-MM-DDTHH:MM (horário Brasil, offset -03 assumido)
//! - INTERVAL_MINUTES=duração em minutos (default 5)
//!
//! Execução:
//! ```bash
//! cargo run -p tucano-examples --bin example_2_get_history_trades --features real_dll
//! ```
//! ou mock:
//! ```bash
//! PROFITDLL_FORCE_MOCK=1 cargo run -p tucano-examples --bin example_2_get_history_trades
//! ```
//!
//! Licença: Apache-2.0 OR MIT.

use profitdll::{backend_kind, new_backend, CallbackEvent, Credentials};
use std::time::Duration;

fn brasil_ts_ms(y: i32, m: u32, d: u32, hh: u32, mm: u32) -> i64 {
    use chrono::{Duration, NaiveDate, TimeZone, Utc};
    let naive = NaiveDate::from_ymd_opt(y, m, d)
        .unwrap()
        .and_hms_opt(hh, mm, 0)
        .unwrap();
    let utc_dt = Utc.from_utc_datetime(&(naive + Duration::hours(3))); // +3h => UTC
    utc_dt.timestamp_millis()
}

fn parse_interval() -> (i64, i64) {
    // Determinístico: PETR4, B, 2025-08-16T10:35, duração 5 minutos
    let y = 2025;
    let m = 8;
    let d = 15;
    let hh = 10;
    let mm = 35;
    let minutes = 5;
    let from_ms = brasil_ts_ms(y, m, d, hh, mm);
    let to_ms = from_ms + minutes * 60_000;
    (from_ms, to_ms)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Força diagnóstico detalhado da DLL em toda execução
    std::env::set_var("PROFITDLL_DIAG", "1");
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_2_get_history_trades][DEBUG] .env carregado (se existia) e PROFITDLL_DIAG=1 ativado por padrão");

    eprintln!("[example_2_get_history_trades][DEBUG] Lendo credenciais do ambiente...");
    let creds = Credentials::from_env().map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERRO] Credenciais inválidas: {e}");
        e
    })?;
    eprintln!("[example_2_get_history_trades][DEBUG] Criando backend...");
    let backend = new_backend().map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERRO] Falha criando backend: {e}");
        e
    })?;
    let kind = backend_kind(&*backend);
    eprintln!("[example_2_get_history_trades][DEBUG] Backend criado: kind={kind}");
    // Diagnóstico explícito para modo live
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        let strict = std::env::var("PROFITDLL_STRICT").unwrap_or_default() == "1";
        let force_mock = std::env::var("PROFITDLL_FORCE_MOCK").unwrap_or_default() == "1";
        if kind != "real_dll" && !force_mock {
            eprintln!("[example_2_get_history_trades][AVISO] Não foi possível usar o modo live (real_dll). Rodando em mock.\n  Dicas:\n  - Verifique se a DLL está acessível e PROFITDLL_PATH está correto.\n  - Ative logs com PROFITDLL_DIAG=1.\n  - Use PROFITDLL_STRICT=1 para forçar erro se não conseguir live.\n  - Veja README/MANUAL.md para requisitos do modo real.");
            if strict {
                eprintln!("[example_2_get_history_trades][FALHA] PROFITDLL_STRICT=1: abortando por não conseguir backend real.");
                return Err("Backend real_dll não disponível".into());
            }
        }
    }

    eprintln!("[example_2_get_history_trades][DEBUG] Chamando initialize_login (timeout 10s)...");
    let login_result = tokio::time::timeout(
        Duration::from_secs(10),
        backend.initialize_login(&creds)
    ).await;
    let mut rx = match login_result {
        Ok(Ok(rx)) => {
            eprintln!("[example_2_get_history_trades][DEBUG] initialize_login OK, canal de eventos pronto.");
            rx
        },
        Ok(Err(e)) => {
            eprintln!("[example_2_get_history_trades][ERRO] Login falhou: {e}");
            return Err(e.into());
        },
        Err(_) => {
            eprintln!("[example_2_get_history_trades][ERRO] initialize_login travou por mais de 10s! Possível deadlock ou bloqueio na DLL. Verifique PROFIT_USER, PROFIT_PASSWORD e logs da DLL.");
            return Err("Timeout em initialize_login".into());
        }
    };

    let ticker = "PETR4".to_string();
    let exchange = "B".to_string();
    let (from_ms, to_ms) = parse_interval();

    println!("[example_2_get_history_trades] Solicitando histórico {ticker}@{exchange} entre {from_ms}..{to_ms} (ms UTC) kind={kind}");
    eprintln!("[example_2_get_history_trades][DEBUG] Chamando request_history_trades...");
    if let Err(e) = backend.request_history_trades(&ticker, &exchange, from_ms, to_ms) {
        eprintln!("[example_2_get_history_trades][ERRO] Falha solicitando histórico: {e}");
    } else {
        eprintln!("[example_2_get_history_trades][DEBUG] request_history_trades OK");
    }

    eprintln!("[example_2_get_history_trades][DEBUG] Chamando subscribe_ticker...");
    if let Err(e) = backend.subscribe_ticker(&ticker, &exchange) {
        eprintln!("[example_2_get_history_trades][AVISO] subscribe_ticker falhou: {e}");
    } else {
        eprintln!("[example_2_get_history_trades][DEBUG] subscribe_ticker OK");
    }

    println!("[example_2_get_history_trades] Aguardando eventos (timeout 15s)...");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    let mut count = 0usize;
    let mut iter = 0usize;
    while tokio::time::Instant::now() < deadline {
        iter += 1;
        if let Some(evt) = tokio::time::timeout(Duration::from_millis(500), rx.recv())
            .await
            .ok()
            .flatten()
        {
            eprintln!("[example_2_get_history_trades][DEBUG] Evento recebido no loop: {evt:?}");
            match evt {
                CallbackEvent::HistoryTrade {
                    ticker: t,
                    exchange: e,
                    price,
                    volume,
                    timestamp,
                    qty,
                    trade_id,
                    source,
                } => {
                    if t == ticker && e == exchange {
                        println!(
                            "trade#{trade_id} ts={} price={} vol={} qty={} source={:?}",
                            timestamp, price, volume, qty, source
                        );
                        count += 1;
                    }
                }
                other => {
                    println!("[example_2_get_history_trades][DEBUG] Evento não-HistoryTrade: {other:?}");
                }
            }
        } else {
            eprintln!("[example_2_get_history_trades][DEBUG] Nenhum evento recebido nesta iteração (iter={iter})");
        }
    }
    println!("[example_2_get_history_trades] Total de negócios coletados: {count}");
    if count == 0 {
        println!("[example_2_get_history_trades][AVISO] Nenhum negócio recebido.\n[example_2_get_history_trades][DICA] Possíveis causas: DLL não acionou callback, ausência de dados no intervalo, ou problema de integração. Rode com PROFITDLL_DIAG=1 para mais detalhes.");
    }

    let _ = backend.unsubscribe_ticker(&ticker, &exchange);
    backend.shutdown();
    Ok(())
}

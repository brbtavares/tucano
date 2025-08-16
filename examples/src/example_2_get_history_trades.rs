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
    let start_s = std::env::var("INTERVAL_START").unwrap_or_else(|_| "2025-08-15T10:30".into());
    let minutes: i64 = std::env::var("INTERVAL_MINUTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|m| *m > 0 && *m <= 60)
        .unwrap_or(5);
    // formato esperado YYYY-MM-DDTHH:MM
    // Espera formato YYYY-MM-DDTHH:MM
    let (date_part, time_part) = match start_s.split_once('T') {
        Some(v) => v,
        None => return (0, 0),
    };
    let mut d_it = date_part.split('-');
    let y = d_it.next().and_then(|v| v.parse().ok()).unwrap_or(2025);
    let m = d_it.next().and_then(|v| v.parse().ok()).unwrap_or(8);
    let d = d_it.next().and_then(|v| v.parse().ok()).unwrap_or(15);
    let mut t_it = time_part.split(':');
    let hh = t_it.next().and_then(|v| v.parse().ok()).unwrap_or(10);
    let mm = t_it.next().and_then(|v| v.parse().ok()).unwrap_or(30);
    let from_ms = brasil_ts_ms(y, m, d, hh, mm);
    let to_ms = from_ms + minutes * 60_000;
    (from_ms, to_ms)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_2_get_history_trades][DEBUG] .env carregado (se existia)");

    let creds = Credentials::from_env().map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERRO] Credenciais inválidas: {e}");
        e
    })?;
    let backend = new_backend().map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERRO] Falha criando backend: {e}");
        e
    })?;
    let kind = backend_kind(&*backend);
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

    let mut rx = backend.initialize_login(&creds).await.map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERRO] Login falhou: {e}");
        e
    })?;

    let ticker = std::env::var("HIST_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("HIST_EXCHANGE").unwrap_or_else(|_| "B".into());
    let (from_ms, to_ms) = parse_interval();

    println!("[example_2_get_history_trades] Solicitando histórico {ticker}@{exchange} entre {from_ms}..{to_ms} (ms UTC) kind={kind}");
    if let Err(e) = backend.request_history_trades(&ticker, &exchange, from_ms, to_ms) {
        eprintln!("[example_2_get_history_trades][ERRO] Falha solicitando histórico: {e}");
    }

    if let Err(e) = backend.subscribe_ticker(&ticker, &exchange) {
        eprintln!("[example_2_get_history_trades][AVISO] subscribe_ticker falhou: {e}");
    }

    println!("[example_2_get_history_trades] Aguardando eventos (timeout 15s)...");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    let mut count = 0usize;
    while tokio::time::Instant::now() < deadline {
        if let Some(CallbackEvent::HistoryTrade {
            ticker: t,
            exchange: e,
            price,
            volume,
            timestamp,
            qty,
            trade_id,
            source,
        }) = tokio::time::timeout(Duration::from_millis(500), rx.recv())
            .await
            .ok()
            .flatten()
        {
            if t == ticker && e == exchange {
                println!(
                    "trade#{trade_id} ts={} price={} vol={} qty={} source={:?}",
                    timestamp, price, volume, qty, source
                );
                count += 1;
            }
        }
    }
    println!("[example_2_get_history_trades] Total de negócios coletados: {count}");
    if count == 0 {
        println!("[example_2_get_history_trades][AVISO] Nenhum negócio recebido.");
    }

    let _ = backend.unsubscribe_ticker(&ticker, &exchange);
    backend.shutdown();
    Ok(())
}

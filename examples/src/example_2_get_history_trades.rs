// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! # Example 2: History Trades Interval (Pull + Optional Incremental)
//!
//! Supports real backend (Windows + feature `real_dll`) or mock. In mock mode, events are
//! generated synthetically for the requested interval.
//!
//! Environment variables:
//! - HIST_TICKER / HIST_EXCHANGE (defaults PETR4 / B)
//! - PROFIT_USER / PROFIT_PASSWORD (credentials)
//! - PROFITDLL_FORCE_MOCK=1 forces mock
//! - PROFITDLL_PATH / PROFITDLL_DIAG / PROFITDLL_STRICT as in example 1
//! - INTERVAL_START=YYYY-MM-DDTHH:MM (Brazil time, offset -03 assumed)
//! - INTERVAL_MINUTES=duration in minutes (default 5)
//!
//! Execution:
//! ```bash
//! cargo run -p tucano-examples --bin example_2_get_history_trades --features real_dll
//! ```
//! or mock:
//! ```bash
//! PROFITDLL_FORCE_MOCK=1 cargo run -p tucano-examples --bin example_2_get_history_trades
//! ```
//!
//! License: Apache-2.0 OR MIT.

use std::time::Duration;
use tucano_profitdll::{backend_kind, new_backend, CallbackEvent, Credentials};

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
    // Deterministic: PETR4, B, 2025-08-16T10:35, duration 5 minutes
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
    // Forces detailed DLL diagnostics on every run
    std::env::set_var("PROFITDLL_DIAG", "1");
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_2_get_history_trades][DEBUG] .env loaded (if present) and PROFITDLL_DIAG=1 enabled by default");

    eprintln!("[example_2_get_history_trades][DEBUG] Reading credentials from environment...");
    let creds = Credentials::from_env().map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERROR] Invalid credentials: {e}");
        e
    })?;
    eprintln!("[example_2_get_history_trades][DEBUG] Creating backend...");
    let backend = new_backend().map_err(|e| {
        eprintln!("[example_2_get_history_trades][ERROR] Failed to create backend: {e}");
        e
    })?;
    let kind = backend_kind(&*backend);
    eprintln!("[example_2_get_history_trades][DEBUG] Backend created: kind={kind}");
    // Diagnóstico explícito para modo live
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        let strict = std::env::var("PROFITDLL_STRICT").unwrap_or_default() == "1";
        let force_mock = std::env::var("PROFITDLL_FORCE_MOCK").unwrap_or_default() == "1";
        if kind != "real_dll" && !force_mock {
            eprintln!("[example_2_get_history_trades][WARNING] Could not use live mode (real_dll). Running in mock.\n  Tips:\n  - Check if the DLL is accessible and PROFITDLL_PATH is correct.\n  - Enable logs with PROFITDLL_DIAG=1.\n  - Use PROFITDLL_STRICT=1 to force error if live is not available.\n  - See README/MANUAL.md for real mode requirements.");
            if strict {
                eprintln!("[example_2_get_history_trades][FAILURE] PROFITDLL_STRICT=1: aborting because real backend is not available.");
                return Err("Backend real_dll not available".into());
            }
        }
    }

    eprintln!("[example_2_get_history_trades][DEBUG] Calling initialize_login (timeout 10s)...");
    let login_result =
        tokio::time::timeout(Duration::from_secs(10), backend.initialize_login(&creds)).await;
    let mut rx = match login_result {
        Ok(Ok(rx)) => {
            eprintln!("[example_2_get_history_trades][DEBUG] initialize_login OK, canal de eventos pronto.");
            rx
        }
        Ok(Err(e)) => {
            eprintln!("[example_2_get_history_trades][ERROR] Login failed: {e}");
            return Err(e.into());
        }
        Err(_) => {
            eprintln!("[example_2_get_history_trades][ERROR] initialize_login stuck for more than 10s! Possible deadlock or block in DLL. Check PROFIT_USER, PROFIT_PASSWORD, and DLL logs.");
            return Err("Timeout in initialize_login".into());
        }
    };

    let ticker = "PETR4".to_string();
    let exchange = "B".to_string();
    let (from_ms, to_ms) = parse_interval();

    println!("[example_2_get_history_trades] Requesting history {ticker}@{exchange} between {from_ms}..{to_ms} (ms UTC) kind={kind}");
    eprintln!("[example_2_get_history_trades][DEBUG] Calling request_history_trades...");
    if let Err(e) = backend.request_history_trades(&ticker, &exchange, from_ms, to_ms) {
        eprintln!("[example_2_get_history_trades][ERROR] Failed to request history: {e}");
    } else {
        eprintln!("[example_2_get_history_trades][DEBUG] request_history_trades OK");
    }

    eprintln!("[example_2_get_history_trades][DEBUG] Calling subscribe_ticker...");
    if let Err(e) = backend.subscribe_ticker(&ticker, &exchange) {
        eprintln!("[example_2_get_history_trades][WARNING] subscribe_ticker failed: {e}");
    } else {
        eprintln!("[example_2_get_history_trades][DEBUG] subscribe_ticker OK");
    }

    println!("[example_2_get_history_trades] Waiting for events (timeout 15s)...");
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
            eprintln!("[example_2_get_history_trades][DEBUG] Event received in loop: {evt:?}");
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
                    println!(
                        "[example_2_get_history_trades][DEBUG] Non-HistoryTrade event: {other:?}"
                    );
                }
            }
        } else {
            eprintln!("[example_2_get_history_trades][DEBUG] No event received in this iteration (iter={iter})");
        }
    }
    println!("[example_2_get_history_trades] Total trades collected: {count}");
    if count == 0 {
        println!("[example_2_get_history_trades][WARNING] No trades received.\n[example_2_get_history_trades][TIP] Possible causes: DLL did not trigger callback, no data in interval, or integration issue. Run with PROFITDLL_DIAG=1 for more details.");
    }

    let _ = backend.unsubscribe_ticker(&ticker, &exchange);
    backend.shutdown();
    Ok(())
}

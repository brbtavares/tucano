// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! # Example 1: Login & Subscribe (Live or Mock)
//!
//! Demonstrates login + subscribe to a ticker and printing initial events.
//! Works in two modes:
//! - Live: Windows + `--features real_dll` + accessible DLL (backend_kind = "real_dll")
//! - Mock: Any OS (backend_kind = "mock") or forced via `PROFITDLL_FORCE_MOCK=1`
//!
//! Environment variables (.env optional):
//! ```env
//! PROFIT_USER=your_user            # Required (mock can use any value)
//! PROFIT_PASSWORD=your_password    # Required (mock can use any value)
//! PROFIT_ACTIVATION_KEY=optional
//! PROFITDLL_PATH=C:\\path\\ProfitDLL.dll   # For live mode if needed
//! EX1_TICKER=PETR4
//! EX1_EXCHANGE=B
//! PROFITDLL_FORCE_MOCK=0            # Set 1 to force mock
//! PROFITDLL_DIAG=1                  # Internal logs
//! PROFITDLL_STRICT=1                # Fails if real backend not available
//! ```
//!
//! Run (live if possible, otherwise mock):
//! ```bash
//! cargo run -p tucano-examples --bin example_1_live_login --features real_dll
//! ```
//! or (force mock):
//! ```bash
//! PROFITDLL_FORCE_MOCK=1 cargo run -p tucano-examples --bin example_1_live_login
//! ```
//!
//! Behavior:
//! 1. Loads `.env`.
//! 2. Creates backend (`new_backend`).
//! 3. Login.
//! 4. Subscribe.
//! 5. Loop until timeout or event limit.
//!
//! License: Apache-2.0 OR MIT.

use tucano_profitdll::{backend_kind, new_backend, Credentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_1_live_login][DEBUG] .env loaded (if present)");

    let creds = Credentials::from_env().map_err(|e| {
        eprintln!("[example_1_live_login][ERROR] Failed to read credentials: {e}");
        e
    })?;
    let backend = new_backend().map_err(|e| {
        eprintln!("[example_1_live_login][ERROR] Failed to create backend: {e}");
        e
    })?;
    let kind = backend_kind(&*backend);
    eprintln!("[example_1_live_login][INFO] backend_kind={kind}");
    // Explicit diagnostic for live mode
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        let strict = std::env::var("PROFITDLL_STRICT").unwrap_or_default() == "1";
        let force_mock = std::env::var("PROFITDLL_FORCE_MOCK").unwrap_or_default() == "1";
        if kind != "real_dll" && !force_mock {
            eprintln!("[example_1_live_login][WARNING] Could not use live mode (real_dll). Running in mock.\n  Tips:\n  - Check if the DLL is accessible and PROFITDLL_PATH is correct.\n  - Enable logs with PROFITDLL_DIAG=1.\n  - Use PROFITDLL_STRICT=1 to force error if live is not available.\n  - See README/MANUAL.md for real mode requirements.");
            if strict {
                eprintln!("[example_1_live_login][FAILURE] PROFITDLL_STRICT=1: aborting because real backend is not available.");
                return Err("Backend real_dll not available".into());
            }
        }
    }

    let mut rx = backend.initialize_login(&creds).await.map_err(|e| {
        eprintln!("[example_1_live_login][ERROR] Login failed: {e}");
        e
    })?;

    let ticker = std::env::var("EX1_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("EX1_EXCHANGE").unwrap_or_else(|_| "B".into());

    if let Err(e) = backend.subscribe_ticker(&ticker, &exchange) {
        eprintln!("[example_1_live_login][ERROR] subscribe_ticker failed: {e}");
    }

    println!(
        "[example_1_live_login] Connected (kind={kind}). Reading events (10s or 30 events)..."
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
                        if count >= MAX_EVENTS { println!("[example_1_live_login] Limit reached."); break; }
                    }
                    None => { println!("[example_1_live_login] Channel closed."); break; }
                }
            }
            _ = &mut timeout => { println!("[example_1_live_login] Timeout."); break; }
        }
    }

    backend.shutdown();
    println!("[example_1_live_login] Finished. Events: {count}");
    Ok(())
}

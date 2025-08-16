// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica.
//! # Example 2: History Trades Interval (Pull + Possível Incremental)
//!
//! Demonstra solicitação de histórico de trades para um intervalo fixo e impressão dos eventos `HistoryTrade` recebidos.
//!
//! Requisitos:
//! - Windows + `--features real_dll`
//! - DLL real acessível (`PROFITDLL_PATH`)
//! - Credenciais (`PROFIT_USER`, `PROFIT_PASSWORD`)
//!
//! Intervalo padrão codificado: 15/08/2025 10:30–10:35 (horário de Brasília, offset fixo -03:00 assumido).
//! Ajustar alterando as chamadas `brasil_ts_ms` no código.
//!
//! Execução:
//! ```bash
//! cargo run -p tucano-examples --features real_dll --bin example_2_get_history_trades
//! ```
//!
//! Variáveis de ambiente relevantes:
//! - `HIST_TICKER` / `HIST_EXCHANGE` (defaults PETR4 / B)
//! - `PROFITDLL_DIAG=1` para logs (inclui trace de `GetHistoryTrades`)
//! - `PROFITDLL_STRICT=1` para falhar caso não consiga backend real
//!
//! Fluxo Interno:
//! 1. Carrega `.env`.
//! 2. Login via `initialize_login`.
//! 3. Calcula timestamps UTC do intervalo fixo.
//! 4. Invoca `request_history_trades`.
//! 5. Faz `subscribe_ticker` (defensivo — caso backend entregue incremental após pull).
//! 6. Loop 15s capturando `CallbackEvent::HistoryTrade`.
//!
//! Limitações & Observações:
//! - Callback pode estar parcial / placeholder durante desenvolvimento inicial.
//! - Sem mecanismo de progresso ou confirmação de término (futuro: progress callback).
//! - Timezone simplificado (não lida com DST antigo; atualmente Brasil sem DST).
//!
//! Extensões futuras planejadas:
//! - Barra de progresso (percentual concluído)
//! - Estatísticas agregadas (VWAP, volume total) ao final
//! - Parametrização via linha de comando
//!
//! Saída esperada (mock ou real):
//! ```text
//! trade#123 ts=2025-08-15 13:30:05 UTC price=... vol=... qty=... source=Pull
//! ...
//! [example_2_get_history_trades] Total de negócios coletados: N
//! ```
//!
//! Troubleshooting:
//! - Nenhum trade: verificar intervalo, ticker ou se callback ainda não implementado.
//! - Erro `Backend não é real_dll`: faltou feature / não está em Windows.
//! - Erro de símbolo GetHistoryTrades: versão da DLL não suporta chamada — atualizar binding ou DLL.
//!
//! Licença: Apache-2.0 OR MIT.

#[cfg(all(target_os = "windows", feature = "real_dll"))]
use profitdll::{backend_kind, new_backend, CallbackEvent, Credentials};
#[cfg(all(target_os = "windows", feature = "real_dll"))]
use std::time::Duration;

#[cfg(all(target_os = "windows", feature = "real_dll"))]
fn fatal(msg: &str) -> ! {
    eprintln!("[example_2_get_history_trades][ERRO] {msg}");
    std::process::exit(1)
}

// Converte timestamp (UTC) de data/hora local Brasil (America/Sao_Paulo) assumindo offset -03:00 estático.
// OBS: Para simplicidade evitamos dependência de time zone; risco: horário de verão (inativo atualmente).
#[cfg(all(target_os = "windows", feature = "real_dll"))]
fn brasil_ts_ms(y: i32, m: u32, d: u32, hh: u32, mm: u32) -> i64 {
    use chrono::{Duration, NaiveDate, TimeZone, Utc};
    let naive = NaiveDate::from_ymd_opt(y, m, d)
        .unwrap()
        .and_hms_opt(hh, mm, 0)
        .unwrap();
    // Adiciona 3h para obter UTC (assumindo BRT sem DST)
    let utc_dt = Utc.from_utc_datetime(&(naive + Duration::hours(3)));
    utc_dt.timestamp_millis()
}

#[cfg(not(all(target_os = "windows", feature = "real_dll")))]
fn main() {
    eprintln!("[example_2_get_history_trades] Este exemplo live só é suportado em Windows com --features real_dll. Use FORCE_RUN_MOCK=1 para testar lógica local (não-representa dados reais)." );
}

#[cfg(all(target_os = "windows", feature = "real_dll"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::from_filename(".env");
    eprintln!("[example_2_get_history_trades][DEBUG] .env carregado (se existia)");

    let creds =
        Credentials::from_env().unwrap_or_else(|e| fatal(&format!("Credenciais inválidas: {e}")));
    let backend = new_backend().unwrap_or_else(|e| fatal(&format!("Falha criando backend: {e}")));
    if backend_kind(&*backend) != "real_dll" {
        fatal("Backend não é real_dll (este exemplo requer Windows + DLL).")
    }

    let mut rx = backend
        .initialize_login(&creds)
        .await
        .unwrap_or_else(|e| fatal(&format!("Login falhou: {e}")));

    let ticker = std::env::var("HIST_TICKER").unwrap_or_else(|_| "PETR4".into());
    let exchange = std::env::var("HIST_EXCHANGE").unwrap_or_else(|_| "B".into());

    // Intervalo 15/08/2025 10:30 a 10:35 (Brasília)
    let from_ms = brasil_ts_ms(2025, 8, 15, 10, 30);
    let to_ms = brasil_ts_ms(2025, 8, 15, 10, 35);
    println!("[example_2_get_history_trades] Solicitando histórico {ticker}@{exchange} entre {from_ms}..{to_ms} (ms UTC)");
    if std::env::var("PROFITDLL_DIAG").ok().as_deref() == Some("1") {
        eprintln!(
            "[example_2_get_history_trades][DIAG] backend_kind={} strict_live={} force_mock={}",
            backend_kind(&*backend),
            std::env::var("PROFITDLL_STRICT").unwrap_or_default(),
            std::env::var("PROFITDLL_FORCE_MOCK").unwrap_or_default()
        );
    }

    // Solicita histórico (pull). Se mock, usamos request_history_trades que gera eventos sintéticos.
    if let Err(e) = backend.request_history_trades(&ticker, &exchange, from_ms, to_ms) {
        eprintln!("[example_2_get_history_trades][ERRO] Falha solicitando histórico: {e}");
    }

    // Tentativa de subscribe para garantir recebimento incremental após histórico (se a DLL usar stream para entregar).
    if backend_kind(&*backend) == "real_dll" {
        if let Err(e) = backend.subscribe_ticker(&ticker, &exchange) {
            eprintln!("[example_2_get_history_trades][AVISO] subscribe_ticker falhou (pode não ser necessário): {e}");
        } else {
            eprintln!("[example_2_get_history_trades][DIAG] subscribe_ticker enviado.");
        }
    }

    println!("[example_2_get_history_trades] Aguardando eventos (timeout 15s)...");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    let mut count = 0usize;
    while tokio::time::Instant::now() < deadline {
        if let Some(evt) = tokio::time::timeout(Duration::from_millis(500), rx.recv())
            .await
            .ok()
            .flatten()
        {
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
                CallbackEvent::AdjustHistory { .. } => { /* ignore for this example */ }
                CallbackEvent::TheoreticalPrice { .. } => { /* ignore */ }
                _ => {}
            }
        }
    }
    println!("[example_2_get_history_trades] Total de negócios coletados: {count}");
    if count == 0 {
        println!("[example_2_get_history_trades][AVISO] Nenhum negócio recebido (callback possivelmente não implementado ou intervalo sem trades).");
    }
    if backend_kind(&*backend) == "real_dll" {
        // Desinscrever para limpeza
        let _ = backend.unsubscribe_ticker(&ticker, &exchange);
    }
    backend.shutdown();
    Ok(())
}

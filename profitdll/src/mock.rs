// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Implementação mock e tipos compartilhados para interface ProfitDLL.
//!
//! Esta camada simula o comportamento da DLL Profit para testes, exemplos e ambientes sem acesso à DLL real.
//! Todos os eventos, tipos e enums seguem a especificação oficial descrita no [MANUAL.md](../MANUAL.md).

use crate::error::*;
use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

/// Enum de eventos assíncronos emitidos pela DLL Profit (**CallbackEvent**).
///
/// Cada variante representa um tipo de callback/documentação oficial da DLL.
/// Consulte o [MANUAL.md](../MANUAL.md#eventos-e-callbacks) para detalhes completos.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum CallbackEvent {
    /// Mudança de estado de conexão (**StateChanged**)
    StateChanged {
        connection_type: ConnectionState,
        result: i32,
    },
    /// Progresso de inscrição em ticker (**ProgressChanged**)
    ProgressChanged {
        ticker: String,
        exchange: String,
        feed_type: i32,
        progress: i32,
    },
    /// Novo negócio/trade (**NewTrade**)
    NewTrade {
        ticker: String,
        exchange: String,
        price: Decimal,
        volume: Decimal,
        timestamp: DateTime<Utc>,
        buy_agent: String,
        sell_agent: String,
        trade_id: i64,
        is_edit: bool,
    },
    /// Resumo diário (**DailySummary**)
    DailySummary {
        ticker: String,
        exchange: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        adjustment: Decimal,
        max_limit: Decimal,
        min_limit: Decimal,
        trades_buyer: Decimal,
        trades_seller: Decimal,
    },
    /// Oferta de livro de preços (**PriceBookOffer**)
    PriceBookOffer {
        ticker: String,
        exchange: String,
        action: BookAction,
        price: Decimal,
        position: i32,
    },
    /// Oferta de livro de ofertas de compra (**OfferBookBid**)
    OfferBookBid {
        ticker: String,
        exchange: String,
        action: BookAction,
        price: Decimal,
        position: i32,
    },
    /// Mudança de conta (**AccountChanged**)
    AccountChanged {
        account_id: String,
        account_holder: String,
        broker_name: String,
        broker_id: i32,
    },
    /// Ticker inválido (**InvalidTicker**)
    InvalidTicker {
        ticker: String,
        exchange: String,
        feed_type: i32,
    },
    /// Ordem atualizada (**OrderUpdated**)
    OrderUpdated { order_id: i64 },
    /// Snapshot completo de ordem (**OrderSnapshot**)
    OrderSnapshot {
        order_id: i64,
        account_id: String,
        ticker: String,
        exchange: String,
        side: OrderSide,
        order_type: OrderType,
        status: OrderStatus,
        quantity: Decimal,
        filled: Decimal,
        price: Option<Decimal>,
        stop_price: Option<Decimal>,
        validity: OrderValidity,
        text: Option<String>,
    },
    /// Trade histórico (**HistoryTrade**): pull ou callback incremental.
    HistoryTrade {
        ticker: String,
        exchange: String,
        price: Decimal,
        volume: Decimal,
        timestamp: DateTime<Utc>,
        qty: i32,
        trade_id: i64,
        source: HistoryTradeSource,
    },
    /// Ajustes corporativos (**AdjustHistory**): dividendos, splits, etc.
    AdjustHistory {
        ticker: String,
        exchange: String,
        value: Decimal,
        adjust_type: String,
        observation: String,
        date_adjust: String,
        date_deliberation: String,
        date_payment: String,
        flags: i32,
        multiplier: Decimal,
    },
    /// Preço teórico (**TheoreticalPrice**): leilão/derivativos.
    TheoreticalPrice {
        ticker: String,
        exchange: String,
        theoretical_price: Decimal,
        quantity: i64,
    },
}

/// Origem do trade histórico (**HistoryTradeSource**).
///
/// Indica a fonte do evento **HistoryTrade** conforme a DLL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryTradeSource {
    /// Proveniente de stream em tempo real mas classificado como histórico (ex: backfill inicial curto)
    RealtimeBackfill,
    /// Resultado de chamada explícita a **GetHistoryTrades** (pull)
    Pull,
    /// Recebido via callback incremental dedicado (caso DLL use canal separado)
    IncrementalCallback,
}

/// Estado de conexão (**ConnectionState**).
///
/// Usado em eventos **StateChanged**.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    Login = 0,
    Routing = 1,
    MarketData = 2,
    MarketLogin = 3,
}

/// Ação de livro de ofertas (**BookAction**).
///
/// Usado em eventos **PriceBookOffer** e **OfferBookBid**.
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum BookAction {
    New = 0,
    Edit = 1,
    Delete = 2,
}

#[derive(Debug)]
struct GeneratorEntry {
    handle: JoinHandle<()>,
    stop: Arc<AtomicBool>,
    events: Arc<AtomicU64>, // número de eventos NewTrade gerados
}

pub struct ProfitConnector {
    _connected: bool,
    sender: Mutex<Option<UnboundedSender<CallbackEvent>>>,
    // Map chave: "ticker|exchange" -> gerador
    generators: Mutex<HashMap<String, GeneratorEntry>>,
    metrics: Mutex<HashMap<String, Arc<AtomicU64>>>,
}
impl ProfitConnector {
    pub fn new(_dll_path: Option<&str>) -> Result<Self, ProfitError> {
        Ok(Self {
            _connected: false,
            sender: Mutex::new(None),
            generators: Mutex::new(HashMap::new()),
            metrics: Mutex::new(HashMap::new()),
        })
    }
    pub async fn initialize_login(
        &self,
        _activation_key: &str,
        _user: &str,
        _password: &str,
    ) -> Result<UnboundedReceiver<CallbackEvent>, ProfitError> {
        let (tx, rx) = unbounded_channel();
        *self.sender.lock().unwrap() = Some(tx);
        Ok(rx)
    }
    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        if let Some(tx) = self.sender.lock().unwrap().as_ref() {
            let _ = tx.send(CallbackEvent::ProgressChanged {
                ticker: ticker.to_string(),
                exchange: exchange.to_string(),
                feed_type: 0,
                progress: 100,
            });
            let _ = tx.send(CallbackEvent::PriceBookOffer {
                ticker: ticker.to_string(),
                exchange: exchange.to_string(),
                action: BookAction::New,
                price: Decimal::from(10),
                position: 0,
            });
            // Inicia gerador sintético periódico para este ticker
            let ticker_s = ticker.to_string();
            let exchange_s = exchange.to_string();
            let tx_clone = tx.clone();
            let stop_flag = Arc::new(AtomicBool::new(false));
            let stop_clone = Arc::clone(&stop_flag);
            let events_counter = Arc::new(AtomicU64::new(0));
            let events_clone = Arc::clone(&events_counter);
            let handle = tokio::spawn(async move {
                let mut seq: i64 = 1;
                // Intervalo configurável (default 500ms)
                let interval_ms: u64 = env::var("MOCK_INTERVAL_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .filter(|v| *v > 0 && *v <= 60_000)
                    .unwrap_or(500);
                while !stop_clone.load(Ordering::Relaxed) {
                    tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    let now = Utc::now();
                    // Preço oscilando deterministicamente
                    let base =
                        Decimal::from(10) + Decimal::from(seq % 20 - 10) / Decimal::from(100);
                    let _ = tx_clone.send(CallbackEvent::NewTrade {
                        ticker: ticker_s.clone(),
                        exchange: exchange_s.clone(),
                        price: base,
                        // seq é i64; o cast anterior era redundante e gerava lint (unnecessary_cast)
                        volume: Decimal::from(100 + (seq % 5) * 10),
                        timestamp: now,
                        buy_agent: "MOCKB".into(),
                        sell_agent: "MOCKS".into(),
                        trade_id: seq,
                        is_edit: false,
                    });
                    events_clone.fetch_add(1, Ordering::Relaxed);
                    // Livro: alterna ação edit/delete de posição 0 a cada 5 trades
                    if seq % 5 == 0 {
                        let action = if (seq / 5) % 2 == 0 {
                            BookAction::Edit
                        } else {
                            BookAction::Delete
                        };
                        let _ = tx_clone.send(CallbackEvent::PriceBookOffer {
                            ticker: ticker_s.clone(),
                            exchange: exchange_s.clone(),
                            action,
                            price: base,
                            position: 0,
                        });
                    }
                    // Emite eventos históricos / teóricos / ajuste a cada 200 trades como placeholders
                    if seq % 200 == 0 {
                        let _ = tx_clone.send(CallbackEvent::HistoryTrade {
                            ticker: ticker_s.clone(),
                            exchange: exchange_s.clone(),
                            price: base,
                            volume: Decimal::from(250),
                            timestamp: now,
                            qty: 25,
                            trade_id: seq,
                            source: HistoryTradeSource::RealtimeBackfill,
                        });
                        let _ = tx_clone.send(CallbackEvent::TheoreticalPrice {
                            ticker: ticker_s.clone(),
                            exchange: exchange_s.clone(),
                            theoretical_price: base + Decimal::from(1),
                            quantity: 1000,
                        });
                        let _ = tx_clone.send(CallbackEvent::AdjustHistory {
                            ticker: ticker_s.clone(),
                            exchange: exchange_s.clone(),
                            value: Decimal::from(1234) / Decimal::from(100),
                            adjust_type: "DIV".into(),
                            observation: "mock-adjust".into(),
                            date_adjust: now.format("%Y-%m-%d").to_string(),
                            date_deliberation: now.format("%Y-%m-%d").to_string(),
                            date_payment: now.format("%Y-%m-%d").to_string(),
                            flags: 0,
                            multiplier: Decimal::from(1),
                        });
                    }
                    seq += 1;
                }
            });
            let key = format!("{ticker}|{exchange}");
            self.metrics
                .lock()
                .unwrap()
                .insert(key.clone(), Arc::clone(&events_counter));
            self.generators.lock().unwrap().insert(
                key,
                GeneratorEntry {
                    handle,
                    stop: stop_flag,
                    events: events_counter,
                },
            );
        }
        Ok(())
    }
    pub fn unsubscribe_ticker(&self, _ticker: &str, _exchange: &str) -> Result<(), ProfitError> {
        let key = format!("{_ticker}|{_exchange}");
        if let Some(entry) = self.generators.lock().unwrap().remove(&key) {
            entry.stop.store(true, Ordering::Relaxed);
            let handle = entry.handle;
            // Aguardar término com timeout (best-effort)
            tokio::spawn(async move {
                if tokio::time::timeout(Duration::from_millis(250), handle)
                    .await
                    .is_err()
                {
                    eprintln!("[mock] Timeout aguardando join de gerador; abortando");
                }
            });
            // Remove métrica associada
            self.metrics.lock().unwrap().remove(&key);
        }
        Ok(())
    }
    /// Sinaliza parada e aguarda (best-effort) término das tasks.
    pub fn shutdown_all(&self) {
        let mut map = self.generators.lock().unwrap();
        if map.is_empty() {
            return;
        }
        let entries: Vec<_> = map.drain().collect();
        // Limpa métricas relacionadas
        self.metrics
            .lock()
            .unwrap()
            .retain(|k, _| !entries.iter().any(|(ek, _)| ek == k));
        // Loga contagem de eventos gerados por ticker antes de sinalizar parada
        for (k, entry) in &entries {
            let generated = entry.events.load(Ordering::Relaxed);
            eprintln!("[mock] shutdown_all: {k} total_new_trades={generated}");
        }
        // Sinaliza parada
        for (_k, entry) in &entries {
            entry.stop.store(true, Ordering::Relaxed);
        }
        // Agenda joins assíncronos (não bloqueia)
        for (_k, entry) in entries {
            let handle = entry.handle;
            tokio::spawn(async move {
                if tokio::time::timeout(Duration::from_millis(250), handle)
                    .await
                    .is_err()
                {
                    eprintln!(
                        "[mock] Timeout aguardando join de gerador (shutdown_all); abortando"
                    );
                }
            });
        }
    }
    pub fn send_order(&self, _order: &SendOrder) -> Result<(), ProfitError> {
        if let Some(tx) = self.sender.lock().unwrap().as_ref() {
            let _ = tx.send(CallbackEvent::OrderUpdated { order_id: 1 });
        }
        Ok(())
    }
    pub fn cancel_order(&self, _order_id: i64) -> Result<(), ProfitError> {
        Ok(())
    }
    pub fn change_order(
        &self,
        _order_id: i64,
        _new_price: Option<Decimal>,
        _new_qty: Option<Decimal>,
    ) -> Result<(), ProfitError> {
        Ok(())
    }

    /// Mock: gera eventos HistoryTrade sintéticos no intervalo [from_ms, to_ms) a cada step_ms.
    pub fn get_history_trades(
        &self,
        ticker: &str,
        exchange: &str,
        from_ms: i64,
        to_ms: i64,
        step_ms: i64,
    ) -> Result<(), ProfitError> {
        if from_ms >= to_ms || step_ms <= 0 {
            return Ok(()); // nada a fazer
        }
        if let Some(tx) = self.sender.lock().unwrap().as_ref() {
            let mut t = from_ms;
            let mut seq: i64 = 1;
            while t < to_ms {
                let price = Decimal::from(10) + Decimal::from(seq % 20 - 10) / Decimal::from(100);
                let volume = Decimal::from(100 + (seq % 5) * 10);
                let ts = chrono::Utc
                    .timestamp_millis_opt(t)
                    .single()
                    .unwrap_or_else(chrono::Utc::now);
                let _ = tx.send(CallbackEvent::HistoryTrade {
                    ticker: ticker.to_string(),
                    exchange: exchange.to_string(),
                    price,
                    volume,
                    timestamp: ts,
                    qty: 1 + (seq % 50) as i32,
                    trade_id: seq,
                    source: HistoryTradeSource::Pull,
                });
                t += step_ms;
                seq += 1;
            }
        }
        Ok(())
    }
}

impl Drop for ProfitConnector {
    fn drop(&mut self) {
        // Best-effort: sinaliza parada para qualquer gerador remanescente
        let mut map = self.generators.lock().unwrap();
        for (_k, entry) in map.drain() {
            entry.stop.store(true, Ordering::Relaxed);
            entry.handle.abort(); // abort no drop para encerrar rápido
        }
    }
}

impl ProfitConnector {
    /// Retorna snapshot das métricas de eventos por ticker (apenas mock).
    pub fn mock_metrics(&self) -> HashMap<String, u64> {
        self.metrics
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIdentifier {
    pub ticker: String,
    pub exchange: String,
}
impl AssetIdentifier {
    pub fn new(ticker: String, exchange: String) -> Self {
        Self { ticker, exchange }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountIdentifier {
    pub account_id: String,
    pub broker: String,
}
impl AccountIdentifier {
    pub fn new(account_id: String, broker: String) -> Self {
        Self { account_id, broker }
    }
}

/// Lado da ordem (**OrderSide**).
///
/// Usado em **SendOrder** e eventos de ordem.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

/// Estrutura de envio de ordem (**SendOrder**).
///
/// Parâmetros conforme função **SendOrder** da DLL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOrder {
    pub asset: AssetIdentifier,
    pub account: AccountIdentifier,
    pub side: OrderSide,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub validity: OrderValidity,
}
impl SendOrder {
    pub fn new_market_order(
        asset: AssetIdentifier,
        account: AccountIdentifier,
        side: OrderSide,
        quantity: Decimal,
    ) -> Self {
        Self {
            asset,
            account,
            side,
            quantity,
            price: None,
            validity: OrderValidity::ImmediateOrCancel,
        }
    }
    pub fn new_limit_order(
        asset: AssetIdentifier,
        account: AccountIdentifier,
        side: OrderSide,
        quantity: Decimal,
        price: Decimal,
    ) -> Self {
        Self {
            asset,
            account,
            side,
            quantity,
            price: Some(price),
            validity: OrderValidity::Day,
        }
    }
}

/// Validade da ordem (**OrderValidity**).
///
/// Usado em **SendOrder** e eventos de ordem.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderValidity {
    Day,
    GoodTillCanceled,
    ImmediateOrCancel,
    FillOrKill,
}

/// Tipo de ordem (**OrderType**).
///
/// Usado em eventos de ordem.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market = 1,
    Limit = 2,
    StopLimit = 4,
}

/// Status da ordem (**OrderStatus**).
///
/// Usado em eventos de ordem e callbacks.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    New = 0,
    PartiallyFilled = 1,
    Filled = 2,
    DoneForDay = 3,
    Canceled = 4,
    Replaced = 5,
    PendingCancel = 6,
    Stopped = 7,
    Rejected = 8,
    Suspended = 9,
    PendingNew = 10,
    Calculated = 11,
    Expired = 12,
    AcceptedForBidding = 13,
    PendingReplace = 14,
    PartiallyFilledCanceled = 15,
    Received = 16,
    PartiallyFilledExpired = 17,
    PartiallyFilledRejected = 18,
    Unknown = 200,
    HadesCreated = 201,
    BrokerSent = 202,
    ClientCreated = 203,
    OrderNotCreated = 204,
    CanceledByAdmin = 205,
    DelayFixGateway = 206,
    ScheduledOrder = 207,
}

impl OrderStatus {
    pub fn from_i32(v: i32) -> Self {
        use OrderStatus::*;
        match v {
            0 => New,
            1 => PartiallyFilled,
            2 => Filled,
            3 => DoneForDay,
            4 => Canceled,
            5 => Replaced,
            6 => PendingCancel,
            7 => Stopped,
            8 => Rejected,
            9 => Suspended,
            10 => PendingNew,
            11 => Calculated,
            12 => Expired,
            13 => AcceptedForBidding,
            14 => PendingReplace,
            15 => PartiallyFilledCanceled,
            16 => Received,
            17 => PartiallyFilledExpired,
            18 => PartiallyFilledRejected,
            200 => Unknown,
            201 => HadesCreated,
            202 => BrokerSent,
            203 => ClientCreated,
            204 => OrderNotCreated,
            205 => CanceledByAdmin,
            206 => DelayFixGateway,
            207 => ScheduledOrder,
            _ => Unknown,
        }
    }
}

// Usa ProfitError unificado de crate::error

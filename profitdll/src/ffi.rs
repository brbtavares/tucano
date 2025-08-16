// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Implementação FFI real (Windows + feature `real_dll`) para a DLL Profit.
//!
//! Esta camada realiza:
//! - Carregamento dinâmico da DLL (**ProfitDLL.dll**)
//! - Registro de callbacks oficiais (ver [MANUAL.md](../MANUAL.md#eventos-e-callbacks))
//! - Login inicial (**InitializeLogin**)
//! - Canal assíncrono para eventos ([`CallbackEvent`])
//! - Envio, cancelamento e alteração de ordens conforme interface oficial
//!
//! OBS: Callbacks adicionais podem ser expandidos conforme necessidade, seguindo a estrutura dos trampolines.
#![allow(non_camel_case_types)]

use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use std::sync::Arc;
use std::{
    ffi::{c_char, c_int, c_void},
    ptr,
    sync::Mutex,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

#[cfg(all(target_os = "windows", feature = "real_dll"))]
use crate::ForeignBuffer;
use crate::{
    error::*, BookAction, CallbackEvent, ConnectionState, OrderSide, OrderStatus, OrderType,
    OrderValidity, SendOrder,
};
use chrono::{TimeZone, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal; // para to_f64

/// Tipo de retorno padrão da DLL (**NResult**).
pub type NResult = i32; // re-export local para facilitar (mantém igual)

// ---- Assinaturas brutas (subset) ----

type StateCallbackRaw =
    unsafe extern "system" fn(conn_type: c_int, result: NResult, ctx: *mut c_void);
type TradeCallbackRaw = unsafe extern "system" fn(
    icker: *const c_char,
    exchange: *const c_char,
    price: f64,
    volume: f64,
    timestamp_ms: i64,
    buy_agent: *const c_char,
    sell_agent: *const c_char,
    trade_id: i64,
    is_edit: c_int,
    ctx: *mut c_void,
);

type BookCallbackRaw = unsafe extern "system" fn(
    side: c_int,
    ticker: *const c_char,
    exchange: *const c_char,
    action: c_int,
    price: f64,
    position: c_int,
    ctx: *mut c_void,
);

type DailySummaryCallbackRaw = unsafe extern "system" fn(
    icker: *const c_char,
    exchange: *const c_char,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    adjustment: f64,
    max_limit: f64,
    min_limit: f64,
    trades_buyer: f64,
    trades_seller: f64,
    ctx: *mut c_void,
);

type AccountCallbackRaw = unsafe extern "system" fn(
    account_id: *const c_char,
    account_holder: *const c_char,
    broker_name: *const c_char,
    broker_id: c_int,
    ctx: *mut c_void,
);

type InvalidTickerCallbackRaw = unsafe extern "system" fn(
    ticker: *const c_char,
    exchange: *const c_char,
    feed_type: c_int,
    ctx: *mut c_void,
);

type OrderCallbackRaw = unsafe extern "system" fn(order_id: i64, ctx: *mut c_void);
type GetOrderDetailsFn =
    unsafe extern "system" fn(order_id: i64, out: *mut COrderDetails) -> NResult;

/// Estrutura C para trade (**ProfitTrade**).
///
/// Usada em callbacks e funções da DLL.
#[repr(C)]
pub struct ProfitTrade {
    pub ticker: *const c_char,
    pub exchange: *const c_char,
    pub price: f64,
    pub volume: f64,
    pub timestamp_ms: i64,
    pub buy_agent: *const c_char,
    pub sell_agent: *const c_char,
    pub trade_id: i64,
    pub is_edit: c_int,
}

/// Estrutura C para atualização de livro de ofertas (**ProfitBookUpdate**).
#[repr(C)]
pub struct ProfitBookUpdate {
    pub side: c_int,
    pub ticker: *const c_char,
    pub exchange: *const c_char,
    pub action: c_int,
    pub price: f64,
    pub position: c_int,
}

/// Estrutura C para resumo diário (**ProfitDailySummary**).
#[repr(C)]
pub struct ProfitDailySummary {
    pub ticker: *const c_char,
    pub exchange: *const c_char,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub adjustment: f64,
    pub max_limit: f64,
    pub min_limit: f64,
    pub trades_buyer: f64,
    pub trades_seller: f64,
}

// SUPOSIÇÃO (layout a confirmar via header real): campos char* + doubles + i32 flags
/// Estrutura C para ajuste corporativo (**ProfitAdjustHistoryV2**).
#[repr(C)]
struct ProfitAdjustHistoryV2 {
    ticker: *const c_char,
    exchange: *const c_char,
    value: f64,
    adjust_type: *const c_char,
    observation: *const c_char,
    date_adjust: *const c_char,
    date_deliberation: *const c_char,
    date_payment: *const c_char,
    flags: c_int,
    multiplier: f64,
}

type TradeCallbackRawV2 = unsafe extern "system" fn(trade: *const ProfitTrade, ctx: *mut c_void);
type BookCallbackRawV2 =
    unsafe extern "system" fn(update: *const ProfitBookUpdate, ctx: *mut c_void);
type DailySummaryCallbackRawV2 =
    unsafe extern "system" fn(summary: *const ProfitDailySummary, ctx: *mut c_void);

// Callback dedicado para histórico incremental (placeholder)
type HistoryTradeCallbackRaw =
    unsafe extern "system" fn(trade: *const ProfitTrade, ctx: *mut c_void);
#[repr(C)]
struct CSendOrder {
    icker: *const c_char,
    exchange: *const c_char,
    side: c_int,
    quantity: f64,
    price: f64,
    validity: c_int,
}

#[repr(C)]
struct CCancelOrder {
    order_id: i64,
}
#[repr(C)]
struct CChangeOrder {
    order_id: i64,
    new_price: f64,
    new_quantity: f64,
}

#[repr(C)]
struct COrderDetails {
    order_id: i64,
    account_id: *const c_char,
    ticker: *const c_char,
    exchange: *const c_char,
    side: c_int,
    order_type: c_int,
    status: c_int,
    quantity: f64,
    filled: f64,
    price: f64,
    stop_price: f64,
    validity: c_int,
    text: *const c_char,
}

#[allow(non_snake_case)]
struct ProfitRaw<'lib> {
    // Alguns builds da DLL podem não expor Initialize/Finalize explicitamente: tratamos como opcionais.
    Initialize: Option<Symbol<'lib, unsafe extern "system" fn() -> NResult>>,
    _Finalize: Option<Symbol<'lib, unsafe extern "system" fn() -> NResult>>,
    SetStateCallback:
        Symbol<'lib, unsafe extern "system" fn(StateCallbackRaw, *mut c_void) -> NResult>,
    Login: Symbol<
        'lib,
        unsafe extern "system" fn(user: *const c_char, pass: *const c_char) -> NResult,
    >,
    SubscribeTicker: Symbol<
        'lib,
        unsafe extern "system" fn(ticker: *const c_char, exch: *const c_char) -> NResult,
    >,
    UnsubscribeTicker: Symbol<
        'lib,
        unsafe extern "system" fn(ticker: *const c_char, exch: *const c_char) -> NResult,
    >,
    SetTradeCallback:
        Option<Symbol<'lib, unsafe extern "system" fn(TradeCallbackRaw, *mut c_void) -> NResult>>,
    SetBookCallback:
        Option<Symbol<'lib, unsafe extern "system" fn(BookCallbackRaw, *mut c_void) -> NResult>>,
    SetDailySummaryCallback: Option<
        Symbol<'lib, unsafe extern "system" fn(DailySummaryCallbackRaw, *mut c_void) -> NResult>,
    >,
    SetAccountCallback:
        Option<Symbol<'lib, unsafe extern "system" fn(AccountCallbackRaw, *mut c_void) -> NResult>>,
    SetInvalidTickerCallback: Option<
        Symbol<'lib, unsafe extern "system" fn(InvalidTickerCallbackRaw, *mut c_void) -> NResult>,
    >,
    SetOrderCallback:
        Option<Symbol<'lib, unsafe extern "system" fn(OrderCallbackRaw, *mut c_void) -> NResult>>,
    SetTradeCallbackV2:
        Option<Symbol<'lib, unsafe extern "system" fn(TradeCallbackRawV2, *mut c_void) -> NResult>>,
    SetBookCallbackV2:
        Option<Symbol<'lib, unsafe extern "system" fn(BookCallbackRawV2, *mut c_void) -> NResult>>,
    SetDailySummaryCallbackV2: Option<
        Symbol<'lib, unsafe extern "system" fn(DailySummaryCallbackRawV2, *mut c_void) -> NResult>,
    >,
    SendOrder: Option<Symbol<'lib, unsafe extern "system" fn(*const CSendOrder) -> NResult>>,
    SendCancelOrderV2:
        Option<Symbol<'lib, unsafe extern "system" fn(*const CCancelOrder) -> NResult>>,
    SendChangeOrderV2:
        Option<Symbol<'lib, unsafe extern "system" fn(*const CChangeOrder) -> NResult>>,
    GetOrderDetails: Option<Symbol<'lib, GetOrderDetailsFn>>,
    SetHistoryTradeCallback: Option<
        Symbol<'lib, unsafe extern "system" fn(HistoryTradeCallbackRaw, *mut c_void) -> NResult>,
    >,
    // --- Novos símbolos (histórico / ajustes / teórico / infra) ---
    GetHistoryTrades: Option<
        Symbol<'lib, unsafe extern "system" fn(*const c_char, *const c_char, i64, i64) -> NResult>,
    >, // (ticker, exchange, from_ms, to_ms)
    SetAdjustHistoryCallbackV2: Option<
        Symbol<
            'lib,
            unsafe extern "system" fn(
                unsafe extern "system" fn(*const c_void, *mut c_void),
                *mut c_void,
            ) -> NResult,
        >,
    >,
    #[allow(clippy::type_complexity)]
    SetTheoreticalPriceCallback: Option<
        Symbol<
            'lib,
            unsafe extern "system" fn(
                unsafe extern "system" fn(
                    *const c_char,
                    *const c_char,
                    f64,
                    f64,
                    f64,
                    f64,
                    *mut c_void,
                ),
                *mut c_void,
            ) -> NResult,
        >,
    >,
    FreePointer: Option<Symbol<'lib, unsafe extern "system" fn(*mut c_void)>>,
}

struct SenderState {
    inner: Mutex<UnboundedSender<CallbackEvent>>,
}

struct ProfitDll {
    _lib: Library,
    raw: ProfitRaw<'static>,
    sender: Arc<SenderState>,
}

static INSTANCE: OnceCell<ProfitDll> = OnceCell::new();
static CALLBACK_GUARD: OnceCell<Mutex<()>> = OnceCell::new();

unsafe extern "system" fn state_callback_trampoline(
    conn_type: c_int,
    result: NResult,
    _ctx: *mut c_void,
) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        let state = match conn_type {
            0 => ConnectionState::Login,
            1 => ConnectionState::Routing,
            2 => ConnectionState::MarketData,
            3 => ConnectionState::MarketLogin,
            _ => ConnectionState::Login,
        };
        if let Ok(guard) = inst.sender.inner.lock() {
            let _ = guard.send(CallbackEvent::StateChanged {
                connection_type: state,
                result,
            });
        }
    }
}

// (Demais trampolines e implementação idênticos ao original, omitidos por brevidade neste patch)
// Para manter o foco do rename, versões completas devem ser copiadas conforme necessidade futura.

pub struct ProfitConnector {
    _marker: (),
}

impl ProfitConnector {
    pub fn new(dll_path: Option<&str>) -> Result<Self, ProfitError> {
        // Carrega instância global somente uma vez.
        INSTANCE.get_or_try_init(|| {
            let path = dll_path.unwrap_or("ProfitDLL.dll");
            if std::env::var("PROFITDLL_DIAG")
                .map(|v| v == "1")
                .unwrap_or(false)
            {
                eprintln!("[profitdll][DIAG] Tentando carregar DLL em: {path}");
            }
            unsafe {
                let lib = Library::new(path).map_err(|e| ProfitError::Load(e.to_string()))?;
                if std::env::var("PROFITDLL_DIAG")
                    .map(|v| v == "1")
                    .unwrap_or(false)
                {
                    eprintln!("[profitdll][DIAG] DLL carregada com sucesso: {path}");
                }
                let raw = load_symbols(&lib)?;
                let (tx, _rx) = unbounded_channel::<CallbackEvent>(); // rx descartado - verdadeiro rx produzido em initialize_login
                Ok(ProfitDll {
                    _lib: lib,
                    raw,
                    sender: Arc::new(SenderState {
                        inner: Mutex::new(tx),
                    }),
                })
            }
        })?;
        Ok(Self { _marker: () })
    }

    pub async fn initialize_login(
        &self,
        _activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>, ProfitError> {
        let inst = INSTANCE.get().expect("instance after new");
        let (tx, rx) = unbounded_channel();
        if let Ok(mut guard) = inst.sender.inner.lock() {
            *guard = tx;
        }
        unsafe {
            // Unconditional trace (independente de variáveis) para confirmar entrada.
            eprintln!("[profitdll][TRACE] entered initialize_login (unconditional)");
            let diag = std::env::var("PROFITDLL_DIAG")
                .map(|v| v == "1")
                .unwrap_or(false);
            let debug_steps = std::env::var("PROFITDLL_DEBUG_STEPS")
                .map(|v| v == "1")
                .unwrap_or(false);
            if debug_steps {
                eprintln!(
                    "[profitdll][STEP] initialize_login start thread={:?}",
                    std::thread::current().id()
                );
            }
            if debug_steps || diag {
                // Resumo de presença de símbolos opcionais / críticos para depuração.
                eprintln!(
                    "[profitdll][STEP] Symbols: Initialize={:?} SetStateCallback=1 Login=1 \
OrderCb={:?} TradeCbV2={:?} TradeCbV1={:?} BookCbV2={:?} BookCbV1={:?} DailySumV2={:?} DailySumV1={:?} AccountCb={:?} InvalidTickerCb={:?} AdjustHistV2={:?} TheorPriceCb={:?} HistTradeCb={:?} GetHistoryTrades={:?}",
                    inst.raw.Initialize.is_some(),
                    inst.raw.SetOrderCallback.is_some(),
                    inst.raw.SetTradeCallbackV2.is_some(),
                    inst.raw.SetTradeCallback.is_some(),
                    inst.raw.SetBookCallbackV2.is_some(),
                    inst.raw.SetBookCallback.is_some(),
                    inst.raw.SetDailySummaryCallbackV2.is_some(),
                    inst.raw.SetDailySummaryCallback.is_some(),
                    inst.raw.SetAccountCallback.is_some(),
                    inst.raw.SetInvalidTickerCallback.is_some(),
                    inst.raw.SetAdjustHistoryCallbackV2.is_some(),
                    inst.raw.SetTheoreticalPriceCallback.is_some(),
                    inst.raw.SetHistoryTradeCallback.is_some(),
                    inst.raw.GetHistoryTrades.is_some(),
                );
            }
            unsafe fn with_timeout<F>(label: &str, debug: bool, f: F) -> Result<(), ProfitError>
            where
                F: FnOnce() -> Result<(), ProfitError> + Send + 'static,
            {
                if !debug {
                    return f();
                }
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let _ = tx.send(f());
                });
                match rx.recv_timeout(std::time::Duration::from_secs(5)) {
                    Ok(r) => r,
                    Err(_) => {
                        eprintln!("[profitdll][STEP][TIMEOUT] {label} >5s");
                        Err(ProfitError::Unknown(-999))
                    }
                }
            }
            if let Some(ref init) = inst.raw.Initialize {
                if diag {
                    eprintln!("[profitdll][DIAG] Initialize()...");
                }
                with_timeout("Initialize", debug_steps, || map((init)()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] Initialize OK");
                }
            } else if diag {
                eprintln!("[profitdll][DIAG] Initialize ausente");
            }
            if diag {
                eprintln!("[profitdll][DIAG] SetStateCallback...");
            }
            with_timeout("SetStateCallback", debug_steps, || {
                map((inst.raw.SetStateCallback)(
                    state_callback_trampoline,
                    ptr::null_mut(),
                ))
            })?;
            if diag {
                eprintln!("[profitdll][DIAG] SetStateCallback OK");
            }
            let c_user = std::ffi::CString::new(user).unwrap();
            let c_pass = std::ffi::CString::new(password).unwrap();
            if diag {
                eprintln!("[profitdll][DIAG] Login({user}, ****)...");
            }
            with_timeout("Login", debug_steps, move || {
                map((inst.raw.Login)(c_user.as_ptr(), c_pass.as_ptr()))
            })?;
            if diag {
                eprintln!("[profitdll][DIAG] Login OK");
            }
            // Registro de callbacks opcionais
            if let Some(ref cb) = inst.raw.SetOrderCallback {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando SetOrderCallback...");
                }
                map(cb(order_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] SetOrderCallback OK");
                }
            }
            // trade (preferência V2)
            if let Some(ref cb_v2) = inst.raw.SetTradeCallbackV2 {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando TradeCallbackV2...");
                }
                map(cb_v2(trade_callback_trampoline_v2, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] TradeCallbackV2 OK");
                }
            } else if let Some(ref cb) = inst.raw.SetTradeCallback {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando TradeCallback (V1)...");
                }
                map(cb(trade_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] TradeCallback (V1) OK");
                }
            }
            // book (preferência V2)
            if let Some(ref cb_v2) = inst.raw.SetBookCallbackV2 {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando BookCallbackV2...");
                }
                map(cb_v2(book_callback_trampoline_v2, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] BookCallbackV2 OK");
                }
            } else if let Some(ref cb) = inst.raw.SetBookCallback {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando BookCallback (V1)...");
                }
                map(cb(book_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] BookCallback (V1) OK");
                }
            }
            // daily summary (preferência V2)
            if let Some(ref cb_v2) = inst.raw.SetDailySummaryCallbackV2 {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando DailySummaryCallbackV2...");
                }
                map(cb_v2(daily_summary_callback_trampoline_v2, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] DailySummaryCallbackV2 OK");
                }
            } else if let Some(ref cb) = inst.raw.SetDailySummaryCallback {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando DailySummaryCallback (V1)...");
                }
                map(cb(daily_summary_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] DailySummaryCallback (V1) OK");
                }
            }
            // accounts
            if let Some(ref cb) = inst.raw.SetAccountCallback {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando AccountCallback...");
                }
                map(cb(account_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] AccountCallback OK");
                }
            }
            // invalid ticker
            if let Some(ref cb) = inst.raw.SetInvalidTickerCallback {
                if diag {
                    eprintln!("[profitdll][DIAG] Registrando InvalidTickerCallback...");
                }
                map(cb(invalid_ticker_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] InvalidTickerCallback OK");
                }
            }
            // ajustes corporativos (V2) - placeholder sem parse
            if let Some(ref cb) = inst.raw.SetAdjustHistoryCallbackV2 {
                if diag {
                    eprintln!(
                        "[profitdll][DIAG] Registrando AdjustHistoryCallbackV2 (placeholder)..."
                    );
                }
                map(cb(adjust_history_callback_trampoline_v2, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] AdjustHistoryCallbackV2 OK (placeholder)");
                }
            }
            // preço teórico - placeholder parcial
            if let Some(ref cb) = inst.raw.SetTheoreticalPriceCallback {
                if diag {
                    eprintln!(
                        "[profitdll][DIAG] Registrando TheoreticalPriceCallback (placeholder)..."
                    );
                }
                map(cb(theoretical_price_callback_trampoline, ptr::null_mut()))?;
                if diag {
                    eprintln!("[profitdll][DIAG] TheoreticalPriceCallback OK (placeholder)");
                }
            }
            // history trade callback - registrar após os demais
            if let Some(ref cb_hist) = inst.raw.SetHistoryTradeCallback {
                if diag {
                    eprintln!(
                        "[profitdll][DIAG] Registrando HistoryTradeCallback (placeholder)..."
                    );
                }
                map(cb_hist(
                    history_trade_callback_trampoline_placeholder,
                    ptr::null_mut(),
                ))?;
                if diag {
                    eprintln!("[profitdll][DIAG] HistoryTradeCallback OK (placeholder)");
                }
            }
        }
        Ok(rx)
    }

    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        with_instance(|inst| unsafe {
            let t = std::ffi::CString::new(ticker).unwrap();
            let e = std::ffi::CString::new(exchange).unwrap();
            map((inst.raw.SubscribeTicker)(t.as_ptr(), e.as_ptr()))
        })
    }
    pub fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        with_instance(|inst| unsafe {
            let t = std::ffi::CString::new(ticker).unwrap();
            let e = std::ffi::CString::new(exchange).unwrap();
            map((inst.raw.UnsubscribeTicker)(t.as_ptr(), e.as_ptr()))
        })
    }
    pub fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError> {
        with_instance(|inst| unsafe {
            if let Some(ref f) = inst.raw.SendOrder {
                let t = std::ffi::CString::new(order.asset.ticker.clone()).unwrap();
                let e = std::ffi::CString::new(order.asset.exchange.clone()).unwrap();
                let side = match order.side {
                    OrderSide::Buy => 0,
                    OrderSide::Sell => 1,
                };
                let validity = match order.validity {
                    OrderValidity::Day => 0,
                    OrderValidity::GoodTillCanceled => 1,
                    OrderValidity::ImmediateOrCancel => 3,
                    OrderValidity::FillOrKill => 4,
                };
                let price = order
                    .price
                    .map(|d| d.to_f64().unwrap())
                    .unwrap_or(SENTINEL_MARKET_OR_KEEP); // sentinel para mercado
                let qty = order.quantity.to_f64().unwrap();
                let c = CSendOrder {
                    icker: t.as_ptr(),
                    exchange: e.as_ptr(),
                    side,
                    quantity: qty,
                    price,
                    validity,
                };
                map(f(&c))
            } else {
                Err(ProfitError::MissingSymbol("SendOrder"))
            }
        })
    }
    pub fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError> {
        with_instance(|inst| unsafe {
            if let Some(ref f) = inst.raw.SendCancelOrderV2 {
                let c = CCancelOrder { order_id };
                map(f(&c))
            } else {
                Err(ProfitError::MissingSymbol("SendCancelOrderV2"))
            }
        })
    }
    pub fn change_order(
        &self,
        order_id: i64,
        new_price: Option<Decimal>,
        new_qty: Option<Decimal>,
    ) -> Result<(), ProfitError> {
        with_instance(|inst| unsafe {
            if let Some(ref f) = inst.raw.SendChangeOrderV2 {
                let c = CChangeOrder {
                    order_id,
                    new_price: new_price
                        .map(|d| d.to_f64().unwrap())
                        .unwrap_or(SENTINEL_MARKET_OR_KEEP),
                    new_quantity: new_qty
                        .map(|d| d.to_f64().unwrap())
                        .unwrap_or(SENTINEL_MARKET_OR_KEEP),
                };
                map(f(&c))
            } else {
                Err(ProfitError::MissingSymbol("SendChangeOrderV2"))
            }
        })
    }
    /// Solicita histórico de trades (pull). Ainda não há parsing de retorno; callback incremental será adicionado em etapa futura.
    pub fn get_history_trades(
        &self,
        ticker: &str,
        exchange: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> Result<(), ProfitError> {
        with_instance(|inst| unsafe {
            if let Some(ref f) = inst.raw.GetHistoryTrades {
                let diag = std::env::var("PROFITDLL_DIAG")
                    .map(|v| v == "1")
                    .unwrap_or(false);
                let t = std::ffi::CString::new(ticker).unwrap();
                let e = std::ffi::CString::new(exchange).unwrap();
                if diag {
                    eprintln!("[profitdll][DIAG] GetHistoryTrades(ticker={ticker}, exchange={exchange}, from_ms={from_ms}, to_ms={to_ms}) chamando...");
                }
                let raw_res = f(t.as_ptr(), e.as_ptr(), from_ms, to_ms);
                if diag {
                    eprintln!("[profitdll][DIAG] GetHistoryTrades retornou={:?}", raw_res);
                }
                map(raw_res)
            } else {
                Err(ProfitError::MissingSymbol("GetHistoryTrades"))
            }
        })
    }
}
pub const SENTINEL_MARKET_OR_KEEP: f64 = -1.0; // preço/quantidade especial (-1 => inferido ou manter)

// -------------------------------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------------------------------

fn with_instance<F, R>(f: F) -> Result<R, ProfitError>
where
    F: FnOnce(&ProfitDll) -> Result<R, ProfitError>,
{
    let inst = INSTANCE.get().ok_or(ProfitError::NotInitialized)?;
    f(inst)
}

// Busca símbolo obrigatório tentando múltiplos nomes (primeiro que existir).
unsafe fn load_required_any<'lib, T>(
    lib: &'lib Library,
    candidates: &[&str],
    debug: bool,
) -> Result<libloading::Symbol<'lib, T>, ProfitError> {
    for name in candidates {
        let full = format!("{name}\0");
        let bytes = full.as_bytes();
        match lib.get::<T>(bytes) {
            Ok(sym) => {
                if debug {
                    eprintln!("[profitdll][LOAD] OK (candidate): {name}");
                }
                return Ok(sym);
            }
            Err(_) => {
                if debug {
                    eprintln!("[profitdll][LOAD] Falhou candidate: {name}");
                }
            }
        }
    }
    // retorna erro com primeiro nome (ou placeholder se vazio)
    let first = candidates.first().copied().unwrap_or("<none>");
    // Converter para 'static leakando (custo irrelevante pois ocorre só em erro de init)
    let leaked: &'static str = Box::leak(first.to_string().into_boxed_str());
    Err(ProfitError::MissingSymbol(leaked))
}

unsafe fn load_symbols(lib: &Library) -> Result<ProfitRaw<'static>, ProfitError> {
    let debug_load = std::env::var("PROFITDLL_DEBUG_LOAD")
        .map(|v| v == "1")
        .unwrap_or(false);
    if debug_load {
        eprintln!("[profitdll][LOAD] Iniciando load_symbols");
    }
    macro_rules! must {
        ($name:ident : $t:ty) => {{
            if debug_load {
                eprintln!("[profitdll][LOAD] Obrigatório: {}", stringify!($name));
            }
            let res: Result<Symbol<$t>, _> = lib.get(concat!(stringify!($name), "\0").as_bytes());
            match res {
                Ok(sym) => {
                    if debug_load {
                        eprintln!("[profitdll][LOAD] OK: {}", stringify!($name));
                    }
                    sym
                }
                Err(_) => {
                    if debug_load {
                        eprintln!(
                            "[profitdll][LOAD][ERRO] Faltando obrigatório: {}",
                            stringify!($name)
                        );
                    }
                    return Err(ProfitError::MissingSymbol(stringify!($name)));
                }
            }
        }};
    }
    macro_rules! opt {
        ($name:ident : $t:ty) => {{
            if debug_load {
                eprintln!("[profitdll][LOAD] Opcional: {}", stringify!($name));
            }
            match lib.get(concat!(stringify!($name), "\0").as_bytes()) {
                Ok(s) => {
                    if debug_load {
                        eprintln!("[profitdll][LOAD] OK opcional: {}", stringify!($name));
                    }
                    Some(s)
                }
                Err(_) => {
                    if debug_load {
                        eprintln!("[profitdll][LOAD] Ausente opcional: {}", stringify!($name));
                    }
                    None
                }
            }
        }};
    }
    // Suporte a variantes de nome para Login (DLL pode exportar com prefixo diferente).
    let login_candidates_env = std::env::var("PROFITDLL_LOGIN_CANDIDATES").ok();
    let login_candidates_owned: Vec<String> = if let Some(raw) = login_candidates_env {
        raw.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        vec![
            "Login".to_string(),       // nome esperado original
            "ProfitLogin".to_string(), // variantes plausíveis
            "APILogin".to_string(),
            "LoginUser".to_string(),
            "LoginA".to_string(),
            "ProfitDllLogin".to_string(),
        ]
    };
    let mut fallback_vec = login_candidates_owned;
    if fallback_vec.is_empty() {
        fallback_vec.push("Login".to_string());
    }
    let login_candidates: Vec<&str> = fallback_vec.iter().map(|s| s.as_str()).collect();
    if debug_load {
        eprintln!("[profitdll][LOAD] Candidates Login: {:?}", login_candidates);
    }
    let login_sym: Symbol<
        unsafe extern "system" fn(user: *const c_char, pass: *const c_char) -> NResult,
    > = load_required_any(lib, &login_candidates, debug_load)?;

    let temp = ProfitRaw {
        Initialize: opt!(Initialize: unsafe extern "system" fn() -> NResult),
        _Finalize: opt!(Finalize: unsafe extern "system" fn() -> NResult),
        SetStateCallback: must!(SetStateCallback: unsafe extern "system" fn(StateCallbackRaw, *mut c_void) -> NResult),
        Login: login_sym,
        SubscribeTicker: must!(SubscribeTicker: unsafe extern "system" fn(ticker: *const c_char, exch: *const c_char) -> NResult),
        UnsubscribeTicker: must!(UnsubscribeTicker: unsafe extern "system" fn(ticker: *const c_char, exch: *const c_char) -> NResult),
        SetTradeCallback: opt!(SetTradeCallback: unsafe extern "system" fn(TradeCallbackRaw, *mut c_void) -> NResult),
        SetBookCallback: opt!(SetBookCallback: unsafe extern "system" fn(BookCallbackRaw, *mut c_void) -> NResult),
        SetDailySummaryCallback: opt!(SetDailySummaryCallback: unsafe extern "system" fn(DailySummaryCallbackRaw, *mut c_void) -> NResult),
        SetAccountCallback: opt!(SetAccountCallback: unsafe extern "system" fn(AccountCallbackRaw, *mut c_void) -> NResult),
        SetInvalidTickerCallback: opt!(SetInvalidTickerCallback: unsafe extern "system" fn(InvalidTickerCallbackRaw, *mut c_void) -> NResult),
        SetOrderCallback: opt!(SetOrderCallback: unsafe extern "system" fn(OrderCallbackRaw, *mut c_void) -> NResult),
        SetTradeCallbackV2: opt!(SetTradeCallbackV2: unsafe extern "system" fn(TradeCallbackRawV2, *mut c_void) -> NResult),
        SetBookCallbackV2: opt!(SetBookCallbackV2: unsafe extern "system" fn(BookCallbackRawV2, *mut c_void) -> NResult),
        SetDailySummaryCallbackV2: opt!(SetDailySummaryCallbackV2: unsafe extern "system" fn(DailySummaryCallbackRawV2, *mut c_void) -> NResult),
        SendOrder: opt!(SendOrder: unsafe extern "system" fn(*const CSendOrder) -> NResult),
        SendCancelOrderV2: opt!(SendCancelOrderV2: unsafe extern "system" fn(*const CCancelOrder) -> NResult),
        SendChangeOrderV2: opt!(SendChangeOrderV2: unsafe extern "system" fn(*const CChangeOrder) -> NResult),
        GetOrderDetails: opt!(GetOrderDetails: GetOrderDetailsFn),
        SetHistoryTradeCallback: opt!(SetHistoryTradeCallback: unsafe extern "system" fn(HistoryTradeCallbackRaw, *mut c_void) -> NResult),
        GetHistoryTrades: opt!(GetHistoryTrades: unsafe extern "system" fn(*const c_char, *const c_char, i64, i64) -> NResult),
        SetAdjustHistoryCallbackV2: opt!(SetAdjustHistoryCallbackV2: unsafe extern "system" fn(unsafe extern "system" fn(*const c_void, *mut c_void), *mut c_void) -> NResult),
        SetTheoreticalPriceCallback: opt!(SetTheoreticalPriceCallback: unsafe extern "system" fn(unsafe extern "system" fn(*const c_char, *const c_char, f64, f64, f64, f64, *mut c_void), *mut c_void) -> NResult),
        FreePointer: opt!(FreePointer: unsafe extern "system" fn(*mut c_void)),
    };
    if debug_load {
        eprintln!("[profitdll][LOAD] Concluído load_symbols");
    }
    // Elevamos lifetime para 'static pois a Library vive dentro de OnceCell enquanto o processo estiver ativo
    Ok(std::mem::transmute::<ProfitRaw<'_>, ProfitRaw<'static>>(
        temp,
    ))
}

fn map(code: NResult) -> Result<(), ProfitError> {
    ProfitError::from_nresult(code)
}

unsafe extern "system" fn order_callback_trampoline(order_id: i64, _ctx: *mut c_void) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            // Tenta snapshot completo
            if let Some(ref get_fn) = inst.raw.GetOrderDetails {
                let mut raw: COrderDetails = COrderDetails {
                    order_id: 0,
                    account_id: ptr::null(),
                    ticker: ptr::null(),
                    exchange: ptr::null(),
                    side: 0,
                    order_type: 0,
                    status: 0,
                    quantity: 0.0,
                    filled: 0.0,
                    price: 0.0,
                    stop_price: 0.0,
                    validity: 0,
                    text: ptr::null(),
                };
                if map(get_fn(order_id, &mut raw as *mut _)).is_ok() {
                    let to_string = |p: *const c_char| unsafe {
                        if p.is_null() {
                            String::new()
                        } else {
                            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
                        }
                    };
                    let evt = CallbackEvent::OrderSnapshot {
                        order_id: raw.order_id,
                        account_id: to_string(raw.account_id),
                        ticker: to_string(raw.ticker),
                        exchange: to_string(raw.exchange),
                        side: if raw.side == 0 {
                            OrderSide::Buy
                        } else {
                            OrderSide::Sell
                        },
                        order_type: match raw.order_type {
                            1 => OrderType::Market,
                            2 => OrderType::Limit,
                            4 => OrderType::StopLimit,
                            _ => OrderType::Market,
                        },
                        status: OrderStatus::from_i32(raw.status),
                        quantity: Decimal::try_from(raw.quantity).unwrap_or(Decimal::ZERO),
                        filled: Decimal::try_from(raw.filled).unwrap_or(Decimal::ZERO),
                        price: if raw.price <= 0.0 {
                            None
                        } else {
                            Decimal::try_from(raw.price).ok()
                        },
                        stop_price: if raw.stop_price <= 0.0 {
                            None
                        } else {
                            Decimal::try_from(raw.stop_price).ok()
                        },
                        validity: match raw.validity {
                            0 => OrderValidity::Day,
                            1 => OrderValidity::GoodTillCanceled,
                            3 => OrderValidity::ImmediateOrCancel,
                            4 => OrderValidity::FillOrKill,
                            _ => OrderValidity::Day,
                        },
                        text: {
                            let s = to_string(raw.text);
                            if s.is_empty() {
                                None
                            } else {
                                Some(s)
                            }
                        },
                    };
                    let _ = sender.send(evt);
                    return;
                }
            }
            let _ = sender.send(CallbackEvent::OrderUpdated { order_id });
        }
    }
}

// ---- Trampolines adicionais ----

unsafe extern "system" fn trade_callback_trampoline(
    ticker: *const c_char,
    exchange: *const c_char,
    price: f64,
    volume: f64,
    timestamp_ms: i64,
    buy_agent: *const c_char,
    sell_agent: *const c_char,
    trade_id: i64,
    is_edit: c_int,
    _ctx: *mut c_void,
) {
    if let Some(inst) = INSTANCE.get() {
        emit_trade(
            inst,
            ticker,
            exchange,
            price,
            volume,
            timestamp_ms,
            buy_agent,
            sell_agent,
            trade_id,
            is_edit != 0,
        );
    }
}

unsafe extern "system" fn trade_callback_trampoline_v2(
    trade: *const ProfitTrade,
    _ctx: *mut c_void,
) {
    if trade.is_null() {
        return;
    }
    if let Some(inst) = INSTANCE.get() {
        let t = &*trade;
        emit_trade(
            inst,
            t.ticker,
            t.exchange,
            t.price,
            t.volume,
            t.timestamp_ms,
            t.buy_agent,
            t.sell_agent,
            t.trade_id,
            t.is_edit != 0,
        );
    }
}

#[allow(clippy::too_many_arguments)]
unsafe fn emit_trade(
    inst: &ProfitDll,
    ticker: *const c_char,
    exchange: *const c_char,
    price: f64,
    volume: f64,
    timestamp_ms: i64,
    buy_agent: *const c_char,
    sell_agent: *const c_char,
    trade_id: i64,
    is_edit: bool,
) {
    let _lock = CALLBACK_GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    if let Ok(sender) = inst.sender.inner.lock() {
        let evt = CallbackEvent::NewTrade {
            ticker: cstr_to_string(ticker),
            exchange: cstr_to_string(exchange),
            price: Decimal::try_from(price).unwrap_or(Decimal::ZERO),
            volume: Decimal::try_from(volume).unwrap_or(Decimal::ZERO),
            timestamp: Utc
                .timestamp_millis_opt(timestamp_ms)
                .single()
                .unwrap_or_else(Utc::now),
            buy_agent: cstr_to_string(buy_agent),
            sell_agent: cstr_to_string(sell_agent),
            trade_id,
            is_edit,
        };
        let _ = sender.send(evt);
    }
}

unsafe extern "system" fn book_callback_trampoline(
    side: c_int,
    ticker: *const c_char,
    exchange: *const c_char,
    action: c_int,
    price: f64,
    position: c_int,
    _ctx: *mut c_void,
) {
    emit_book(side, ticker, exchange, action, price, position);
}

unsafe extern "system" fn book_callback_trampoline_v2(
    update: *const ProfitBookUpdate,
    _ctx: *mut c_void,
) {
    if update.is_null() {
        return;
    }
    let u = &*update;
    emit_book(u.side, u.ticker, u.exchange, u.action, u.price, u.position);
}

unsafe fn emit_book(
    side: c_int,
    ticker: *const c_char,
    exchange: *const c_char,
    action: c_int,
    price: f64,
    position: c_int,
) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            let book_action = match action {
                0 => BookAction::New,
                1 => BookAction::Edit,
                2 => BookAction::Delete,
                _ => BookAction::Edit,
            };
            let evt = if side == 0 {
                CallbackEvent::PriceBookOffer {
                    ticker: cstr_to_string(ticker),
                    exchange: cstr_to_string(exchange),
                    action: book_action,
                    price: Decimal::try_from(price).unwrap_or(Decimal::ZERO),
                    position,
                }
            } else {
                CallbackEvent::OfferBookBid {
                    ticker: cstr_to_string(ticker),
                    exchange: cstr_to_string(exchange),
                    action: book_action,
                    price: Decimal::try_from(price).unwrap_or(Decimal::ZERO),
                    position,
                }
            };
            let _ = sender.send(evt);
        }
    }
}

unsafe extern "system" fn daily_summary_callback_trampoline(
    ticker: *const c_char,
    exchange: *const c_char,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    adjustment: f64,
    max_limit: f64,
    min_limit: f64,
    trades_buyer: f64,
    trades_seller: f64,
    _ctx: *mut c_void,
) {
    emit_daily(
        ticker,
        exchange,
        open,
        high,
        low,
        close,
        volume,
        adjustment,
        max_limit,
        min_limit,
        trades_buyer,
        trades_seller,
    );
}

unsafe extern "system" fn daily_summary_callback_trampoline_v2(
    summary: *const ProfitDailySummary,
    _ctx: *mut c_void,
) {
    if summary.is_null() {
        return;
    }
    let s = &*summary;
    emit_daily(
        s.ticker,
        s.exchange,
        s.open,
        s.high,
        s.low,
        s.close,
        s.volume,
        s.adjustment,
        s.max_limit,
        s.min_limit,
        s.trades_buyer,
        s.trades_seller,
    );
}

#[allow(clippy::too_many_arguments)]
unsafe fn emit_daily(
    ticker: *const c_char,
    exchange: *const c_char,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    adjustment: f64,
    max_limit: f64,
    min_limit: f64,
    trades_buyer: f64,
    trades_seller: f64,
) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            let evt = CallbackEvent::DailySummary {
                ticker: cstr_to_string(ticker),
                exchange: cstr_to_string(exchange),
                open: dec(open),
                high: dec(high),
                low: dec(low),
                close: dec(close),
                volume: dec(volume),
                adjustment: dec(adjustment),
                max_limit: dec(max_limit),
                min_limit: dec(min_limit),
                trades_buyer: dec(trades_buyer),
                trades_seller: dec(trades_seller),
            };
            let _ = sender.send(evt);
        }
    }
}

unsafe extern "system" fn account_callback_trampoline(
    account_id: *const c_char,
    account_holder: *const c_char,
    broker_name: *const c_char,
    broker_id: c_int,
    _ctx: *mut c_void,
) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            let evt = CallbackEvent::AccountChanged {
                account_id: cstr_to_string(account_id),
                account_holder: cstr_to_string(account_holder),
                broker_name: cstr_to_string(broker_name),
                broker_id,
            };
            let _ = sender.send(evt);
        }
    }
}

unsafe extern "system" fn invalid_ticker_callback_trampoline(
    ticker: *const c_char,
    exchange: *const c_char,
    feed_type: c_int,
    _ctx: *mut c_void,
) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            let evt = CallbackEvent::InvalidTicker {
                ticker: cstr_to_string(ticker),
                exchange: cstr_to_string(exchange),
                feed_type,
            };
            let _ = sender.send(evt);
        }
    }
}

// Placeholder: assinatura exata de V2 de ajustes ainda não mapeada; recebemos ponteiro genérico.
unsafe extern "system" fn adjust_history_callback_trampoline_v2(
    data: *const c_void,
    _ctx: *mut c_void,
) {
    if data.is_null() {
        return;
    }
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            // Opcional: diagnóstico hexdump inicial das primeiras bytes para mapear layout futuro.
            if std::env::var("PROFITDLL_DIAG")
                .map(|v| v == "1")
                .unwrap_or(false)
            {
                unsafe {
                    let bytes = std::slice::from_raw_parts(data as *const u8, 64);
                    let mut line = String::new();
                    for (i, b) in bytes.iter().enumerate() {
                        line.push_str(&format!("{b:02X} "));
                        if (i + 1) % 16 == 0 {
                            line.push('|');
                        }
                    }
                    eprintln!("[profitdll][DIAG] AdjustHistory raw head: {line}");
                }
            }
            // Tenta parse do layout suposto; se algo incoerente cai no placeholder.
            let evt = unsafe {
                let rec = data as *const ProfitAdjustHistoryV2;
                if !rec.is_null() {
                    let r = &*rec;
                    // heurística simples: ticker e exchange não nulos e value não NaN
                    if !r.ticker.is_null() && !r.exchange.is_null() && r.value.is_finite() {
                        CallbackEvent::AdjustHistory {
                            ticker: cstr_to_string(r.ticker),
                            exchange: cstr_to_string(r.exchange),
                            value: dec(r.value),
                            adjust_type: cstr_to_string(r.adjust_type),
                            observation: cstr_to_string(r.observation),
                            date_adjust: cstr_to_string(r.date_adjust),
                            date_deliberation: cstr_to_string(r.date_deliberation),
                            date_payment: cstr_to_string(r.date_payment),
                            flags: r.flags,
                            multiplier: dec(r.multiplier),
                        }
                    } else {
                        CallbackEvent::AdjustHistory {
                            ticker: String::new(),
                            exchange: String::new(),
                            value: Decimal::ZERO,
                            adjust_type: String::from("<pending-parse>"),
                            observation: String::new(),
                            date_adjust: String::new(),
                            date_deliberation: String::new(),
                            date_payment: String::new(),
                            flags: 0,
                            multiplier: Decimal::ONE,
                        }
                    }
                } else {
                    CallbackEvent::AdjustHistory {
                        ticker: String::new(),
                        exchange: String::new(),
                        value: Decimal::ZERO,
                        adjust_type: String::from("<pending-parse>"),
                        observation: String::new(),
                        date_adjust: String::new(),
                        date_deliberation: String::new(),
                        date_payment: String::new(),
                        flags: 0,
                        multiplier: Decimal::ONE,
                    }
                }
            };
            let _ = sender.send(evt);
        }
    }
}

// Placeholder teórico: parâmetros após preço não totalmente definidos; mapeamos price e quantity.
unsafe extern "system" fn theoretical_price_callback_trampoline(
    ticker: *const c_char,
    exchange: *const c_char,
    theoretical_price: f64,
    quantity: f64,
    _p3: f64,
    _p4: f64,
    _ctx: *mut c_void,
) {
    if let Some(inst) = INSTANCE.get() {
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            let evt = CallbackEvent::TheoreticalPrice {
                ticker: cstr_to_string(ticker),
                exchange: cstr_to_string(exchange),
                theoretical_price: dec(theoretical_price),
                quantity: quantity as i64,
            };
            let _ = sender.send(evt);
        }
    }
}

unsafe extern "system" fn history_trade_callback_trampoline_placeholder(
    trade: *const ProfitTrade,
    _ctx: *mut c_void,
) {
    if trade.is_null() {
        return;
    }
    if let Some(inst) = INSTANCE.get() {
        let t = unsafe { &*trade };
        let _lock = CALLBACK_GUARD
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap();
        if let Ok(sender) = inst.sender.inner.lock() {
            let evt = CallbackEvent::HistoryTrade {
                ticker: cstr_to_string(t.ticker),
                exchange: cstr_to_string(t.exchange),
                price: dec(t.price),
                volume: dec(t.volume),
                timestamp: Utc
                    .timestamp_millis_opt(t.timestamp_ms)
                    .single()
                    .unwrap_or_else(Utc::now),
                qty: (t.volume.max(0.0) as i64).clamp(0, i32::MAX as i64) as i32,
                trade_id: t.trade_id,
                source: crate::mock::HistoryTradeSource::IncrementalCallback,
            };
            let _ = sender.send(evt);
        }
    }
}

// ---- Utilidades ----
fn cstr_to_string(p: *const c_char) -> String {
    unsafe {
        if p.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
        }
    }
}
fn dec(v: f64) -> Decimal {
    Decimal::try_from(v).unwrap_or(Decimal::ZERO)
}

#[allow(dead_code)]
pub fn wrap_foreign_buffer(ptr: *mut c_void) -> Option<ForeignBuffer> {
    #[cfg(all(target_os = "windows", feature = "real_dll"))]
    {
        if let Some(inst) = INSTANCE.get() {
            let free_fn = inst.raw.FreePointer.as_ref().map(|s| **s);
            return Some(ForeignBuffer::new(ptr, free_fn));
        }
    }
    None
}

// Fim

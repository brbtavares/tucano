//! Implementação FFI real (Windows + feature `real_dll`).
//!
//! Estratégia em camadas:
//! 1. Declarações mínimas de tipos/constantes espelhando ProfitDLL (subset inicial).
//! 2. Carregamento dinâmico via `libloading` para não exigir link em build cross.
//! 3. Tradução de callbacks para `CallbackEvent` (compartilhado) via canal.
//! 4. API segura `ProfitConnector` similar ao mock.
//!
//! Incremental: começamos apenas com inicialização, login e assinatura de ticker
//! para validar pipeline. Próximos passos (ordem):
//! - Mapear TODOS códigos NL_* restantes (atual: subset principal)
//! - Adicionar structs #[repr(C)] completas (ordens, negócios, livro)
//! - Registrar demais callbacks (trades, book, summary, account)
//! - Implementar envio/cancelamento de ordens
//! - Teste de integração (feature gated) verificando handshake básico
//! - Documentação de segurança (aliasing, threads)

#![allow(non_camel_case_types)]

use std::{ffi::{c_char, c_int, c_void}, ptr, sync::Mutex};
use libloading::{Library, Symbol};
use once_cell::sync::OnceCell;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use std::sync::Arc;

use crate::{CallbackEvent, ConnectionState, BookAction, error::*, SendOrder, OrderSide, OrderValidity, OrderType, OrderStatus};
use rust_decimal::Decimal;
use chrono::{Utc, TimeZone};

// Erros agora via crate::error::ProfitError / from_nresult

// ---- Assinaturas brutas (subset) ----

// Tipos de callback simplificados (refinar com assinatura real depois)
type StateCallbackRaw = unsafe extern "system" fn(conn_type: c_int, result: NResult, ctx: *mut c_void);
// Assumindo assinatura simplificada para trade: strings + primitivos. Ajustar para struct #[repr(C)] se necessário.
type TradeCallbackRaw = unsafe extern "system" fn(
	ticker: *const c_char,
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
// Book: side 0=offer 1=bid
type BookCallbackRaw = unsafe extern "system" fn(
	side: c_int,
	ticker: *const c_char,
	exchange: *const c_char,
	action: c_int,
	price: f64,
	position: c_int,
	ctx: *mut c_void,
);
// Daily summary
type DailySummaryCallbackRaw = unsafe extern "system" fn(
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
	ctx: *mut c_void,
);
// Account changed
type AccountCallbackRaw = unsafe extern "system" fn(
	account_id: *const c_char,
	account_holder: *const c_char,
	broker_name: *const c_char,
	broker_id: c_int,
	ctx: *mut c_void,
);
// Invalid ticker
type InvalidTickerCallbackRaw = unsafe extern "system" fn(
	ticker: *const c_char,
	exchange: *const c_char,
	feed_type: c_int,
	ctx: *mut c_void,
);

// Order status callback (simplificada: apenas order_id; expandir conforme layout real)
type OrderCallbackRaw = unsafe extern "system" fn(order_id: i64, ctx: *mut c_void);
type GetOrderDetailsFn = unsafe extern "system" fn(order_id: i64, out: *mut COrderDetails) -> NResult;

// ---- Versões baseadas em structs #[repr(C)] (V2 hipotéticas) ----
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

#[repr(C)]
pub struct ProfitBookUpdate {
	pub side: c_int, // 0 offer, 1 bid
	pub ticker: *const c_char,
	pub exchange: *const c_char,
	pub action: c_int, // 0 new 1 edit 2 delete
	pub price: f64,
	pub position: c_int,
}

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

type TradeCallbackRawV2 = unsafe extern "system" fn(trade: *const ProfitTrade, ctx: *mut c_void);
type BookCallbackRawV2 = unsafe extern "system" fn(update: *const ProfitBookUpdate, ctx: *mut c_void);
type DailySummaryCallbackRawV2 = unsafe extern "system" fn(summary: *const ProfitDailySummary, ctx: *mut c_void);

// Ordem simplificada (subset) para envio inicial – substituir por layout completo conforme manual.
#[repr(C)]
struct CSendOrder {
	ticker: *const c_char,
	exchange: *const c_char,
	side: c_int,      // 0 buy 1 sell (mapeado do mock OrderSide)
	quantity: f64,
	price: f64,       // -1.0 indica ordem a mercado
	validity: c_int,  // 0=Day 1=GTC 2=IOC 3=FOK
}

// Estruturas simplificadas para alteração/cancelamento
#[repr(C)]
struct CCancelOrder { order_id: i64 }
#[repr(C)]
struct CChangeOrder { order_id: i64, new_price: f64, new_quantity: f64 }

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

// Funções esperadas (nomes provisórios a confirmar com manual)
#[allow(non_snake_case)]
struct ProfitRaw<'lib> {
	Initialize: Symbol<'lib, unsafe extern "system" fn() -> NResult>,
	Finalize: Symbol<'lib, unsafe extern "system" fn() -> NResult>,
	SetStateCallback: Symbol<'lib, unsafe extern "system" fn(StateCallbackRaw, *mut c_void) -> NResult>,
	Login: Symbol<'lib, unsafe extern "system" fn(user: *const c_char, pass: *const c_char) -> NResult>,
	SubscribeTicker: Symbol<'lib, unsafe extern "system" fn(ticker: *const c_char, exch: *const c_char) -> NResult>,
	UnsubscribeTicker: Symbol<'lib, unsafe extern "system" fn(ticker: *const c_char, exch: *const c_char) -> NResult>,
	SetTradeCallback: Option<Symbol<'lib, unsafe extern "system" fn(TradeCallbackRaw, *mut c_void) -> NResult>>,
	SetBookCallback: Option<Symbol<'lib, unsafe extern "system" fn(BookCallbackRaw, *mut c_void) -> NResult>>,
	SetDailySummaryCallback: Option<Symbol<'lib, unsafe extern "system" fn(DailySummaryCallbackRaw, *mut c_void) -> NResult>>,
	SetAccountCallback: Option<Symbol<'lib, unsafe extern "system" fn(AccountCallbackRaw, *mut c_void) -> NResult>>,
	SetInvalidTickerCallback: Option<Symbol<'lib, unsafe extern "system" fn(InvalidTickerCallbackRaw, *mut c_void) -> NResult>>,
	SetOrderCallback: Option<Symbol<'lib, unsafe extern "system" fn(OrderCallbackRaw, *mut c_void) -> NResult>>,
	// V2 struct-based
	SetTradeCallbackV2: Option<Symbol<'lib, unsafe extern "system" fn(TradeCallbackRawV2, *mut c_void) -> NResult>>,
	SetBookCallbackV2: Option<Symbol<'lib, unsafe extern "system" fn(BookCallbackRawV2, *mut c_void) -> NResult>>,
	SetDailySummaryCallbackV2: Option<Symbol<'lib, unsafe extern "system" fn(DailySummaryCallbackRawV2, *mut c_void) -> NResult>>,
	SendOrder: Option<Symbol<'lib, unsafe extern "system" fn(*const CSendOrder) -> NResult>>,
	SendCancelOrderV2: Option<Symbol<'lib, unsafe extern "system" fn(*const CCancelOrder) -> NResult>>,
	SendChangeOrderV2: Option<Symbol<'lib, unsafe extern "system" fn(*const CChangeOrder) -> NResult>>,
	GetOrderDetails: Option<Symbol<'lib, GetOrderDetailsFn>>,
}

// ---- Loader singleton ----
struct SenderState { inner: Mutex<UnboundedSender<CallbackEvent>> }

struct ProfitDll {
	lib: Library,
	raw: ProfitRaw<'static>,
	sender: Arc<SenderState>,
}

static INSTANCE: OnceCell<ProfitDll> = OnceCell::new();
static CALLBACK_GUARD: OnceCell<Mutex<()>> = OnceCell::new();

unsafe extern "system" fn state_callback_trampoline(conn_type: c_int, result: NResult, _ctx: *mut c_void) {
	if let Some(inst) = INSTANCE.get() {
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let state = match conn_type { 0 => ConnectionState::Login, 1 => ConnectionState::Routing, 2 => ConnectionState::MarketData, 3 => ConnectionState::MarketLogin, _ => ConnectionState::Login };
		if let Ok(guard) = inst.sender.inner.lock() { let _ = guard.send(CallbackEvent::StateChanged { connection_type: state, result }); }
	}
}

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
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let to_str = |p: *const c_char| -> String {
			if p.is_null() { return String::new(); }
			unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
		};
		let price_d = Decimal::from_f64_retain(price).unwrap_or_default();
		let volume_d = Decimal::from_f64_retain(volume).unwrap_or_default();
		let ts = Utc.timestamp_millis_opt(timestamp_ms).single().unwrap_or_else(|| Utc.timestamp_nanos(0));
		if let Ok(guard) = inst.sender.inner.lock() {
			let _ = guard.send(CallbackEvent::NewTrade {
				ticker: to_str(ticker),
				exchange: to_str(exchange),
				price: price_d,
				volume: volume_d,
				timestamp: ts,
				buy_agent: to_str(buy_agent),
				sell_agent: to_str(sell_agent),
				trade_id,
				is_edit: is_edit != 0,
			});
		}
	}
}

unsafe extern "system" fn trade_callback_trampoline_v2(trade: *const ProfitTrade, ctx: *mut c_void) {
	if trade.is_null() { return; }
	unsafe {
		let t = &*trade;
		trade_callback_trampoline(
			t.ticker,
			t.exchange,
			t.price,
			t.volume,
			t.timestamp_ms,
			t.buy_agent,
			t.sell_agent,
			t.trade_id,
			t.is_edit,
			ctx,
		);
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
	if let Some(inst) = INSTANCE.get() {
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let to_str = |p: *const c_char| -> String { if p.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() } } };
		let price_d = Decimal::from_f64_retain(price).unwrap_or_default();
		let action_mapped = match action { 0 => BookAction::New, 1 => BookAction::Edit, 2 => BookAction::Delete, _ => BookAction::Edit };
		if let Ok(guard) = inst.sender.inner.lock() {
			let event = if side == 0 {
				CallbackEvent::PriceBookOffer { ticker: to_str(ticker), exchange: to_str(exchange), action: action_mapped, price: price_d, position }
			} else {
				CallbackEvent::OfferBookBid { ticker: to_str(ticker), exchange: to_str(exchange), action: action_mapped, price: price_d, position }
			};
			let _ = guard.send(event);
		}
	}
}

unsafe extern "system" fn book_callback_trampoline_v2(update: *const ProfitBookUpdate, ctx: *mut c_void) {
	if update.is_null() { return; }
	unsafe {
		let u = &*update;
		book_callback_trampoline(
			u.side,
			u.ticker,
			u.exchange,
			u.action,
			u.price,
			u.position,
			ctx,
		);
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
	if let Some(inst) = INSTANCE.get() {
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let to_str = |p: *const c_char| -> String { if p.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() } } };
		if let Ok(guard) = inst.sender.inner.lock() {
			let _ = guard.send(CallbackEvent::DailySummary {
				ticker: to_str(ticker),
				exchange: to_str(exchange),
				open: Decimal::from_f64_retain(open).unwrap_or_default(),
				high: Decimal::from_f64_retain(high).unwrap_or_default(),
				low: Decimal::from_f64_retain(low).unwrap_or_default(),
				close: Decimal::from_f64_retain(close).unwrap_or_default(),
				volume: Decimal::from_f64_retain(volume).unwrap_or_default(),
				adjustment: Decimal::from_f64_retain(adjustment).unwrap_or_default(),
				max_limit: Decimal::from_f64_retain(max_limit).unwrap_or_default(),
				min_limit: Decimal::from_f64_retain(min_limit).unwrap_or_default(),
				trades_buyer: Decimal::from_f64_retain(trades_buyer).unwrap_or_default(),
				trades_seller: Decimal::from_f64_retain(trades_seller).unwrap_or_default(),
			});
		}
	}
}

unsafe extern "system" fn daily_summary_callback_trampoline_v2(summary: *const ProfitDailySummary, ctx: *mut c_void) {
	if summary.is_null() { return; }
	unsafe {
		let s = &*summary;
		daily_summary_callback_trampoline(
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
			ctx,
		);
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
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let to_str = |p: *const c_char| -> String { if p.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() } } };
		if let Ok(guard) = inst.sender.inner.lock() {
			let _ = guard.send(CallbackEvent::AccountChanged {
				account_id: to_str(account_id),
				account_holder: to_str(account_holder),
				broker_name: to_str(broker_name),
				broker_id: broker_id as i32,
			});
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
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let to_str = |p: *const c_char| -> String { if p.is_null() { String::new() } else { unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() } } };
		if let Ok(guard) = inst.sender.inner.lock() {
			let _ = guard.send(CallbackEvent::InvalidTicker {
				ticker: to_str(ticker),
				exchange: to_str(exchange),
				feed_type: feed_type as i32,
			});
		}
	}
}

unsafe extern "system" fn order_callback_trampoline(order_id: i64, _ctx: *mut c_void) {
	if let Some(inst) = INSTANCE.get() {
		let _lock = CALLBACK_GUARD.get_or_init(|| Mutex::new(())).lock().unwrap();
		let mut fallback = true;
		if let Some(ref get_fn) = inst.raw.GetOrderDetails {
			let mut raw = COrderDetails { order_id: 0, account_id: ptr::null(), ticker: ptr::null(), exchange: ptr::null(), side: 0, order_type: 0, status: 0, quantity: 0.0, filled: 0.0, price: 0.0, stop_price: 0.0, validity: 0, text: ptr::null() };
			if (get_fn)(order_id, &mut raw as *mut COrderDetails) == NL_OK {
				let to_opt = |p: *const c_char| -> Option<String> { if p.is_null() { None } else { Some(unsafe { std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }) } };
				let to_str = |p: *const c_char| -> String { to_opt(p).unwrap_or_default() };
				let side = match raw.side { 0 => OrderSide::Buy, 1 => OrderSide::Sell, _ => OrderSide::Buy };
				let order_type = match raw.order_type { 1 => OrderType::Market, 2 => OrderType::Limit, 4 => OrderType::StopLimit, _ => OrderType::Limit };
				let status = OrderStatus::from_i32(raw.status);
				let validity = match raw.validity { 0 => OrderValidity::Day, 1 => OrderValidity::GoodTillCanceled, 2 => OrderValidity::ImmediateOrCancel, 3 => OrderValidity::FillOrKill, _ => OrderValidity::Day };
				let price = if raw.price <= 0.0 { None } else { Decimal::from_f64_retain(raw.price) };
				let stop_price = if raw.stop_price <= 0.0 { None } else { Decimal::from_f64_retain(raw.stop_price) };
				if let Ok(guard) = inst.sender.inner.lock() {
					let _ = guard.send(CallbackEvent::OrderSnapshot {
						order_id,
						account_id: to_str(raw.account_id),
						ticker: to_str(raw.ticker),
						exchange: to_str(raw.exchange),
						side,
						order_type,
						status,
						quantity: Decimal::from_f64_retain(raw.quantity).unwrap_or_default(),
						filled: Decimal::from_f64_retain(raw.filled).unwrap_or_default(),
						price: price.flatten(),
						stop_price: stop_price.flatten(),
						validity,
						text: to_opt(raw.text),
					});
					fallback = false;
				}
			}
		}
		if fallback { if let Ok(guard) = inst.sender.inner.lock() { let _ = guard.send(CallbackEvent::OrderUpdated { order_id }); } }
	}
}

fn cstring(s: &str) -> Vec<u8> { // simple helper (owned, null-terminated)
	let mut v = s.as_bytes().to_vec();
	v.push(0); v
}

/// Conector real (espelha API do mock)
pub struct ProfitConnector { _marker: () }

impl ProfitConnector {
	/// Carrega a DLL (caminho explícito ou nome padrão no PATH).
	pub fn new(dll_path: Option<&str>) -> Result<Self, ProfitError> {
		if INSTANCE.get().is_some() {
			return Ok(Self { _marker: () });
		}
		let path = dll_path.unwrap_or("ProfitDLL.dll");
		unsafe {
			let lib = Library::new(path).map_err(|e| ProfitError::Load(e.to_string()))?;
			macro_rules! sym { ($name:ident) => { lib.get::<unsafe extern "system" fn() -> NResult>(concat!(stringify!($name)).as_bytes()) }; }
			// Precisamos tipos distintos, então carregamos manualmente.
			let Initialize: Symbol<unsafe extern "system" fn() -> NResult> = lib.get(b"Initialize").map_err(|_| ProfitError::MissingSymbol("Initialize"))?;
			let Finalize: Symbol<unsafe extern "system" fn() -> NResult> = lib.get(b"Finalize").map_err(|_| ProfitError::MissingSymbol("Finalize"))?;
			let SetStateCallback: Symbol<unsafe extern "system" fn(StateCallbackRaw, *mut c_void) -> NResult> = lib.get(b"SetStateCallback").map_err(|_| ProfitError::MissingSymbol("SetStateCallback"))?;
			let Login: Symbol<unsafe extern "system" fn(*const c_char, *const c_char) -> NResult> = lib.get(b"Login").map_err(|_| ProfitError::MissingSymbol("Login"))?;
			let SubscribeTicker: Symbol<unsafe extern "system" fn(*const c_char, *const c_char) -> NResult> = lib.get(b"SubscribeTicker").map_err(|_| ProfitError::MissingSymbol("SubscribeTicker"))?;
			let UnsubscribeTicker: Symbol<unsafe extern "system" fn(*const c_char, *const c_char) -> NResult> = lib.get(b"UnsubscribeTicker").map_err(|_| ProfitError::MissingSymbol("UnsubscribeTicker"))?;

			let (tx, _rx) = unbounded_channel();
			let sender = Arc::new(SenderState { inner: Mutex::new(tx) });
			let SetTradeCallback = lib.get(b"SetTradeCallback").ok();
			let SetBookCallback = lib.get(b"SetBookCallback").ok();
			let SetDailySummaryCallback = lib.get(b"SetDailySummaryCallback").ok();
			let SetAccountCallback = lib.get(b"SetAccountCallback").ok();
			let SetInvalidTickerCallback = lib.get(b"SetInvalidTickerCallback").ok();
			let SetOrderCallback = lib.get(b"SetOrderCallback").ok();
			let SendCancelOrderV2 = lib.get(b"SendCancelOrderV2").ok();
			let SendChangeOrderV2 = lib.get(b"SendChangeOrderV2").ok();
			let SetTradeCallbackV2 = lib.get(b"SetTradeCallbackV2").ok();
			let SetBookCallbackV2 = lib.get(b"SetBookCallbackV2").ok();
			let SetDailySummaryCallbackV2 = lib.get(b"SetDailySummaryCallbackV2").ok();
			let SendOrder = lib.get(b"SendOrder").ok();
			let GetOrderDetails = lib.get(b"GetOrderDetails").ok();
			let raw = ProfitRaw { Initialize, Finalize, SetStateCallback, Login, SubscribeTicker, UnsubscribeTicker, SetTradeCallback, SetBookCallback, SetDailySummaryCallback, SetAccountCallback, SetInvalidTickerCallback, SetOrderCallback, SetTradeCallbackV2, SetBookCallbackV2, SetDailySummaryCallbackV2, SendOrder, SendCancelOrderV2, SendChangeOrderV2, GetOrderDetails };
			let inst = ProfitDll { lib, raw, sender };
			INSTANCE.set(std::mem::transmute::<ProfitDll, ProfitDll>(inst)).ok();
		}
		Ok(Self { _marker: () })
	}

	/// Inicializa, registra callback de estado e efetua login. Retorna receiver de eventos.
	pub async fn initialize_login(&self, _activation_key: &str, user: &str, password: &str) -> Result<tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>, ProfitError> {
		// Activation key ignorada por enquanto (necessário via função separada se existir)
		unsafe {
			let inst = INSTANCE.get().expect("DLL não carregada");
			ProfitError::from_nresult((inst.raw.Initialize)())?;
			ProfitError::from_nresult((inst.raw.SetStateCallback)(state_callback_trampoline, ptr::null_mut()))?;
			// Callbacks de trade e book são opcionais
			// Prefer V2 struct-based if disponível
			if let Some(ref f) = inst.raw.SetTradeCallbackV2 { let _ = ProfitError::from_nresult((f)(trade_callback_trampoline_v2, ptr::null_mut())); }
			else if let Some(ref f) = inst.raw.SetTradeCallback { let _ = ProfitError::from_nresult((f)(trade_callback_trampoline, ptr::null_mut())); }
			if let Some(ref f) = inst.raw.SetBookCallbackV2 { let _ = ProfitError::from_nresult((f)(book_callback_trampoline_v2, ptr::null_mut())); }
			else if let Some(ref f) = inst.raw.SetBookCallback { let _ = ProfitError::from_nresult((f)(book_callback_trampoline, ptr::null_mut())); }
			if let Some(ref f) = inst.raw.SetDailySummaryCallbackV2 { let _ = ProfitError::from_nresult((f)(daily_summary_callback_trampoline_v2, ptr::null_mut())); }
			else if let Some(ref f) = inst.raw.SetDailySummaryCallback { let _ = ProfitError::from_nresult((f)(daily_summary_callback_trampoline, ptr::null_mut())); }
			if let Some(ref f) = inst.raw.SetAccountCallback { let _ = ProfitError::from_nresult((f)(account_callback_trampoline, ptr::null_mut())); }
			if let Some(ref f) = inst.raw.SetInvalidTickerCallback { let _ = ProfitError::from_nresult((f)(invalid_ticker_callback_trampoline, ptr::null_mut())); }
			if let Some(ref f) = inst.raw.SetOrderCallback { let _ = ProfitError::from_nresult((f)(order_callback_trampoline, ptr::null_mut())); }
			let u = cstring(user); let p = cstring(password);
			ProfitError::from_nresult((inst.raw.Login)(u.as_ptr() as *const c_char, p.as_ptr() as *const c_char))?;
			let (new_tx, rx) = unbounded_channel();
			if let Ok(mut guard) = inst.sender.inner.lock() { *guard = new_tx; }
			Ok(rx)
		}
	}

	pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
		unsafe {
			let inst = INSTANCE.get().expect("DLL não carregada");
			let t = cstring(ticker); let e = cstring(exchange);
			ProfitError::from_nresult((inst.raw.SubscribeTicker)(t.as_ptr() as *const c_char, e.as_ptr() as *const c_char))
		}
	}
	pub fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
		unsafe {
			let inst = INSTANCE.get().expect("DLL não carregada");
			let t = cstring(ticker); let e = cstring(exchange);
			ProfitError::from_nresult((inst.raw.UnsubscribeTicker)(t.as_ptr() as *const c_char, e.as_ptr() as *const c_char))
		}
	}

	/// Envia uma ordem simplificada (market se price None) usando símbolo `SendOrder` quando disponível.
	pub fn send_order(&self, order: &SendOrder) -> Result<(), ProfitError> {
		unsafe {
			let inst = INSTANCE.get().expect("DLL não carregada");
			let Some(ref f) = inst.raw.SendOrder else { return Err(ProfitError::MissingSymbol("SendOrder")); };
			let t = cstring(&order.asset.ticker);
			let e = cstring(&order.asset.exchange);
			let side = match order.side { OrderSide::Buy => 0, OrderSide::Sell => 1 };
			let quantity = order.quantity.to_f64().unwrap_or(0.0);
			let price = match order.price { Some(p) => p.to_f64().unwrap_or(-1.0), None => -1.0 }; // -1.0 => market sentinel
			let validity = match order.validity { OrderValidity::Day => 0, OrderValidity::GoodTillCanceled => 1, OrderValidity::ImmediateOrCancel => 2, OrderValidity::FillOrKill => 3 };
			let c_ord = CSendOrder { ticker: t.as_ptr() as *const c_char, exchange: e.as_ptr() as *const c_char, side, quantity, price, validity };
			ProfitError::from_nresult((f)(&c_ord as *const CSendOrder))
		}
	}

	/// Solicita cancelamento de ordem pelo id local.
	pub fn cancel_order(&self, order_id: i64) -> Result<(), ProfitError> {
		unsafe {
			let inst = INSTANCE.get().expect("DLL não carregada");
			let Some(ref f) = inst.raw.SendCancelOrderV2 else { return Err(ProfitError::MissingSymbol("SendCancelOrderV2")); };
			let c = CCancelOrder { order_id };
			ProfitError::from_nresult((f)(&c as *const CCancelOrder))
		}
	}

	/// Solicita alteração de ordem (novo preço / quantidade). Use -1.0 para manter valor.
	pub fn change_order(&self, order_id: i64, new_price: Option<Decimal>, new_quantity: Option<Decimal>) -> Result<(), ProfitError> {
		unsafe {
			let inst = INSTANCE.get().expect("DLL não carregada");
			let Some(ref f) = inst.raw.SendChangeOrderV2 else { return Err(ProfitError::MissingSymbol("SendChangeOrderV2")); };
			let np = new_price.map(|d| d.to_f64().unwrap_or(-1.0)).unwrap_or(-1.0);
			let nq = new_quantity.map(|d| d.to_f64().unwrap_or(-1.0)).unwrap_or(-1.0);
			let c = CChangeOrder { order_id, new_price: np, new_quantity: nq };
			ProfitError::from_nresult((f)(&c as *const CChangeOrder))
		}
	}

	/// TODO: implementar get_order_details para converter order_id -> OrderSnapshot (necessário struct C completa).
	#[allow(dead_code)]
	fn get_order_snapshot(&self, _order_id: i64) {
		// Placeholder – dependerá de símbolos: GetOrderDetails / GetOrder / GetOrderProfitID
	}
}

impl Drop for ProfitConnector {
	fn drop(&mut self) {
		// Não finalizamos globalmente múltiplas vezes; simples best-effort.
		unsafe {
			if let Some(inst) = INSTANCE.get() { let _ = ProfitError::from_nresult((inst.raw.Finalize)()); }
		}
	}
}

// Segurança / Limitations (resumo):
// - Uso de singleton global para simplificar: ProfitDLL presumidamente modelo process-wide.
// - Interior mutability implementada via Mutex substituindo hack anterior de cast mutável.
// - Callbacks adicionais serão adicionados incrementalmente.
// - TODO: adicionar testes cfg(windows) para garantir símbolos.


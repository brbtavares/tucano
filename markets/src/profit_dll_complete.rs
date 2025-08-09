//! Implementação completa da ProfitDLL com FFI para Windows
//!
//! Este módulo contém a implementação real da ProfitDLL incluindo:
//! - Bindings FFI para as funções da DLL
//! - Implementação híbrida (mock + real)
//! - Configuração condicional para Windows/Linux

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// ============================================================================
// FFI BINDINGS PARA PROFITDLL (WINDOWS APENAS)
// ============================================================================

#[cfg(target_os = "windows")]
#[link(name = "ProfitDLL")]
extern "C" {
    // Conexão e autenticação
    fn DLL_Initialize(activation_key: *const c_char) -> c_int;
    fn DLL_Login(user: *const c_char, password: *const c_char) -> c_int;
    fn DLL_Logout() -> c_int;
    fn DLL_Disconnect() -> c_int;

    // Configuração de callbacks
    fn DLL_SetOnConnect(callback: extern "C" fn(connection_type: c_int, result: c_int));
    fn DLL_SetOnDisconnect(callback: extern "C" fn(connection_type: c_int));
    fn DLL_SetOnLogin(callback: extern "C" fn(connection_type: c_int, result: c_int));
    fn DLL_SetOnSubscriptionProgress(
        callback: extern "C" fn(
            ticker: *const c_char,
            exchange: *const c_char,
            feed_type: c_int,
            progress: c_int,
        ),
    );

    // Market Data Callbacks
    fn DLL_SetOnNewTrade(
        callback: extern "C" fn(
            ticker: *const c_char,
            exchange: *const c_char,
            price: f64,
            volume: f64,
            timestamp: i64,
            buy_agent: *const c_char,
            sell_agent: *const c_char,
            trade_id: i64,
            is_edit: c_int,
        ),
    );

    fn DLL_SetOnDailySummary(
        callback: extern "C" fn(
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
        ),
    );

    fn DLL_SetOnPriceBookOffer(
        callback: extern "C" fn(
            ticker: *const c_char,
            exchange: *const c_char,
            operation: c_int,
            position: c_int,
            quantity: f64,
            price: f64,
            qtd_offers: c_int,
        ),
    );

    // Subscrições
    fn DLL_SubscribeTicker(
        ticker: *const c_char,
        exchange: *const c_char,
        feed_type: c_int,
    ) -> c_int;
    fn DLL_UnsubscribeTicker(
        ticker: *const c_char,
        exchange: *const c_char,
        feed_type: c_int,
    ) -> c_int;
    fn DLL_SubscribeAccount(account: *const c_char) -> c_int;
    fn DLL_UnsubscribeAccount(account: *const c_char) -> c_int;

    // Execução de ordens
    fn DLL_SendOrder(
        ticker: *const c_char,
        exchange: *const c_char,
        account: *const c_char,
        side: c_int,
        quantity: f64,
        price: f64,
        validity: c_int,
        order_type: c_int,
    ) -> c_int;

    fn DLL_CancelOrder(order_id: *const c_char) -> c_int;
    fn DLL_ModifyOrder(order_id: *const c_char, quantity: f64, price: f64) -> c_int;

    // Consultas
    fn DLL_GetAccountBalance(account: *const c_char) -> f64;
    fn DLL_GetAccountPosition(
        account: *const c_char,
        ticker: *const c_char,
        exchange: *const c_char,
    ) -> f64;
}

// ============================================================================
// IMPLEMENTAÇÃO HÍBRIDA DO PROFIT CONNECTOR
// ============================================================================

/// Global event sender para callbacks C
static mut GLOBAL_EVENT_SENDER: Option<Arc<Mutex<mpsc::UnboundedSender<CallbackEvent>>>> = None;

/// Implementação completa do ProfitConnector com suporte a DLL real
#[derive(Debug)]
pub struct ProfitConnector {
    dll_path: Option<String>,
    is_initialized: bool,
    is_logged_in: bool,
    event_sender: Option<mpsc::UnboundedSender<CallbackEvent>>,
}

impl ProfitConnector {
    pub fn new(dll_path: Option<&str>) -> Result<Self, String> {
        Ok(Self {
            dll_path: dll_path.map(|s| s.to_string()),
            is_initialized: false,
            is_logged_in: false,
            event_sender: None,
        })
    }

    pub async fn initialize_login(
        &mut self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<mpsc::UnboundedReceiver<CallbackEvent>, String> {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.event_sender = Some(sender.clone());

        // Configurar sender global para callbacks C
        unsafe {
            GLOBAL_EVENT_SENDER = Some(Arc::new(Mutex::new(sender)));
        }

        #[cfg(target_os = "windows")]
        {
            if self.dll_path.is_some() {
                return self
                    .initialize_real_dll(activation_key, user, password, receiver)
                    .await;
            }
        }

        // Fallback para implementação mock
        self.initialize_mock(activation_key, user, password, receiver)
            .await
    }

    #[cfg(target_os = "windows")]
    async fn initialize_real_dll(
        &mut self,
        activation_key: &str,
        user: &str,
        password: &str,
        receiver: mpsc::UnboundedReceiver<CallbackEvent>,
    ) -> Result<mpsc::UnboundedReceiver<CallbackEvent>, String> {
        unsafe {
            // Configurar todos os callbacks
            DLL_SetOnConnect(on_connect_callback);
            DLL_SetOnDisconnect(on_disconnect_callback);
            DLL_SetOnLogin(on_login_callback);
            DLL_SetOnSubscriptionProgress(on_subscription_progress_callback);
            DLL_SetOnNewTrade(on_new_trade_callback);
            DLL_SetOnDailySummary(on_daily_summary_callback);
            DLL_SetOnPriceBookOffer(on_price_book_offer_callback);

            // Inicializar DLL
            let key_cstr =
                CString::new(activation_key).map_err(|_| "Erro ao converter activation_key")?;
            let result = DLL_Initialize(key_cstr.as_ptr());

            if result != NL_OK {
                return Err(format!("Falha na inicialização da DLL: {}", result));
            }

            self.is_initialized = true;

            // Fazer login
            let user_cstr = CString::new(user).map_err(|_| "Erro ao converter usuário")?;
            let pass_cstr = CString::new(password).map_err(|_| "Erro ao converter senha")?;
            let login_result = DLL_Login(user_cstr.as_ptr(), pass_cstr.as_ptr());

            if login_result != NL_OK {
                return Err(format!("Falha no login: {}", login_result));
            }

            self.is_logged_in = true;
        }

        println!("✅ ProfitConnector: Inicializado com DLL real");
        Ok(receiver)
    }

    async fn initialize_mock(
        &mut self,
        _activation_key: &str,
        _user: &str,
        _password: &str,
        receiver: mpsc::UnboundedReceiver<CallbackEvent>,
    ) -> Result<mpsc::UnboundedReceiver<CallbackEvent>, String> {
        self.is_initialized = true;
        self.is_logged_in = true;
        println!("🔄 ProfitConnector: Usando implementação mock");
        Ok(receiver)
    }

    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), String> {
        if !self.is_logged_in {
            return Err("Não logado".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            if self.dll_path.is_some() {
                unsafe {
                    let ticker_cstr =
                        CString::new(ticker).map_err(|_| "Erro ao converter ticker")?;
                    let exchange_cstr =
                        CString::new(exchange).map_err(|_| "Erro ao converter exchange")?;

                    let result = DLL_SubscribeTicker(
                        ticker_cstr.as_ptr(),
                        exchange_cstr.as_ptr(),
                        0, // FEED_TYPE_TRADES
                    );

                    if result != NL_OK {
                        return Err(format!("Falha na subscrição: {}", result));
                    }
                }

                println!(
                    "📊 ProfitConnector (DLL): Subscribed to {} on {}",
                    ticker, exchange
                );
                return Ok(());
            }
        }

        // Mock implementation
        println!(
            "📊 ProfitConnector (Mock): Subscribing to {} on {}",
            ticker, exchange
        );
        Ok(())
    }

    pub fn send_order(&self, order: &SendOrder) -> Result<String, String> {
        if !self.is_logged_in {
            return Err("Não logado".to_string());
        }

        #[cfg(target_os = "windows")]
        {
            if self.dll_path.is_some() {
                unsafe {
                    let ticker_cstr = CString::new(&order.asset.ticker)
                        .map_err(|_| "Erro ao converter ticker")?;
                    let exchange_cstr = CString::new(&order.asset.exchange)
                        .map_err(|_| "Erro ao converter exchange")?;
                    let account_cstr = CString::new(&order.account.account_id)
                        .map_err(|_| "Erro ao converter account")?;

                    let side = match order.side {
                        OrderSide::Buy => 1,
                        OrderSide::Sell => 2,
                    };

                    let price = order.price.unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0);
                    let quantity = order.quantity.to_f64().unwrap_or(0.0);

                    let validity = match order.validity {
                        OrderValidity::Day => 1,
                        OrderValidity::GoodTillCanceled => 2,
                        OrderValidity::ImmediateOrCancel => 3,
                        OrderValidity::FillOrKill => 4,
                    };

                    let order_type = if order.price.is_some() { 1 } else { 2 }; // Limit vs Market

                    let result = DLL_SendOrder(
                        ticker_cstr.as_ptr(),
                        exchange_cstr.as_ptr(),
                        account_cstr.as_ptr(),
                        side,
                        quantity,
                        price,
                        validity,
                        order_type,
                    );

                    if result < 0 {
                        return Err(format!("Falha no envio da ordem: {}", result));
                    }

                    return Ok(result.to_string());
                }
            }
        }

        // Mock implementation
        let order_id = format!("MOCK_{}", chrono::Utc::now().timestamp_millis());
        println!(
            "💼 ProfitConnector (Mock): Sending order {} for {} {}",
            order_id, order.quantity, order.asset.ticker
        );
        Ok(order_id)
    }
}

// ============================================================================
// CALLBACKS C PARA EVENTOS DA DLL
// ============================================================================

#[cfg(target_os = "windows")]
extern "C" fn on_connect_callback(connection_type: c_int, result: c_int) {
    send_event(CallbackEvent::StateChanged {
        connection_type: ConnectionState::from(connection_type),
        result,
    });
}

#[cfg(target_os = "windows")]
extern "C" fn on_disconnect_callback(connection_type: c_int) {
    send_event(CallbackEvent::StateChanged {
        connection_type: ConnectionState::from(connection_type),
        result: -1, // Disconnected
    });
}

#[cfg(target_os = "windows")]
extern "C" fn on_login_callback(connection_type: c_int, result: c_int) {
    send_event(CallbackEvent::StateChanged {
        connection_type: ConnectionState::from(connection_type),
        result,
    });
}

#[cfg(target_os = "windows")]
extern "C" fn on_subscription_progress_callback(
    ticker: *const c_char,
    exchange: *const c_char,
    feed_type: c_int,
    progress: c_int,
) {
    let ticker_str = unsafe { CStr::from_ptr(ticker).to_string_lossy().to_string() };
    let exchange_str = unsafe { CStr::from_ptr(exchange).to_string_lossy().to_string() };

    send_event(CallbackEvent::ProgressChanged {
        ticker: ticker_str,
        exchange: exchange_str,
        feed_type,
        progress,
    });
}

#[cfg(target_os = "windows")]
extern "C" fn on_new_trade_callback(
    ticker: *const c_char,
    exchange: *const c_char,
    price: f64,
    volume: f64,
    timestamp: i64,
    buy_agent: *const c_char,
    sell_agent: *const c_char,
    trade_id: i64,
    is_edit: c_int,
) {
    let ticker_str = unsafe { CStr::from_ptr(ticker).to_string_lossy().to_string() };
    let exchange_str = unsafe { CStr::from_ptr(exchange).to_string_lossy().to_string() };
    let buy_agent_str = unsafe { CStr::from_ptr(buy_agent).to_string_lossy().to_string() };
    let sell_agent_str = unsafe { CStr::from_ptr(sell_agent).to_string_lossy().to_string() };

    send_event(CallbackEvent::NewTrade {
        ticker: ticker_str,
        exchange: exchange_str,
        price: Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO),
        volume: Decimal::from_f64_retain(volume).unwrap_or(Decimal::ZERO),
        timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or(Utc::now()),
        buy_agent: buy_agent_str,
        sell_agent: sell_agent_str,
        trade_id,
        is_edit: is_edit != 0,
    });
}

#[cfg(target_os = "windows")]
extern "C" fn on_daily_summary_callback(
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
    let ticker_str = unsafe { CStr::from_ptr(ticker).to_string_lossy().to_string() };
    let exchange_str = unsafe { CStr::from_ptr(exchange).to_string_lossy().to_string() };

    send_event(CallbackEvent::DailySummary {
        ticker: ticker_str,
        exchange: exchange_str,
        open: Decimal::from_f64_retain(open).unwrap_or(Decimal::ZERO),
        high: Decimal::from_f64_retain(high).unwrap_or(Decimal::ZERO),
        low: Decimal::from_f64_retain(low).unwrap_or(Decimal::ZERO),
        close: Decimal::from_f64_retain(close).unwrap_or(Decimal::ZERO),
        volume: Decimal::from_f64_retain(volume).unwrap_or(Decimal::ZERO),
        adjustment: Decimal::from_f64_retain(adjustment).unwrap_or(Decimal::ZERO),
        max_limit: Decimal::from_f64_retain(max_limit).unwrap_or(Decimal::ZERO),
        min_limit: Decimal::from_f64_retain(min_limit).unwrap_or(Decimal::ZERO),
        trades_buyer: Decimal::from_f64_retain(trades_buyer).unwrap_or(Decimal::ZERO),
        trades_seller: Decimal::from_f64_retain(trades_seller).unwrap_or(Decimal::ZERO),
    });
}

#[cfg(target_os = "windows")]
extern "C" fn on_price_book_offer_callback(
    ticker: *const c_char,
    exchange: *const c_char,
    operation: c_int,
    position: c_int,
    quantity: f64,
    price: f64,
    qtd_offers: c_int,
) {
    let ticker_str = unsafe { CStr::from_ptr(ticker).to_string_lossy().to_string() };
    let exchange_str = unsafe { CStr::from_ptr(exchange).to_string_lossy().to_string() };

    send_event(CallbackEvent::PriceBookOffer {
        ticker: ticker_str,
        exchange: exchange_str,
        operation,
        position,
        quantity: Decimal::from_f64_retain(quantity).unwrap_or(Decimal::ZERO),
        price: Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO),
        qtd_offers,
    });
}

fn send_event(event: CallbackEvent) {
    unsafe {
        if let Some(sender_arc) = &GLOBAL_EVENT_SENDER {
            if let Ok(sender) = sender_arc.lock() {
                let _ = sender.send(event);
            }
        }
    }
}

// ============================================================================
// RE-EXPORT DOS TIPOS EXISTENTES
// ============================================================================

pub use crate::profit_dll::*;

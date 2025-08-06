//! Utilitários para interface com Windows DLL

use crate::{ProfitError, NResult, SendOrder, Position};
use chrono::{DateTime, Utc};

// Alias para Result com ProfitError como erro padrão
type Result<T> = std::result::Result<T, ProfitError>;

#[cfg(windows)]
use crate::{
    error::Result,
    types::*,
    callbacks::*,
};

#[cfg(windows)]
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HMODULE, SYSTEMTIME},
        System::LibraryLoader::{FreeLibrary, GetProcAddress, LoadLibraryW},
    },
};

#[cfg(windows)]
use std::{
    ffi::c_void,
    mem,
    ptr,
};

#[cfg(windows)]
use chrono::{DateTime, Utc};

/// Carregador da DLL para interface com ProfitDLL
#[cfg(windows)]
pub struct DllLoader {
    handle: HMODULE,
}

#[cfg(windows)]
impl DllLoader {
    /// Carrega a DLL do caminho especificado ou procura automaticamente
    pub fn new(dll_path: Option<&str>) -> Result<Self> {
        let path = if let Some(custom_path) = dll_path {
            custom_path.to_string()
        } else {
            // Procurar automaticamente em locais padrão
            Self::find_dll_path()?
        };
        
        let wide_path = to_wide_string(&path);
        
        unsafe {
            let handle = LoadLibraryW(PCWSTR(wide_path.as_ptr()))?;
            if handle.0 == 0 {
                return Err(ProfitError::LibraryNotFound(path));
            }
            
            tracing::debug!("ProfitDLL loaded successfully from: {}", path);
            Ok(Self { handle })
        }
    }

    /// Encontra automaticamente o caminho da DLL
    fn find_dll_path() -> Result<String> {
        let possible_paths = vec![
            // 1. Diretório lib/ relativo ao projeto
            "./lib/ProfitDLL.dll",
            "../lib/ProfitDLL.dll", 
            "../../lib/ProfitDLL.dll",
            
            // 2. Diretório de trabalho atual
            "./ProfitDLL.dll",
            
            // 3. PATH do sistema (Windows procura automaticamente)
            "ProfitDLL.dll",
        ];

        for path in possible_paths {
            if std::path::Path::new(path).exists() {
                tracing::debug!("Found ProfitDLL at: {}", path);
                return Ok(path.to_string());
            }
        }

        // Se não encontrar, usa o nome padrão e deixa o Windows procurar no PATH
        tracing::warn!("ProfitDLL not found in standard locations, trying system PATH");
        Ok("ProfitDLL.dll".to_string())
    }

    /// Obtém ponteiro para função da DLL
    fn get_function<T>(&self, name: &str) -> Result<T> {
        let name_cstr = std::ffi::CString::new(name)
            .map_err(|_| ProfitError::StringConversion)?;
        
        unsafe {
            let proc = GetProcAddress(self.handle, windows::core::PCSTR(name_cstr.as_ptr() as *const u8));
            if proc.is_none() {
                return Err(ProfitError::FunctionNotFound(name.to_string()));
            }
            
            Ok(mem::transmute_copy(&proc))
        }
    }

    /// Inicializa login completo (routing + market data)
    pub fn dll_initialize_login(
        &self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<NResult> {
        type DllInitializeLoginFn = unsafe extern "stdcall" fn(
            activation_key: *const u16,
            user: *const u16,
            password: *const u16,
            state_callback: extern "stdcall" fn(i32, i32),
            history_callback: *const c_void,
            order_change_callback: *const c_void,
            account_callback: extern "stdcall" fn(*const u16, *const u16, *const u16, i32),
            new_trade_callback: extern "stdcall" fn(*const u16, *const u16, i32, *const u16, u32, f64, f64, i32, i32, i32, i32, u8),
            new_daily_callback: extern "stdcall" fn(*const u16, *const u16, i32, *const u16, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, i32, i32, i32, i32, i32, i32, i32),
            price_book_callback: extern "stdcall" fn(*const u16, *const u16, i32, i32, i32, i32, i64, i32, f64, *const u8, *const u8),
            offer_book_callback: extern "stdcall" fn(*const u16, *const u16, i32, i32, i32, i32, i64, i32, f64, *const u8, *const u8),
            history_trade_callback: *const c_void,
            progress_callback: extern "stdcall" fn(*const u16, *const u16, i32, i32),
            tiny_book_callback: *const c_void,
        ) -> NResult;

        let func: DllInitializeLoginFn = self.get_function("DLLInitializeLogin")?;
        
        let activation_key_wide = to_wide_string(activation_key);
        let user_wide = to_wide_string(user);
        let password_wide = to_wide_string(password);

        unsafe {
            let result = func(
                activation_key_wide.as_ptr(),
                user_wide.as_ptr(),
                password_wide.as_ptr(),
                state_callback,
                ptr::null(),           // history_callback
                ptr::null(),           // order_change_callback
                account_callback,
                new_trade_callback,
                daily_callback,
                price_book_callback,
                offer_book_callback,
                ptr::null(),           // history_trade_callback
                progress_callback,
                ptr::null(),           // tiny_book_callback
            );
            Ok(result)
        }
    }

    /// Inicializa apenas market data
    pub fn dll_initialize_market_login(
        &self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<NResult> {
        type DllInitializeMarketLoginFn = unsafe extern "stdcall" fn(
            activation_key: *const u16,
            user: *const u16,
            password: *const u16,
            state_callback: extern "stdcall" fn(i32, i32),
            new_trade_callback: extern "stdcall" fn(*const u16, *const u16, i32, *const u16, u32, f64, f64, i32, i32, i32, i32, u8),
            new_daily_callback: extern "stdcall" fn(*const u16, *const u16, i32, *const u16, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, i32, i32, i32, i32, i32, i32, i32),
            price_book_callback: extern "stdcall" fn(*const u16, *const u16, i32, i32, i32, i32, i64, i32, f64, *const u8, *const u8),
            offer_book_callback: extern "stdcall" fn(*const u16, *const u16, i32, i32, i32, i32, i64, i32, f64, *const u8, *const u8),
            history_trade_callback: *const c_void,
            progress_callback: extern "stdcall" fn(*const u16, *const u16, i32, i32),
            tiny_book_callback: *const c_void,
        ) -> NResult;

        let func: DllInitializeMarketLoginFn = self.get_function("DLLInitializeMarketLogin")?;
        
        let activation_key_wide = to_wide_string(activation_key);
        let user_wide = to_wide_string(user);
        let password_wide = to_wide_string(password);

        unsafe {
            let result = func(
                activation_key_wide.as_ptr(),
                user_wide.as_ptr(),
                password_wide.as_ptr(),
                state_callback,
                new_trade_callback,
                daily_callback,
                price_book_callback,
                offer_book_callback,
                ptr::null(),           // history_trade_callback
                progress_callback,
                ptr::null(),           // tiny_book_callback
            );
            Ok(result)
        }
    }

    /// Finaliza a DLL
    pub fn dll_finalize(&self) -> Result<NResult> {
        type DllFinalizeFn = unsafe extern "stdcall" fn() -> NResult;
        let func: DllFinalizeFn = self.get_function("DLLFinalize")?;
        
        unsafe { Ok(func()) }
    }

    /// Subscreve ticker para cotações
    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<NResult> {
        type SubscribeTickerFn = unsafe extern "stdcall" fn(*const u16, *const u16) -> NResult;
        let func: SubscribeTickerFn = self.get_function("SubscribeTicker")?;
        
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe { Ok(func(ticker_wide.as_ptr(), exchange_wide.as_ptr())) }
    }

    /// Remove subscrição de ticker
    pub fn unsubscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<NResult> {
        type UnsubscribeTickerFn = unsafe extern "stdcall" fn(*const u16, *const u16) -> NResult;
        let func: UnsubscribeTickerFn = self.get_function("UnsubscribeTicker")?;
        
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe { Ok(func(ticker_wide.as_ptr(), exchange_wide.as_ptr())) }
    }

    /// Subscreve price book
    pub fn subscribe_price_book(&self, ticker: &str, exchange: &str) -> Result<NResult> {
        type SubscribePriceBookFn = unsafe extern "stdcall" fn(*const u16, *const u16) -> NResult;
        let func: SubscribePriceBookFn = self.get_function("SubscribePriceBook")?;
        
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe { Ok(func(ticker_wide.as_ptr(), exchange_wide.as_ptr())) }
    }

    /// Remove subscrição de price book
    pub fn unsubscribe_price_book(&self, ticker: &str, exchange: &str) -> Result<NResult> {
        type UnsubscribePriceBookFn = unsafe extern "stdcall" fn(*const u16, *const u16) -> NResult;
        let func: UnsubscribePriceBookFn = self.get_function("UnsubscribePriceBook")?;
        
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe { Ok(func(ticker_wide.as_ptr(), exchange_wide.as_ptr())) }
    }

    /// Envia ordem usando estrutura SendOrder
    pub fn send_order(&self, order: &SendOrder) -> Result<i64> {
        type SendOrderFn = unsafe extern "stdcall" fn(*const c_void) -> i64;
        let func: SendOrderFn = self.get_function("SendOrder")?;
        
        // Converter para estrutura C compatível
        let c_order = self.send_order_to_c_struct(order)?;
        
        unsafe { Ok(func(&c_order as *const _ as *const c_void)) }
    }

    /// Envia ordem de compra limitada (função deprecated, mas mantida para compatibilidade)
    pub fn send_buy_order(
        &self,
        account_id: &str,
        broker_id: &str,
        password: &str,
        ticker: &str,
        exchange: &str,
        price: f64,
        quantity: i32,
    ) -> Result<i64> {
        type SendBuyOrderFn = unsafe extern "stdcall" fn(
            *const u16, *const u16, *const u16, *const u16, *const u16, f64, i32
        ) -> i64;
        let func: SendBuyOrderFn = self.get_function("SendBuyOrder")?;
        
        let account_wide = to_wide_string(account_id);
        let broker_wide = to_wide_string(broker_id);
        let password_wide = to_wide_string(password);
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe {
            Ok(func(
                account_wide.as_ptr(),
                broker_wide.as_ptr(),
                password_wide.as_ptr(),
                ticker_wide.as_ptr(),
                exchange_wide.as_ptr(),
                price,
                quantity,
            ))
        }
    }

    /// Envia ordem de venda limitada (função deprecated, mas mantida para compatibilidade)
    pub fn send_sell_order(
        &self,
        account_id: &str,
        broker_id: &str,
        password: &str,
        ticker: &str,
        exchange: &str,
        price: f64,
        quantity: i32,
    ) -> Result<i64> {
        type SendSellOrderFn = unsafe extern "stdcall" fn(
            *const u16, *const u16, *const u16, *const u16, *const u16, f64, i32
        ) -> i64;
        let func: SendSellOrderFn = self.get_function("SendSellOrder")?;
        
        let account_wide = to_wide_string(account_id);
        let broker_wide = to_wide_string(broker_id);
        let password_wide = to_wide_string(password);
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe {
            Ok(func(
                account_wide.as_ptr(),
                broker_wide.as_ptr(),
                password_wide.as_ptr(),
                ticker_wide.as_ptr(),
                exchange_wide.as_ptr(),
                price,
                quantity,
            ))
        }
    }

    /// Cancela ordem
    pub fn send_cancel_order(
        &self,
        account_id: &str,
        broker_id: &str,
        cl_order_id: &str,
        password: &str,
    ) -> Result<NResult> {
        type SendCancelOrderFn = unsafe extern "stdcall" fn(
            *const u16, *const u16, *const u16, *const u16
        ) -> NResult;
        let func: SendCancelOrderFn = self.get_function("SendCancelOrder")?;
        
        let account_wide = to_wide_string(account_id);
        let broker_wide = to_wide_string(broker_id);
        let cl_order_wide = to_wide_string(cl_order_id);
        let password_wide = to_wide_string(password);
        
        unsafe {
            Ok(func(
                account_wide.as_ptr(),
                broker_wide.as_ptr(),
                cl_order_wide.as_ptr(),
                password_wide.as_ptr(),
            ))
        }
    }

    /// Obtém posição (versão simplificada)
    pub fn get_position(
        &self,
        account_id: &str,
        broker_id: &str,
        ticker: &str,
        exchange: &str,
    ) -> Result<Position> {
        type GetPositionFn = unsafe extern "stdcall" fn(
            *const u16, *const u16, *const u16, *const u16
        ) -> *const c_void;
        let func: GetPositionFn = self.get_function("GetPosition")?;
        
        let account_wide = to_wide_string(account_id);
        let broker_wide = to_wide_string(broker_id);
        let ticker_wide = to_wide_string(ticker);
        let exchange_wide = to_wide_string(exchange);
        
        unsafe {
            let position_ptr = func(
                account_wide.as_ptr(),
                broker_wide.as_ptr(),
                ticker_wide.as_ptr(),
                exchange_wide.as_ptr(),
            );
            
            if position_ptr.is_null() {
                return Err(ProfitError::NoPosition);
            }
            
            // Parse da estrutura retornada (implementação simplificada)
            self.parse_position_data(position_ptr, account_id, broker_id, ticker, exchange)
        }
    }

    /// Obtém número de contas
    pub fn get_account_count(&self) -> Result<i32> {
        type GetAccountCountFn = unsafe extern "stdcall" fn() -> i32;
        let func: GetAccountCountFn = self.get_function("GetAccountCount")?;
        
        unsafe { Ok(func()) }
    }

    /// Obtém relógio do servidor
    pub fn get_server_clock(&self) -> Result<DateTime<Utc>> {
        type GetServerClockFn = unsafe extern "stdcall" fn(
            *mut f64, *mut i32, *mut i32, *mut i32, *mut i32, *mut i32, *mut i32, *mut i32
        ) -> NResult;
        let func: GetServerClockFn = self.get_function("GetServerClock")?;
        
        unsafe {
            let mut dt_date: f64 = 0.0;
            let mut year: i32 = 0;
            let mut month: i32 = 0;
            let mut day: i32 = 0;
            let mut hour: i32 = 0;
            let mut min: i32 = 0;
            let mut sec: i32 = 0;
            let mut milisec: i32 = 0;
            
            let result = func(&mut dt_date, &mut year, &mut month, &mut day, 
                             &mut hour, &mut min, &mut sec, &mut milisec);
            
            if result != NL_OK {
                return Err(ProfitError::from(result));
            }
            
            // Converter para DateTime
            use chrono::TimeZone;
            let dt = Utc.with_ymd_and_hms(year, month as u32, day as u32, 
                                         hour as u32, min as u32, sec as u32)
                .single()
                .ok_or(ProfitError::InvalidDateFormat("Invalid server time".to_string()))?;
            
            Ok(dt)
        }
    }

    /// Define uso de day trade
    pub fn set_day_trade(&self, use_day_trade: i32) -> Result<NResult> {
        type SetDayTradeFn = unsafe extern "stdcall" fn(i32) -> NResult;
        let func: SetDayTradeFn = self.get_function("SetDayTrade")?;
        
        unsafe { Ok(func(use_day_trade)) }
    }

    /// Converte SendOrder para estrutura C
    fn send_order_to_c_struct(&self, order: &SendOrder) -> Result<CConnectorSendOrder> {
        Ok(CConnectorSendOrder {
            version: order.version,
            account_id: CConnectorAccountIdentifier {
                version: order.account_id.version,
                broker_id: order.account_id.broker_id,
                account_id: to_wide_string(&order.account_id.account_id).as_ptr(),
                sub_account_id: to_wide_string(&order.account_id.sub_account_id).as_ptr(),
                reserved: order.account_id.reserved,
            },
            asset_id: CConnectorAssetIdentifier {
                version: order.asset_id.version,
                ticker: to_wide_string(&order.asset_id.ticker).as_ptr(),
                exchange: to_wide_string(&order.asset_id.exchange).as_ptr(),
                feed_type: order.asset_id.feed_type,
            },
            password: to_wide_string(&order.password).as_ptr(),
            order_type: order.order_type as u8,
            order_side: order.order_side as u8,
            price: order.price,
            stop_price: order.stop_price,
            quantity: order.quantity,
        })
    }

    /// Parse dados de posição retornados pela DLL
    fn parse_position_data(
        &self,
        _ptr: *const c_void,
        account_id: &str,
        broker_id: &str,
        ticker: &str,
        exchange: &str,
    ) -> Result<Position> {
        // Implementação simplificada - na prática, seria necessário
        // fazer parse completo da estrutura retornada pela DLL
        Ok(Position {
            version: 0,
            account_id: AccountIdentifier::new(
                broker_id.parse().unwrap_or(0),
                account_id.to_string(),
                String::new(),
            ),
            asset_id: AssetIdentifier::new(ticker, exchange, 0),
            open_quantity: 0,
            open_average_price: rust_decimal::Decimal::ZERO,
            open_side: PositionSide::Unknown,
            daily_average_sell_price: rust_decimal::Decimal::ZERO,
            daily_sell_quantity: 0,
            daily_average_buy_price: rust_decimal::Decimal::ZERO,
            daily_buy_quantity: 0,
            daily_quantity_d1: 0,
            daily_quantity_d2: 0,
            daily_quantity_d3: 0,
            daily_quantity_blocked: 0,
            daily_quantity_pending: 0,
            daily_quantity_alloc: 0,
            daily_quantity_provision: 0,
            daily_quantity: 0,
            daily_quantity_available: 0,
            position_type: PositionType::Consolidated,
        })
    }
}

#[cfg(windows)]
impl Drop for DllLoader {
    fn drop(&mut self) {
        unsafe {
            let _ = FreeLibrary(self.handle);
        }
    }
}

// Estruturas C para interface com DLL
#[cfg(windows)]
#[repr(C)]
struct CConnectorAccountIdentifier {
    version: u8,
    broker_id: i32,
    account_id: *const u16,
    sub_account_id: *const u16,
    reserved: i64,
}

#[cfg(windows)]
#[repr(C)]
struct CConnectorAssetIdentifier {
    version: u8,
    ticker: *const u16,
    exchange: *const u16,
    feed_type: u8,
}

#[cfg(windows)]
#[repr(C)]
struct CConnectorSendOrder {
    version: u8,
    account_id: CConnectorAccountIdentifier,
    asset_id: CConnectorAssetIdentifier,
    password: *const u16,
    order_type: u8,
    order_side: u8,
    price: f64,
    stop_price: f64,
    quantity: i64,
}

// Versão mock para non-Windows
#[cfg(not(windows))]
pub struct DllLoader;

#[cfg(not(windows))]
impl DllLoader {
    pub fn new(_dll_path: Option<&str>) -> Result<Self> {
        Err(ProfitError::NotInitialized)
    }

    pub fn dll_initialize_login(&self, _: &str, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn dll_initialize_market_login(&self, _: &str, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn dll_finalize(&self) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn subscribe_ticker(&self, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn unsubscribe_ticker(&self, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn subscribe_price_book(&self, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn unsubscribe_price_book(&self, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn send_order(&self, _: &SendOrder) -> Result<i64> {
        Err(ProfitError::NotInitialized)
    }

    pub fn send_buy_order(&self, _: &str, _: &str, _: &str, _: &str, _: &str, _: f64, _: i32) -> Result<i64> {
        Err(ProfitError::NotInitialized)
    }

    pub fn send_sell_order(&self, _: &str, _: &str, _: &str, _: &str, _: &str, _: f64, _: i32) -> Result<i64> {
        Err(ProfitError::NotInitialized)
    }

    pub fn send_cancel_order(&self, _: &str, _: &str, _: &str, _: &str) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }

    pub fn get_position(&self, _: &str, _: &str, _: &str, _: &str) -> Result<Position> {
        Err(ProfitError::NotInitialized)
    }

    pub fn get_account_count(&self) -> Result<i32> {
        Err(ProfitError::NotInitialized)
    }

    pub fn get_server_clock(&self) -> Result<DateTime<Utc>> {
        Err(ProfitError::NotInitialized)
    }

    pub fn set_day_trade(&self, _: i32) -> Result<NResult> {
        Err(ProfitError::NotInitialized)
    }
}

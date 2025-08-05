//! Tipos de dados para interface com ProfitDLL

use chrono::{DateTime, Utc, TimeZone};
use serde::{Deserialize, Serialize};
use std::ffi::{CString, OsString};
use rust_decimal::Decimal;
use smol_str::SmolStr;

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

/// Resultado das operações da DLL
pub type NResult = i32;

// Constantes de resultado - Error Codes da documentação
pub const NL_OK: NResult = 0;
pub const NL_INTERNAL_ERROR: NResult = -2147483647; // 0x80000001
pub const NL_NOT_INITIALIZED: NResult = -2147483646; // 0x80000002
pub const NL_INVALID_ARGS: NResult = -2147483645; // 0x80000003
pub const NL_WAITING_SERVER: NResult = -2147483644; // 0x80000004
pub const NL_NO_LOGIN: NResult = -2147483643; // 0x80000005
pub const NL_NO_LICENSE: NResult = -2147483642; // 0x80000006
pub const NL_OUT_OF_RANGE: NResult = -2147483639; // 0x80000009
pub const NL_MARKET_ONLY: NResult = -2147483638; // 0x8000000A
pub const NL_NO_POSITION: NResult = -2147483637; // 0x8000000B
pub const NL_NOT_FOUND: NResult = -2147483636; // 0x8000000C
pub const NL_VERSION_NOT_SUPPORTED: NResult = -2147483635; // 0x8000000D
pub const NL_OCO_NO_RULES: NResult = -2147483634; // 0x8000000E
pub const NL_EXCHANGE_UNKNOWN: NResult = -2147483633; // 0x8000000F
pub const NL_NO_OCO_DEFINED: NResult = -2147483632; // 0x80000010
pub const NL_INVALID_SERIE: NResult = -2147483631; // 0x80000011
pub const NL_LICENSE_NOT_ALLOWED: NResult = -2147483630; // 0x80000012
pub const NL_NOT_HARD_LOGOUT: NResult = -2147483629; // 0x80000013
pub const NL_SERIE_NO_HISTORY: NResult = -2147483628; // 0x80000014
pub const NL_ASSET_NO_DATA: NResult = -2147483627; // 0x80000015
pub const NL_SERIE_NO_DATA: NResult = -2147483626; // 0x80000016
pub const NL_SERIE_NO_MORE_HISTORY: NResult = -2147483624; // 0x80000018
pub const NL_SERIE_MAX_COUNT: NResult = -2147483623; // 0x80000019
pub const NL_DUPLICATE_RESOURCE: NResult = -2147483622; // 0x8000001A
pub const NL_UNSIGNED_CONTRACT: NResult = -2147483621; // 0x8000001B
pub const NL_NO_PASSWORD: NResult = -2147483620; // 0x8000001C
pub const NL_NO_USER: NResult = -2147483619; // 0x8000001D
pub const NL_FILE_ALREADY_EXISTS: NResult = -2147483618; // 0x8000001E
pub const NL_INVALID_TICKER: NResult = -2147483617; // 0x8000001F
pub const NL_NOT_MASTER_ACCOUNT: NResult = -2147483616; // 0x80000020

/// Estados de conexão
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ConnectionState {
    Login = 0,
    Routing = 1,
    MarketData = 2,
    MarketLogin = 3,
}

/// Estados de login
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum LoginState {
    Connected = 0,
    Invalid = 1,
    InvalidPassword = 2,
    BlockedPassword = 3,
    ExpiredPassword = 4,
    UnknownError = 200,
}

/// Estados de roteamento
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum RoutingState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    BrokerDisconnected = 3,
    BrokerConnecting = 4,
    BrokerConnected = 5,
}

/// Estados de market data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum MarketState {
    Disconnected = 0,
    Connecting = 1,
    Waiting = 2,
    NotLogged = 3,
    Connected = 4,
}

/// Tipos de ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderType {
    Market = 1,
    Limit = 2,
    StopLimit = 4,
}

/// Lado da ordem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OrderSide {
    Buy = 1,
    Sell = 2,
}

/// Tipo de posição
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PositionType {
    DayTrade = 1,
    Consolidated = 2,
}

/// Lado da posição
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PositionSide {
    Unknown = 0,
    Purchased = 1,
    Sold = 2,
}

/// Identificador de conta
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountIdentifier {
    pub version: u8,
    pub broker_id: i32,
    pub account_id: String,
    pub sub_account_id: String,
    pub reserved: i64,
}

impl AccountIdentifier {
    pub fn new(broker_id: i32, account_id: String, sub_account_id: String) -> Self {
        Self {
            version: 0,
            broker_id,
            account_id,
            sub_account_id,
            reserved: 0,
        }
    }

    /// Cria identificador apenas com conta principal
    pub fn main_account(broker_id: i32, account_id: String) -> Self {
        Self::new(broker_id, account_id, String::new())
    }

    /// Verifica se é uma subconta
    pub fn is_sub_account(&self) -> bool {
        !self.sub_account_id.is_empty()
    }
}

/// Identificador de ativo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetIdentifier {
    pub version: u8,
    pub ticker: SmolStr,
    pub exchange: SmolStr,
    pub feed_type: u8,
}

impl AssetIdentifier {
    pub fn new(ticker: &str, exchange: &str, feed_type: u8) -> Self {
        Self {
            version: 0,
            ticker: SmolStr::new(ticker),
            exchange: SmolStr::new(exchange),
            feed_type,
        }
    }

    pub fn ticker(&self) -> &str {
        &self.ticker
    }

    pub fn exchange(&self) -> &str {
        &self.exchange
    }

    /// Cria ativo para Bovespa
    pub fn bovespa(ticker: &str) -> Self {
        Self::new(ticker, "B", 0)
    }

    /// Cria ativo para BMF
    pub fn bmf(ticker: &str) -> Self {
        Self::new(ticker, "F", 0)
    }
}

/// Identificador de ordem
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderIdentifier {
    pub version: u8,
    pub local_order_id: i64,
    pub cl_order_id: String,
}

impl OrderIdentifier {
    pub fn new(local_order_id: i64, cl_order_id: String) -> Self {
        Self {
            version: 0,
            local_order_id,
            cl_order_id,
        }
    }

    pub fn from_local_id(local_order_id: i64) -> Self {
        Self::new(local_order_id, String::new())
    }

    pub fn from_cl_order_id(cl_order_id: String) -> Self {
        Self::new(0, cl_order_id)
    }
}

/// Estrutura para envio de ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOrder {
    pub version: u8,
    pub account_id: AccountIdentifier,
    pub asset_id: AssetIdentifier,
    pub password: String,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub price: f64,
    pub stop_price: f64,
    pub quantity: i64,
}

impl SendOrder {
    /// Cria ordem a mercado
    pub fn new_market_order(
        account_id: AccountIdentifier,
        asset_id: AssetIdentifier,
        password: String,
        order_side: OrderSide,
        quantity: i64,
    ) -> Self {
        Self {
            version: 0,
            account_id,
            asset_id,
            password,
            order_type: OrderType::Market,
            order_side,
            price: -1.0, // Market orders use -1
            stop_price: -1.0,
            quantity,
        }
    }

    /// Cria ordem limitada
    pub fn new_limit_order(
        account_id: AccountIdentifier,
        asset_id: AssetIdentifier,
        password: String,
        order_side: OrderSide,
        price: f64,
        quantity: i64,
    ) -> Self {
        Self {
            version: 0,
            account_id,
            asset_id,
            password,
            order_type: OrderType::Limit,
            order_side,
            price,
            stop_price: -1.0,
            quantity,
        }
    }

    /// Cria ordem stop
    pub fn new_stop_order(
        account_id: AccountIdentifier,
        asset_id: AssetIdentifier,
        password: String,
        order_side: OrderSide,
        price: f64,
        stop_price: f64,
        quantity: i64,
    ) -> Self {
        Self {
            version: 0,
            account_id,
            asset_id,
            password,
            order_type: OrderType::StopLimit,
            order_side,
            price,
            stop_price,
            quantity,
        }
    }
}

/// Estrutura para modificação de ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeOrder {
    pub version: u8,
    pub account_id: AccountIdentifier,
    pub order_id: OrderIdentifier,
    pub password: String,
    pub price: f64,
    pub stop_price: f64,
    pub quantity: i64,
}

/// Estrutura para cancelamento de ordem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrder {
    pub version: u8,
    pub account_id: AccountIdentifier,
    pub order_id: OrderIdentifier,
    pub password: String,
}

/// Trade information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub version: u8,
    pub asset_id: AssetIdentifier,
    pub trade_date: DateTime<Utc>,
    pub trade_number: u32,
    pub price: Decimal,
    pub quantity: i64,
    pub volume: Decimal,
    pub buy_agent: i32,
    pub sell_agent: i32,
    pub trade_type: u8,
    pub is_edit: bool,
}

/// Informações de ordem completa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub version: u8,
    pub order_id: OrderIdentifier,
    pub account_id: AccountIdentifier,
    pub asset_id: AssetIdentifier,
    pub quantity: i64,
    pub traded_quantity: i64,
    pub leaves_quantity: i64,
    pub price: Decimal,
    pub stop_price: Decimal,
    pub average_price: Decimal,
    pub order_side: OrderSide,
    pub order_type: OrderType,
    pub order_status: u8,
    pub validity_type: u8,
    pub date: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub close_date: Option<DateTime<Utc>>,
    pub validity_date: Option<DateTime<Utc>>,
    pub text_message: String,
}

/// Informações de posição
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub version: u8,
    pub account_id: AccountIdentifier,
    pub asset_id: AssetIdentifier,
    pub open_quantity: i64,
    pub open_average_price: Decimal,
    pub open_side: PositionSide,
    pub daily_average_sell_price: Decimal,
    pub daily_sell_quantity: i64,
    pub daily_average_buy_price: Decimal,
    pub daily_buy_quantity: i64,
    pub daily_quantity_d1: i64,
    pub daily_quantity_d2: i64,
    pub daily_quantity_d3: i64,
    pub daily_quantity_blocked: i64,
    pub daily_quantity_pending: i64,
    pub daily_quantity_alloc: i64,
    pub daily_quantity_provision: i64,
    pub daily_quantity: i64,
    pub daily_quantity_available: i64,
    pub position_type: PositionType,
}

/// Book entry (price level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookEntry {
    pub price: Decimal,
    pub quantity: i64,
    pub count: i32,
}

/// Order book completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub asset_id: AssetIdentifier,
    pub buy_levels: Vec<BookEntry>,
    pub sell_levels: Vec<BookEntry>,
    pub timestamp: DateTime<Utc>,
}

/// Ações do book
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum BookAction {
    Add = 0,
    Edit = 1,
    Delete = 2,
    DeleteFrom = 3,
    FullBook = 4,
}

/// Informações de conta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub version: u8,
    pub account_id: AccountIdentifier,
    pub broker_name: String,
    pub owner_name: String,
    pub sub_owner_name: String,
    pub account_flags: u32,
}

/// Dados diários agregados
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyData {
    pub asset_id: AssetIdentifier,
    pub date: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub adjustment: Decimal,
    pub max_limit: Decimal,
    pub min_limit: Decimal,
    pub volume_buyer: Decimal,
    pub volume_seller: Decimal,
    pub quantity: i32,
    pub trades_count: i32,
    pub open_contracts: i32,
    pub quantity_buyer: i32,
    pub quantity_seller: i32,
    pub trades_buyer: i32,
    pub trades_seller: i32,
}

/// Exchange constants
pub mod exchanges {
    pub const BCB: &str = "A";      // Banco Central
    pub const BOVESPA: &str = "B";  // Bovespa
    pub const CAMBIO: &str = "D";   // Câmbio
    pub const ECONOMIC: &str = "E"; // Dados econômicos
    pub const BMF: &str = "F";      // BMF
    pub const METRICS: &str = "K";  // Métricas
    pub const CME: &str = "M";      // CME
    pub const NASDAQ: &str = "N";   // Nasdaq
    pub const OXR: &str = "O";      // OXR
    pub const PIONEER: &str = "P";  // Pioneer
    pub const DOW_JONES: &str = "X"; // Dow Jones
    pub const NYSE: &str = "Y";     // NYSE
}

/// Flags para trade callbacks
pub const TC_IS_EDIT: u32 = 1;
pub const TC_LAST_PACKET: u32 = 2;

/// Utility functions for Windows string conversion
#[cfg(windows)]
pub(crate) fn to_wide_string(s: &str) -> Vec<u16> {
    OsString::from(s).encode_wide().chain(Some(0)).collect()
}

#[cfg(windows)]
pub(crate) fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    
    unsafe {
        let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(ptr, len);
        String::from_utf16_lossy(slice)
    }
}

#[cfg(not(windows))]
pub(crate) fn to_wide_string(_s: &str) -> Vec<u16> {
    Vec::new()
}

#[cfg(not(windows))]
pub(crate) fn from_wide_ptr(_ptr: *const u16) -> String {
    String::new()
}

/// Converte DateTime para formato Windows SYSTEMTIME
#[cfg(windows)]
pub(crate) fn datetime_to_systemtime(dt: DateTime<Utc>) -> windows::Win32::Foundation::SYSTEMTIME {
    let dt_local = dt.naive_utc();
    windows::Win32::Foundation::SYSTEMTIME {
        wYear: dt_local.year() as u16,
        wMonth: dt_local.month() as u16,
        wDayOfWeek: dt_local.weekday().num_days_from_sunday() as u16,
        wDay: dt_local.day() as u16,
        wHour: dt_local.hour() as u16,
        wMinute: dt_local.minute() as u16,
        wSecond: dt_local.second() as u16,
        wMilliseconds: (dt_local.nanosecond() / 1_000_000) as u16,
    }
}

/// Converte SYSTEMTIME para DateTime
#[cfg(windows)]
pub(crate) fn systemtime_to_datetime(st: windows::Win32::Foundation::SYSTEMTIME) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(
        st.wYear as i32,
        st.wMonth as u32,
        st.wDay as u32,
        st.wHour as u32,
        st.wMinute as u32,
        st.wSecond as u32,
    ).single().unwrap_or_else(|| Utc::now())
}

#[cfg(not(windows))]
pub(crate) fn datetime_to_systemtime(_dt: DateTime<Utc>) -> () {
    ()
}

#[cfg(not(windows))]
pub(crate) fn systemtime_to_datetime(_st: ()) -> DateTime<Utc> {
    Utc::now()
}

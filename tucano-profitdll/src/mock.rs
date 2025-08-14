//! Implementação mock & tipos compartilhados.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum CallbackEvent {
    StateChanged {
        connection_type: ConnectionState,
        result: i32,
    },
    ProgressChanged {
        ticker: String,
        exchange: String,
        feed_type: i32,
        progress: i32,
    },
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
    PriceBookOffer {
        ticker: String,
        exchange: String,
        action: BookAction,
        price: Decimal,
        position: i32,
    },
    OfferBookBid {
        ticker: String,
        exchange: String,
        action: BookAction,
        price: Decimal,
        position: i32,
    },
    AccountChanged {
        account_id: String,
        account_holder: String,
        broker_name: String,
        broker_id: i32,
    },
    InvalidTicker {
        ticker: String,
        exchange: String,
        feed_type: i32,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    Login = 0,
    Routing = 1,
    MarketData = 2,
    MarketLogin = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum BookAction {
    New = 0,
    Edit = 1,
    Delete = 2,
}

pub type NResult = i32;
pub const NL_OK: NResult = 0;
pub const NL_INTERNAL_ERROR: NResult = -2147483647;
pub const NL_NOT_INITIALIZED: NResult = -2147483646;
pub const NL_INVALID_ARGS: NResult = -2147483645;
pub const NL_WAITING_SERVER: NResult = -2147483644;
pub const NL_NO_LOGIN: NResult = -2147483643;
pub const NL_NO_LICENSE: NResult = -2147483642;

#[derive(Debug)]
pub struct ProfitConnector {
    _connected: bool,
}
impl ProfitConnector {
    pub fn new(_dll_path: Option<&str>) -> Result<Self, String> {
        Ok(Self { _connected: false })
    }
    pub async fn initialize_login(
        &self,
        _activation_key: &str,
        _user: &str,
        _password: &str,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>, String> {
        let (_s, r) = tokio::sync::mpsc::unbounded_channel();
        Ok(r)
    }
    pub fn subscribe_ticker(&self, _ticker: &str, _exchange: &str) -> Result<(), String> {
        Ok(())
    }
    pub fn unsubscribe_ticker(&self, _ticker: &str, _exchange: &str) -> Result<(), String> {
        Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy = 0,
    Sell = 1,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderValidity {
    Day,
    GoodTillCanceled,
    ImmediateOrCancel,
    FillOrKill,
}

#[derive(Debug, thiserror::Error)]
pub enum ProfitError {
    #[error("Conexão falhou: {0}")]
    ConnectionFailed(String),
    #[error("Erro interno: {0}")]
    InternalError(String),
    #[error("Argumentos inválidos: {0}")]
    InvalidArgs(String),
    #[error("Não inicializado")]
    NotInitialized,
    #[error("Sem login")]
    NoLogin,
    #[error("Sem licença")]
    NoLicense,
}

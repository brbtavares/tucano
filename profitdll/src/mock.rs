//! Implementação mock & tipos compartilhados.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::error::*;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver, unbounded_channel};
use std::sync::Mutex;

#[non_exhaustive]
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
    OrderUpdated {
        order_id: i64,
    },
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
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ConnectionState {
    Login = 0,
    Routing = 1,
    MarketData = 2,
    MarketLogin = 3,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum BookAction {
    New = 0,
    Edit = 1,
    Delete = 2,
}


#[derive(Debug)]
pub struct ProfitConnector {
    _connected: bool,
    sender: Mutex<Option<UnboundedSender<CallbackEvent>>>,
}
impl ProfitConnector {
    pub fn new(_dll_path: Option<&str>) -> Result<Self, ProfitError> { Ok(Self { _connected: false, sender: Mutex::new(None) }) }
    pub async fn initialize_login(&self, _activation_key: &str, _user: &str, _password: &str)
        -> Result<UnboundedReceiver<CallbackEvent>, ProfitError>
    {
        let (tx, rx) = unbounded_channel();
        *self.sender.lock().unwrap() = Some(tx);
        Ok(rx)
    }
    pub fn subscribe_ticker(&self, ticker: &str, exchange: &str) -> Result<(), ProfitError> {
        if let Some(tx) = self.sender.lock().unwrap().as_ref() {
            let _ = tx.send(CallbackEvent::ProgressChanged { ticker: ticker.to_string(), exchange: exchange.to_string(), feed_type: 0, progress: 100 });
            let _ = tx.send(CallbackEvent::PriceBookOffer { ticker: ticker.to_string(), exchange: exchange.to_string(), action: BookAction::New, price: Decimal::from(10), position: 0 });
        }
        Ok(())
    }
    pub fn unsubscribe_ticker(&self, _ticker: &str, _exchange: &str) -> Result<(), ProfitError> { Ok(()) }
    pub fn send_order(&self, _order: &SendOrder) -> Result<(), ProfitError> {
        if let Some(tx) = self.sender.lock().unwrap().as_ref() {
            let _ = tx.send(CallbackEvent::OrderUpdated { order_id: 1 });
        }
        Ok(())
    }
    pub fn cancel_order(&self, _order_id: i64) -> Result<(), ProfitError> { Ok(()) }
    pub fn change_order(&self, _order_id: i64, _new_price: Option<Decimal>, _new_qty: Option<Decimal>) -> Result<(), ProfitError> { Ok(()) }
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

#[non_exhaustive]
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

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderValidity {
    Day,
    GoodTillCanceled,
    ImmediateOrCancel,
    FillOrKill,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market = 1,
    Limit = 2,
    StopLimit = 4,
}

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
    pub fn from_i32(v: i32) -> Self { use OrderStatus::*; match v {0=>New,1=>PartiallyFilled,2=>Filled,3=>DoneForDay,4=>Canceled,5=>Replaced,6=>PendingCancel,7=>Stopped,8=>Rejected,9=>Suspended,10=>PendingNew,11=>Calculated,12=>Expired,13=>AcceptedForBidding,14=>PendingReplace,15=>PartiallyFilledCanceled,16=>Received,17=>PartiallyFilledExpired,18=>PartiallyFilledRejected,200=>Unknown,201=>HadesCreated,202=>BrokerSent,203=>ClientCreated,204=>OrderNotCreated,205=>CanceledByAdmin,206=>DelayFixGateway,207=>ScheduledOrder,_=>Unknown} }
}

// Usa ProfitError unificado de crate::error

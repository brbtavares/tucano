use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Binance server time response
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceServerTime {
    #[serde(rename = "serverTime")]
    pub server_time: u64,
}

/// Binance account information
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceAccountInfo {
    #[serde(rename = "makerCommission")]
    pub maker_commission: i32,
    #[serde(rename = "takerCommission")]
    pub taker_commission: i32,
    #[serde(rename = "buyerCommission")]
    pub buyer_commission: i32,
    #[serde(rename = "sellerCommission")]
    pub seller_commission: i32,
    #[serde(rename = "canTrade")]
    pub can_trade: bool,
    #[serde(rename = "canWithdraw")]
    pub can_withdraw: bool,
    #[serde(rename = "canDeposit")]
    pub can_deposit: bool,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub balances: Vec<BinanceBalance>,
    pub permissions: Vec<String>,
}

/// Binance balance information
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

/// Binance order information
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceOrder {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: BinanceOrderStatus,
    #[serde(rename = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    #[serde(rename = "type")]
    pub order_type: BinanceOrderType,
    pub side: BinanceOrderSide,
    #[serde(rename = "stopPrice")]
    pub stop_price: String,
    #[serde(rename = "icebergQty")]
    pub iceberg_qty: String,
    pub time: u64,
    #[serde(rename = "updateTime")]
    pub update_time: u64,
    #[serde(rename = "isWorking")]
    pub is_working: bool,
    #[serde(rename = "origQuoteOrderQty")]
    pub orig_quote_order_qty: String,
}

/// Binance order response from placing an order
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceOrderResponse {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "transactTime")]
    pub transact_time: u64,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: BinanceOrderStatus,
    #[serde(rename = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    #[serde(rename = "type")]
    pub order_type: BinanceOrderType,
    pub side: BinanceOrderSide,
    pub fills: Vec<BinanceOrderFill>,
}

/// Binance order fill information
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceOrderFill {
    pub price: String,
    pub qty: String,
    pub commission: String,
    #[serde(rename = "commissionAsset")]
    pub commission_asset: String,
    #[serde(rename = "tradeId")]
    pub trade_id: u64,
}

/// Binance order cancellation response
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceOrderCancel {
    pub symbol: String,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i64,
    #[serde(rename = "clientOrderId")]
    pub client_order_id: String,
    #[serde(rename = "origClientOrderId")]
    pub orig_client_order_id: String,
    pub price: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    #[serde(rename = "executedQty")]
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: BinanceOrderStatus,
    #[serde(rename = "timeInForce")]
    pub time_in_force: BinanceTimeInForce,
    #[serde(rename = "type")]
    pub order_type: BinanceOrderType,
    pub side: BinanceOrderSide,
}

/// Binance trade information
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BinanceTrade {
    pub symbol: String,
    pub id: u64,
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "orderListId")]
    pub order_list_id: i64,
    pub price: String,
    pub qty: String,
    #[serde(rename = "quoteQty")]
    pub quote_qty: String,
    pub commission: String,
    #[serde(rename = "commissionAsset")]
    pub commission_asset: String,
    pub time: u64,
    #[serde(rename = "isBuyer")]
    pub is_buyer: bool,
    #[serde(rename = "isMaker")]
    pub is_maker: bool,
    #[serde(rename = "isBestMatch")]
    pub is_best_match: bool,
}

/// Binance order status
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum BinanceOrderStatus {
    #[serde(rename = "NEW")]
    New,
    #[serde(rename = "PARTIALLY_FILLED")]
    PartiallyFilled,
    #[serde(rename = "FILLED")]
    Filled,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "PENDING_CANCEL")]
    PendingCancel,
    #[serde(rename = "REJECTED")]
    Rejected,
    #[serde(rename = "EXPIRED")]
    Expired,
}

/// Binance order type
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum BinanceOrderType {
    #[serde(rename = "LIMIT")]
    Limit,
    #[serde(rename = "MARKET")]
    Market,
    #[serde(rename = "STOP_LOSS")]
    StopLoss,
    #[serde(rename = "STOP_LOSS_LIMIT")]
    StopLossLimit,
    #[serde(rename = "TAKE_PROFIT")]
    TakeProfit,
    #[serde(rename = "TAKE_PROFIT_LIMIT")]
    TakeProfitLimit,
    #[serde(rename = "LIMIT_MAKER")]
    LimitMaker,
}

/// Binance order side
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum BinanceOrderSide {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

/// Binance time in force
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum BinanceTimeInForce {
    #[serde(rename = "GTC")]
    GoodTillCanceled,
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    #[serde(rename = "FOK")]
    FillOrKill,
}

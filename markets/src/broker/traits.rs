// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! Core broker traits
//!
//! Defines the essential interfaces that brokers must implement
//! for trading, market data, and account management.

use crate::{Asset, ExchangeId};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Unique identifier for brokers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrokerId {
    ProfitDLL,
    Mock,
    Other(String),
}

/// Broker-specific error types
#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Market data error: {0}")]
    MarketDataError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Core broker trait for connectivity and identification
pub trait Broker {
    fn id(&self) -> BrokerId;
    fn name(&self) -> &'static str;
    fn supported_exchanges(&self) -> Vec<ExchangeId>;
}

/// Market data provider capability
#[async_trait]
pub trait MarketDataProvider: Broker {
    type MarketEvent: Send + Sync;
    type SubscriptionId: Send + Sync;

    async fn connect(&mut self) -> Result<(), BrokerError>;
    async fn disconnect(&mut self) -> Result<(), BrokerError>;

    async fn subscribe_market_data(
        &mut self,
        asset: &(dyn Asset + Send + Sync),
        exchange: ExchangeId,
    ) -> Result<Self::SubscriptionId, BrokerError>;

    async fn unsubscribe_market_data(
        &mut self,
        subscription_id: Self::SubscriptionId,
    ) -> Result<(), BrokerError>;

    async fn next_market_event(&mut self) -> Option<Self::MarketEvent>;
}

/// Order execution capability
#[async_trait]
pub trait OrderExecutor: Broker {
    type OrderId: Send + Sync;
    type ExecutionEvent: Send + Sync;

    async fn connect(&mut self) -> Result<(), BrokerError>;
    async fn disconnect(&mut self) -> Result<(), BrokerError>;

    async fn submit_order(&mut self, order: OrderRequest) -> Result<Self::OrderId, BrokerError>;
    async fn cancel_order(&mut self, order_id: Self::OrderId) -> Result<(), BrokerError>;

    async fn next_execution_event(&mut self) -> Option<Self::ExecutionEvent>;
}

/// Account information provider
#[async_trait]
pub trait AccountProvider: Broker {
    type Balance: Send + Sync;
    type Position: Send + Sync;
    type AccountEvent: Send + Sync;

    async fn connect(&mut self) -> Result<(), BrokerError>;

    async fn get_balances(&self) -> Result<Vec<Self::Balance>, BrokerError>;
    async fn get_positions(&self) -> Result<Vec<Self::Position>, BrokerError>;

    async fn next_account_event(&mut self) -> Option<Self::AccountEvent>;
}

/// Complete broker implementation combining all capabilities
pub trait FullBroker: MarketDataProvider + OrderExecutor + AccountProvider {}

/// Simple order request structure for broker execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub exchange: ExchangeId,
    pub side: OrderSide,
    pub quantity: rust_decimal::Decimal,
    pub price: Option<rust_decimal::Decimal>, // None for market orders
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

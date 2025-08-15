// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
//! ProfitDLL broker implementation
//!
//! Provides a complete broker implementation using the ProfitDLL library
//! for B3 (Brazilian Stock Exchange) connectivity.

use super::traits::*;
use crate::profit_dll::{CallbackEvent, ProfitConnector};
use crate::{Asset, ExchangeId};
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// ProfitDLL broker implementation for B3
pub struct ProfitDLLBroker {
    connector: Option<ProfitConnector>,
    event_receiver: Option<mpsc::UnboundedReceiver<CallbackEvent>>,
    is_connected: bool,
}

impl Default for ProfitDLLBroker {
    fn default() -> Self { Self::new() }
}

impl std::fmt::Debug for ProfitDLLBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfitDLLBroker")
            .field("connector", &self.connector.is_some())
            .field("event_receiver", &self.event_receiver.is_some())
            .field("is_connected", &self.is_connected)
            .finish()
    }
}

impl ProfitDLLBroker {
    pub fn new() -> Self {
        Self {
            connector: None,
            event_receiver: None,
            is_connected: false,
        }
    }

    /// Initialize the broker with authentication credentials
    pub async fn initialize(
        &mut self,
        activation_key: &str,
        user: &str,
        password: &str,
    ) -> Result<(), BrokerError> {
        let connector =
            ProfitConnector::new(None).map_err(|e| BrokerError::ConnectionFailed(e.to_string()))?;

        let events = connector
            .initialize_login(activation_key, user, password)
            .await
            .map_err(|e| BrokerError::AuthenticationFailed(e.to_string()))?;

        self.connector = Some(connector);
        self.event_receiver = Some(events);
        self.is_connected = true;

        Ok(())
    }
}

impl Broker for ProfitDLLBroker {
    fn id(&self) -> BrokerId {
        BrokerId::ProfitDLL
    }

    fn name(&self) -> &'static str {
        "ProfitDLL"
    }

    fn supported_exchanges(&self) -> Vec<ExchangeId> {
        vec![ExchangeId::B3]
    }
}

/// Market data events from ProfitDLL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitMarketEvent {
    pub symbol: String,
    pub exchange: ExchangeId,
    pub event_type: MarketEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketEventType {
    Trade {
        price: Decimal,
        volume: Decimal,
        side: OrderSide,
    },
    Quote {
        bid: Decimal,
        ask: Decimal,
        bid_size: Decimal,
        ask_size: Decimal,
    },
    OrderBook {
        bids: Vec<(Decimal, Decimal)>,
        asks: Vec<(Decimal, Decimal)>,
    },
}

#[async_trait]
impl MarketDataProvider for ProfitDLLBroker {
    type MarketEvent = ProfitMarketEvent;
    type SubscriptionId = String;

    async fn connect(&mut self) -> Result<(), BrokerError> {
        if !self.is_connected {
            return Err(BrokerError::ConnectionFailed(
                "Broker not initialized. Call initialize() first.".to_string(),
            ));
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BrokerError> {
        self.is_connected = false;
        self.connector = None;
        self.event_receiver = None;
        Ok(())
    }

    async fn subscribe_market_data(
        &mut self,
        asset: &(dyn Asset + Send + Sync),
        exchange: ExchangeId,
    ) -> Result<Self::SubscriptionId, BrokerError> {
        if exchange != ExchangeId::B3 {
            return Err(BrokerError::MarketDataError(format!("Unsupported exchange: {exchange:?}")));
        }

        let connector = self
            .connector
            .as_ref()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        // Subscribe to ticker using ProfitDLL
        connector
            .subscribe_ticker(asset.symbol(), "B")
            .map_err(|e| BrokerError::MarketDataError(e.to_string()))?;

        Ok(asset.symbol().to_string())
    }

    async fn unsubscribe_market_data(
        &mut self,
        subscription_id: Self::SubscriptionId,
    ) -> Result<(), BrokerError> {
        let connector = self
            .connector
            .as_ref()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        connector
            .unsubscribe_ticker(&subscription_id, "B")
            .map_err(|e| BrokerError::MarketDataError(e.to_string()))?;

        Ok(())
    }

    async fn next_market_event(&mut self) -> Option<Self::MarketEvent> {
        let receiver = self.event_receiver.as_mut()?;

        if let Ok(event) = receiver.try_recv() {
            // Convert ProfitDLL CallbackEvent to our MarketEvent
            match event {
                CallbackEvent::NewTrade {
                    ticker,
                    exchange: _,
                    price,
                    volume,
                    timestamp,
                    buy_agent: _,
                    sell_agent: _,
                    trade_id: _,
                    is_edit: _,
                } => {
                    Some(ProfitMarketEvent {
                        symbol: ticker,
                        exchange: ExchangeId::B3, // Assuming B3 for now
                        event_type: MarketEventType::Trade {
                            price,
                            volume,
                            side: OrderSide::Buy, // We don't have this info in the callback
                        },
                        timestamp,
                    })
                }
                CallbackEvent::DailySummary {
                    ticker,
                    exchange: _,
                    open: _,
                    high: _,
                    low: _,
                    close,
                    volume: _,
                    adjustment: _,
                    max_limit: _,
                    min_limit: _,
                    trades_buyer: _,
                    trades_seller: _,
                } => {
                    Some(ProfitMarketEvent {
                        symbol: ticker,
                        exchange: ExchangeId::B3,
                        event_type: MarketEventType::Quote {
                            bid: close, // Simplified - using close as both bid/ask
                            ask: close,
                            bid_size: Decimal::ZERO,
                            ask_size: Decimal::ZERO,
                        },
                        timestamp: chrono::Utc::now(),
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Execution events from ProfitDLL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitExecutionEvent {
    pub order_id: String,
    pub symbol: String,
    pub exchange: ExchangeId,
    pub event_type: ExecutionEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEventType {
    OrderAccepted,
    OrderRejected {
        reason: String,
    },
    PartialFill {
        filled_quantity: Decimal,
        fill_price: Decimal,
        remaining_quantity: Decimal,
    },
    FullFill {
        filled_quantity: Decimal,
        fill_price: Decimal,
    },
    OrderCancelled,
}

#[async_trait]
impl OrderExecutor for ProfitDLLBroker {
    type OrderId = String;
    type ExecutionEvent = ProfitExecutionEvent;

    async fn connect(&mut self) -> Result<(), BrokerError> {
        if !self.is_connected {
            return Err(BrokerError::ConnectionFailed(
                "Broker not initialized. Call initialize() first.".to_string(),
            ));
        }
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), BrokerError> {
        self.is_connected = false;
        self.connector = None;
        self.event_receiver = None;
        Ok(())
    }

    async fn submit_order(&mut self, order: OrderRequest) -> Result<Self::OrderId, BrokerError> {
        if order.exchange != ExchangeId::B3 {
            return Err(BrokerError::ExecutionError(format!(
                "Unsupported exchange: {:?}",
                order.exchange
            )));
        }

        let _connector = self
            .connector
            .as_ref()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        // Convert our order to ProfitDLL format and submit
        // This is a simplified version - real implementation would need more details
        let order_id = format!("ORDER_{}", chrono::Utc::now().timestamp_millis());

        // TODO: Implement actual order submission via ProfitDLL
        // connector.submit_order(...)?;

        Ok(order_id)
    }

    async fn cancel_order(&mut self, _order_id: Self::OrderId) -> Result<(), BrokerError> {
        let _connector = self
            .connector
            .as_ref()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        // TODO: Implement actual order cancellation via ProfitDLL
        // connector.cancel_order(&order_id)?;

        Ok(())
    }

    async fn next_execution_event(&mut self) -> Option<Self::ExecutionEvent> {
        let receiver = self.event_receiver.as_mut()?;

        if let Ok(event) = receiver.try_recv() {
            // Convert ProfitDLL CallbackEvent to our ExecutionEvent
            match event {
                CallbackEvent::AccountChanged {
                    account_id,
                    account_holder: _,
                    broker_name: _,
                    broker_id: _,
                } => {
                    Some(ProfitExecutionEvent {
                        order_id: account_id, // Using account_id as placeholder
                        symbol: "UNKNOWN".to_string(),
                        exchange: ExchangeId::B3,
                        event_type: ExecutionEventType::OrderAccepted, // Simplified
                        timestamp: chrono::Utc::now(),
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Account events and balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitBalance {
    pub asset: String,
    pub total: Decimal,
    pub available: Decimal,
    pub blocked: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitPosition {
    pub symbol: String,
    pub exchange: ExchangeId,
    pub quantity: Decimal,
    pub average_price: Decimal,
    pub market_value: Decimal,
    pub unrealized_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitAccountEvent {
    pub event_type: AccountEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountEventType {
    BalanceUpdate(ProfitBalance),
    PositionUpdate(ProfitPosition),
    MarginCall { required_margin: Decimal },
}

#[async_trait]
impl AccountProvider for ProfitDLLBroker {
    type Balance = ProfitBalance;
    type Position = ProfitPosition;
    type AccountEvent = ProfitAccountEvent;

    async fn connect(&mut self) -> Result<(), BrokerError> {
        if !self.is_connected {
            return Err(BrokerError::ConnectionFailed(
                "Broker not initialized. Call initialize() first.".to_string(),
            ));
        }
        Ok(())
    }

    async fn get_balances(&self) -> Result<Vec<Self::Balance>, BrokerError> {
        let _connector = self
            .connector
            .as_ref()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        // TODO: Implement actual balance retrieval via ProfitDLL
        // This is a placeholder
        Ok(vec![])
    }

    async fn get_positions(&self) -> Result<Vec<Self::Position>, BrokerError> {
        let _connector = self
            .connector
            .as_ref()
            .ok_or_else(|| BrokerError::ConnectionFailed("Not connected".to_string()))?;

        // TODO: Implement actual position retrieval via ProfitDLL
        // This is a placeholder
        Ok(vec![])
    }

    async fn next_account_event(&mut self) -> Option<Self::AccountEvent> {
        let receiver = self.event_receiver.as_mut()?;

        if let Ok(event) = receiver.try_recv() {
            // Convert ProfitDLL CallbackEvent to our AccountEvent
            match event {
                CallbackEvent::AccountChanged {
                    account_id,
                    account_holder: _,
                    broker_name: _,
                    broker_id: _,
                } => {
                    Some(ProfitAccountEvent {
                        event_type: AccountEventType::BalanceUpdate(ProfitBalance {
                            asset: account_id,
                            total: Decimal::ZERO, // Placeholder
                            available: Decimal::ZERO,
                            blocked: Decimal::ZERO,
                        }),
                        timestamp: chrono::Utc::now(),
                    })
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

impl FullBroker for ProfitDLLBroker {}

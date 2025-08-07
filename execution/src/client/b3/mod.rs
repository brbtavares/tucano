//! B3 execution client implementation using ProfitDLL
//!
//! This module provides the B3ExecutionClient that integrates with the Brazilian
//! stock exchange through the ProfitDLL library, implementing the ExecutionClient trait
//! for full compatibility with the Toucan framework.

pub mod adapter;

use crate::{
    UnindexedAccountEvent, UnindexedAccountSnapshot, InstrumentAccountSnapshot,
    client::ExecutionClient,
    error::{UnindexedClientError, UnindexedOrderError},
    order::{
        request::{OrderRequestCancel, OrderRequestOpen, UnindexedOrderResponseCancel},
        state::Open,
        Order, OrderKey, OrderKind,
        id::{OrderId},
    },
    balance::AssetBalance,
    trade::Trade,
};
use markets::{
    Side, Exchange, Instrument as InstrumentTrait, Asset,
    asset::name::AssetNameExchange,
    exchange::ExchangeId,
    instrument::name::InstrumentNameExchange,
};
use profit_dll::{ProfitConnector, SendOrder, OrderSide, ProfitError};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use smol_str::SmolStr;

/// Configuration for B3 execution client
#[derive(Debug, Clone)]
pub struct B3Config {
    /// DLL path override (optional)
    pub dll_path: Option<String>,
    /// API credentials
    pub activation_key: String,
    pub username: String,
    pub password: String,
    /// Connection settings
    pub auto_reconnect: bool,
    pub connection_timeout_secs: u64,
}

impl B3Config {
    pub fn new(activation_key: String, username: String, password: String) -> Self {
        Self {
            dll_path: None,
            activation_key,
            username,
            password,
            auto_reconnect: true,
            connection_timeout_secs: 30,
        }
    }

    pub fn with_dll_path(mut self, dll_path: String) -> Self {
        self.dll_path = Some(dll_path);
        self
    }

    pub fn with_auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
        self
    }

    pub fn with_connection_timeout(mut self, timeout_secs: u64) -> Self {
        self.connection_timeout_secs = timeout_secs;
        self
    }
}

/// B3 execution client using ProfitDLL
pub struct B3ExecutionClient {
    config: B3Config,
    connector: Arc<Mutex<Option<ProfitConnector>>>,
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<UnindexedAccountEvent>>>>,
}

impl Clone for B3ExecutionClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connector: self.connector.clone(),
            event_sender: self.event_sender.clone(),
        }
    }
}

impl std::fmt::Debug for B3ExecutionClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("B3ExecutionClient")
            .field("config", &self.config)
            .finish()
    }
}

impl ExecutionClient for B3ExecutionClient {
    const EXCHANGE: ExchangeId = ExchangeId::B3;

    type Config = B3Config;
    type AccountStream = UnboundedReceiverStream<UnindexedAccountEvent>;

    fn new(config: Self::Config) -> Self {
        Self {
            config,
            connector: Arc::new(Mutex::new(None)),
            event_sender: Arc::new(Mutex::new(None)),
        }
    }

    async fn fetch_balances(
        &self,
    ) -> Result<Vec<AssetBalance<AssetNameExchange>>, UnindexedClientError> {
        // Ensure connection is established
        self.ensure_connected().await?;
        
        // Return empty balances for now - in real implementation, 
        // this would query ProfitDLL for current balances
        Ok(Vec::new())
    }

    async fn fetch_open_orders(
        &self,
    ) -> Result<Vec<Order<ExchangeId, InstrumentNameExchange, Open>>, UnindexedClientError> {
        // Ensure connection is established
        self.ensure_connected().await?;
        
        // Return empty orders for now - in real implementation,
        // this would query ProfitDLL for current open orders
        Ok(Vec::new())
    }

    async fn fetch_trades(
        &self,
        _time_since: DateTime<Utc>,
    ) -> Result<Vec<Trade<QuoteAsset, InstrumentNameExchange>>, UnindexedClientError> {
        // Ensure connection is established
        self.ensure_connected().await?;
        
        // Return empty trades for now - in real implementation,
        // this would query ProfitDLL for trades since the specified time
        Ok(Vec::new())
    }

    async fn account_snapshot(
        &self,
        _assets: &[AssetNameExchange],
        instruments: &[InstrumentNameExchange],
    ) -> Result<UnindexedAccountSnapshot, UnindexedClientError> {
        // Ensure connection is established
        self.ensure_connected().await?;

        // Fetch current data
        let balances = self.fetch_balances().await?;
        let orders = self.fetch_open_orders().await?;

        // Group orders by instrument
        let instrument_snapshots: Vec<InstrumentAccountSnapshot<ExchangeId, AssetNameExchange, InstrumentNameExchange>> = 
            instruments.iter().map(|instrument| {
                let instrument_orders = orders
                    .iter()
                    .filter(|order| &order.key.instrument == instrument)
                    .map(|order| order.clone().into()) // Convert to OrderSnapshot
                    .collect();

                InstrumentAccountSnapshot::new(instrument.clone(), instrument_orders)
            }).collect();

        Ok(UnindexedAccountSnapshot::new(
            ExchangeId::B3,
            balances,
            instrument_snapshots,
        ))
    }

    async fn account_stream(
        &self,
        _assets: &[AssetNameExchange],
        _instruments: &[InstrumentNameExchange],
    ) -> Result<Self::AccountStream, UnindexedClientError> {
        // Ensure connection is established
        self.ensure_connected().await?;

        let (tx, rx) = mpsc::unbounded_channel();
        
        // Store the sender for later use
        {
            let mut sender_guard = self.event_sender.lock().await;
            *sender_guard = Some(tx);
        }

        // Start processing events from ProfitDLL
        self.start_event_processing().await?;

        Ok(UnboundedReceiverStream::new(rx))
    }

    async fn cancel_order(
        &self,
        _request: OrderRequestCancel<ExchangeId, &InstrumentNameExchange>,
    ) -> Option<UnindexedOrderResponseCancel> {
        match self.ensure_connected().await {
            Ok(_) => {
                // Implementation would call ProfitDLL cancel order function
                // For now, return None indicating failure
                tracing::warn!("B3 order cancellation not yet implemented");
                None
            }
            Err(e) => {
                tracing::error!("Failed to connect to B3: {:?}", e);
                None
            }
        }
    }

    async fn open_order(
        &self,
        request: OrderRequestOpen<ExchangeId, &InstrumentNameExchange>,
    ) -> Option<Order<ExchangeId, InstrumentNameExchange, Result<Open, UnindexedOrderError>>> {
        match self.ensure_connected().await {
            Ok(_) => {
                match self.send_order_to_b3(&request).await {
                    Ok(order) => Some(order),
                    Err(e) => {
                        tracing::error!("Failed to send order to B3: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to connect to B3: {:?}", e);
                None
            }
        }
    }
}

impl B3ExecutionClient {
    /// Ensure connection to B3 is established
    async fn ensure_connected(&self) -> Result<(), UnindexedClientError> {
        let mut connector_guard = self.connector.lock().await;
        
        if connector_guard.is_none() {
            // Create new connection
            let connector = ProfitConnector::new(self.config.dll_path.as_deref())
                .map_err(|e| UnindexedClientError::AccountSnapshot(e.to_string()))?;

            // Initialize login
            let _events = connector
                .initialize_login(
                    &self.config.activation_key,
                    &self.config.username,
                    &self.config.password,
                )
                .await
                .map_err(|e| UnindexedClientError::AccountSnapshot(e.to_string()))?;

            *connector_guard = Some(connector);
        }

        Ok(())
    }

    /// Start processing events from ProfitDLL and convert them to Toucan events
    async fn start_event_processing(&self) -> Result<(), UnindexedClientError> {
        // This would start a background task to process ProfitDLL events
        // and convert them to Toucan AccountEvents
        tracing::info!("Starting B3 event processing");
        Ok(())
    }

    /// Send order to B3 through ProfitDLL
    async fn send_order_to_b3(
        &self,
        request: &OrderRequestOpen<ExchangeId, &InstrumentNameExchange>,
    ) -> Result<Order<ExchangeId, InstrumentNameExchange, Result<Open, UnindexedOrderError>>, ProfitError> {
        let connector_guard = self.connector.lock().await;
        let _connector = connector_guard.as_ref().ok_or_else(|| {
            ProfitError::ConnectionFailure("No active connection to B3".to_string())
        })?;

        // Convert Toucan order request to ProfitDLL order
        let _profit_order = self.convert_to_profit_order(request)?;

        // Send order through ProfitDLL
        // This is a placeholder - actual implementation would call connector.send_order()
        let instrument_str = format!("{:?}", request.key.instrument); // Convert to string
        tracing::info!("Sending order to B3: instrument={}", instrument_str);

        // Create the order structure with all required fields
        let order = Order {
            key: OrderKey {
                exchange: ExchangeId::B3,
                instrument: (*request.key.instrument).clone(),
                strategy: request.key.strategy.clone(),
                cid: request.key.cid.clone(),
            },
            side: request.state.side,
            price: request.state.price,
            quantity: request.state.quantity,
            kind: request.state.kind,
            time_in_force: request.state.time_in_force,
            state: Ok(Open::new(
                OrderId::from(SmolStr::new("B3_ORDER_123")),
                chrono::Utc::now(),
                Decimal::ZERO,
            )),
        };

        Ok(order)
    }

    /// Convert Toucan order request to ProfitDLL SendOrder
    fn convert_to_profit_order(
        &self,
        request: &OrderRequestOpen<ExchangeId, &InstrumentNameExchange>,
    ) -> Result<SendOrder, ProfitError> {
        // This is a simplified conversion
        // Real implementation would need proper account/asset mapping
        
        use profit_dll::{AssetIdentifier, AccountIdentifier};

        let account = AccountIdentifier::new(
            1, // placeholder broker_id
            "default".to_string(), // placeholder account_id
            "".to_string(), // sub_account_id
        );

        let instrument_str = format!("{:?}", request.key.instrument);
        let asset = AssetIdentifier::new(
            &instrument_str, // ticker
            "BOVESPA", // exchange (B3)
            0, // feed_type
        );

        let order_side = match request.state.side {
            Side::Buy => OrderSide::Buy,
            Side::Sell => OrderSide::Sell,
        };

        let order = match request.state.kind {
            OrderKind::Market => {
                SendOrder::new_market_order(
                    account,
                    asset,
                    "".to_string(), // password - should come from config
                    order_side,
                    request.state.quantity.to_string().parse().unwrap_or(0.0) as i64,
                )
            }
            OrderKind::Limit => {
                SendOrder::new_limit_order(
                    account,
                    asset,
                    "".to_string(), // password
                    order_side,
                    request.state.quantity.to_string().parse().unwrap_or(0.0) as f64,
                    request.state.price.to_string().parse().unwrap_or(0.0) as i64,
                )
            }
        };

        Ok(order)
    }
}

// Re-export for easier access
pub use B3ExecutionClient as B3Client;

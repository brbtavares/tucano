//! B3 execution client implementation using ProfitDLL
//!
//! This module provides the B3ExecutionClient that integrates with the Brazilian
//! stock exchange through the ProfitDLL library, implementing the ExecutionClient trait
//! for full compatibility with the Toucan framework.

pub mod adapter;

use crate::{
    balance::AssetBalance,
    client::ExecutionClient,
    compat::QuoteAsset,
    error::{AssetNameExchange, InstrumentNameExchange, UnindexedClientError, UnindexedOrderError},
    order::{
        id::OrderId,
        request::{OrderRequestCancel, OrderRequestOpen, UnindexedOrderResponseCancel},
        state::Open,
        Order, OrderKey, OrderKind,
    },
    trade::Trade,
    transport::{Transport, TransportInstrument, TransportSide, TransportOrderKind, TransportTimeInForce, TransportAccountId, ProfitDLLTransport, TransportEvent},
    InstrumentAccountSnapshot, UnindexedAccountEvent, UnindexedAccountSnapshot,
};
use chrono::{DateTime, Utc};
use markets::{ExchangeId, ProfitError, Side};
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;

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
#[derive(Debug, Clone)]
struct StoredOrderCtx {
    instrument: String,
    side: Side,
    price: Decimal,
    quantity: Decimal,
    kind: OrderKind,
    tif: crate::order::TimeInForce,
}

pub struct B3ExecutionClient {
    config: B3Config,
    transport: Arc<dyn Transport>,
    event_sender: Arc<Mutex<Option<mpsc::UnboundedSender<UnindexedAccountEvent>>>>,
    order_ctx: Arc<Mutex<HashMap<String, StoredOrderCtx>>>,
}

impl Clone for B3ExecutionClient {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            transport: self.transport.clone(),
            event_sender: self.event_sender.clone(),
            order_ctx: self.order_ctx.clone(),
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
        // For now, always use ProfitDLLTransport; later allow injection
        let transport = ProfitDLLTransport::new(
            config.dll_path.clone(),
            config.activation_key.clone(),
            config.username.clone(),
            config.password.clone(),
        );
    Self { config, transport: Arc::new(transport), event_sender: Arc::new(Mutex::new(None)), order_ctx: Arc::new(Mutex::new(HashMap::new())) }
    }

    async fn fetch_balances(
        &self,
    ) -> Result<Vec<AssetBalance<AssetNameExchange>>, UnindexedClientError> {
    self.ensure_connected().await?;

        // Return empty balances for now - in real implementation,
        // this would query ProfitDLL for current balances
        Ok(Vec::new())
    }

    async fn fetch_open_orders(
        &self,
    ) -> Result<Vec<Order<ExchangeId, InstrumentNameExchange, Open>>, UnindexedClientError> {
    self.ensure_connected().await?;

        // Return empty orders for now - in real implementation,
        // this would query ProfitDLL for current open orders
        Ok(Vec::new())
    }

    async fn fetch_trades(
        &self,
        _time_since: DateTime<Utc>,
    ) -> Result<Vec<Trade<QuoteAsset, InstrumentNameExchange>>, UnindexedClientError> {
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
    self.ensure_connected().await?;

        // Fetch current data
        let balances = self.fetch_balances().await?;
        let orders = self.fetch_open_orders().await?;

        // Group orders by instrument
        let instrument_snapshots: Vec<
            InstrumentAccountSnapshot<ExchangeId, AssetNameExchange, InstrumentNameExchange>,
        > = instruments
            .iter()
            .map(|instrument| {
                let instrument_orders = orders
                    .iter()
                    .filter(|order| &order.key.instrument == instrument)
                    .map(|order| order.clone().into()) // Convert to OrderSnapshot
                    .collect();

                InstrumentAccountSnapshot::new(instrument.clone(), instrument_orders)
            })
            .collect();

        Ok(UnindexedAccountSnapshot::new(
            ExchangeId::B3,
            None,
            None,
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
            Ok(_) => match self.send_order_to_b3(&request).await {
                Ok(order) => Some(order),
                Err(e) => {
                    tracing::error!("Failed to send order to B3: {:?}", e);
                    None
                }
            },
            Err(e) => {
                tracing::error!("Failed to connect to B3: {:?}", e);
                None
            }
        }
    }
}

impl B3ExecutionClient {
    /// Ensure connection to B3 is established
    async fn ensure_connected(&self) -> Result<(), UnindexedClientError> { self.transport.connect().await.map_err(|e| UnindexedClientError::AccountSnapshot(e.to_string())) }

    /// Start processing events from ProfitDLL and convert them to Toucan events
    async fn start_event_processing(&self) -> Result<(), UnindexedClientError> {
        // This would start a background task to process ProfitDLL events
        // and convert them to Toucan AccountEvents
        tracing::info!("Starting B3 event processing");
        let mut rx = self.transport.account_events().await.map_err(|e| UnindexedClientError::AccountStream(e.to_string()))?;
        let sender_holder = self.event_sender.clone();
        let order_ctx = self.order_ctx.clone();
    tokio::spawn(async move {
            use crate::order::{state::Open, id::{StrategyId, ClientOrderId}};
            use integration::snapshot::Snapshot;
            while let Some(evt) = rx.recv().await {
        if let TransportEvent::OrderAccepted { client_cid, id: _ } = evt {
                    if let Some(tx) = sender_holder.lock().await.as_ref() {
                        let ctx_opt = { order_ctx.lock().await.get(&client_cid).cloned() };
                        if let Some(ctx) = ctx_opt {
                            let order: Order<ExchangeId, String, crate::order::state::OrderState<String, String>> = Order {
                                key: OrderKey { exchange: ExchangeId::B3, instrument: ctx.instrument.clone(), strategy: StrategyId(SmolStr::new_inline("default")), cid: ClientOrderId(SmolStr::from(client_cid.clone())) },
                                side: ctx.side,
                                price: ctx.price,
                                quantity: ctx.quantity,
                                kind: ctx.kind,
                                time_in_force: ctx.tif,
                                state: crate::order::state::OrderState::Active(
                                    crate::order::state::ActiveOrderState::Open(
                                        Open::new(OrderId::from(SmolStr::new_inline("TEMP")), chrono::Utc::now(), Decimal::ZERO)
                                    )
                                ),
                            };
                            let snapshot = Snapshot(order);
                            let event = crate::AccountEvent { exchange: ExchangeId::B3, broker: Some("ProfitDLL".to_string()), account: Some("default".to_string()), kind: crate::AccountEventKind::OrderSnapshot(snapshot) };
                            let _ = tx.send(event);
                        } else {
                            tracing::warn!(cid=%client_cid, "OrderAccepted sem contexto armazenado");
                        }
                    }
                }
            }
        });
        Ok(())
    }

    /// Send order to B3 through ProfitDLL
    async fn send_order_to_b3(
        &self,
        request: &OrderRequestOpen<ExchangeId, &InstrumentNameExchange>,
    ) -> Result<
        Order<ExchangeId, InstrumentNameExchange, Result<Open, UnindexedOrderError>>,
        ProfitError,
    > {
        // Convert request to transport invocation
        let instrument_str = format!("{:?}", request.key.instrument);
        let instrument = TransportInstrument::new(instrument_str.clone(), "B3");
        let side = match request.state.side { Side::Buy => TransportSide::Buy, Side::Sell => TransportSide::Sell };
        let kind = match request.state.kind { OrderKind::Market => TransportOrderKind::Market, OrderKind::Limit => TransportOrderKind::Limit };
        // Map internal TimeInForce to transport variant
        let tif = match request.state.time_in_force {
            crate::order::TimeInForce::GoodUntilCancelled { .. } => TransportTimeInForce::GTC,
            crate::order::TimeInForce::GoodUntilEndOfDay => TransportTimeInForce::Day,
            crate::order::TimeInForce::ImmediateOrCancel => TransportTimeInForce::IOC,
            crate::order::TimeInForce::FillOrKill => TransportTimeInForce::FOK,
        };
        let account = TransportAccountId::new("default", "ProfitDLL");
    let _opened = self.transport.open_order(&instrument, side, request.state.quantity, kind, Some(request.state.price), tif, request.key.cid.0.as_str(), &account)
            .await
            .map_err(|e| ProfitError::ConnectionFailed(e.to_string()))?;
        // Store context for later event reconciliation
        {
            self.order_ctx.lock().await.insert(
                request.key.cid.0.to_string(),
                StoredOrderCtx {
                    instrument: (*request.key.instrument).clone(),
                    side: request.state.side,
                    price: request.state.price,
                    quantity: request.state.quantity,
                    kind: request.state.kind,
                    tif: request.state.time_in_force,
                },
            );
        }
        tracing::info!("Sending order via transport: instrument={}", instrument.symbol);
        // Build internal order representation
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

}

// Re-export for easier access
pub use B3ExecutionClient as B3Client;

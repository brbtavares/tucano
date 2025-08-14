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
    transport::{
        MockTransport, Transport, TransportAccountId, TransportEvent, TransportInstrument,
        TransportOrderKind, TransportSide, TransportTimeInForce,
    },
    InstrumentAccountSnapshot, UnindexedAccountEvent, UnindexedAccountSnapshot,
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tucano_markets::{ExchangeId, Side};
use tucano_profitdll::ProfitError;
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
    /// Broker identifier (Phase 1 multi-broker scaffold)
    pub broker_id: String,
    /// Account identifier within broker
    pub account_id: String,
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
            broker_id: "ProfitDLL".to_string(),
            account_id: "default".to_string(),
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

    pub fn with_broker_id(mut self, broker_id: impl Into<String>) -> Self {
        self.broker_id = broker_id.into();
        self
    }

    pub fn with_account_id(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = account_id.into();
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
    strategy: crate::order::id::StrategyId,
    order_id: Option<OrderId>,
    filled_qty: Decimal,
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
        // ProfitDLL transport extracted; using MockTransport placeholder.
        let transport = MockTransport::default();
        Self {
            config,
            transport: Arc::new(transport),
            event_sender: Arc::new(Mutex::new(None)),
            order_ctx: Arc::new(Mutex::new(HashMap::new())),
        }
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
    async fn ensure_connected(&self) -> Result<(), UnindexedClientError> {
        self.transport
            .connect()
            .await
            .map_err(|e| UnindexedClientError::AccountSnapshot(e.to_string()))
    }

    /// Start processing events from ProfitDLL and convert them to Toucan events
    async fn start_event_processing(&self) -> Result<(), UnindexedClientError> {
        // This would start a background task to process ProfitDLL events
        // and convert them to Toucan AccountEvents
        tracing::info!("Starting B3 event processing");
        let mut rx = self
            .transport
            .account_events()
            .await
            .map_err(|e| UnindexedClientError::AccountStream(e.to_string()))?;
        let sender_holder = self.event_sender.clone();
        let order_ctx = self.order_ctx.clone();
        let broker = self.config.broker_id.clone();
        let account = self.config.account_id.clone();
        tokio::spawn(async move {
            use crate::order::{id::ClientOrderId, state::Open};
            use crate::trade::{AssetFees, Trade, TradeId};
            use tucano_integration::snapshot::Snapshot; // needed for constructing OrderSnapshot events
            while let Some(evt) = rx.recv().await {
                match evt {
                    TransportEvent::OrderAccepted { client_cid, id } => {
                        if let Some(tx) = sender_holder.lock().await.as_ref() {
                            // update context with order id
                            let mut guard = order_ctx.lock().await;
                            if let Some(ctx) = guard.get_mut(&client_cid) {
                                let internal_id = OrderId::from(SmolStr::from(id.0.clone()));
                                ctx.order_id = Some(internal_id.clone());
                                let order: Order<
                                    ExchangeId,
                                    String,
                                    crate::order::state::OrderState<String, String>,
                                > = Order {
                                    key: OrderKey {
                                        exchange: ExchangeId::B3,
                                        instrument: ctx.instrument.clone(),
                                        strategy: ctx.strategy.clone(),
                                        cid: ClientOrderId(SmolStr::from(client_cid.clone())),
                                    },
                                    side: ctx.side,
                                    price: ctx.price,
                                    quantity: ctx.quantity,
                                    kind: ctx.kind,
                                    time_in_force: ctx.tif,
                                    state: crate::order::state::OrderState::active(Open::new(
                                        internal_id,
                                        chrono::Utc::now(),
                                        Decimal::ZERO,
                                    )),
                                };
                                let snapshot = Snapshot(order);
                                let event = crate::AccountEvent {
                                    exchange: ExchangeId::B3,
                                    broker: Some(broker.clone()),
                                    account: Some(account.clone()),
                                    kind: crate::AccountEventKind::OrderSnapshot(snapshot),
                                };
                                let _ = tx.send(event);
                            } else {
                                tracing::warn!(cid=%client_cid, "OrderAccepted sem contexto armazenado");
                            }
                        }
                    }
                    TransportEvent::Trade {
                        order_id,
                        price,
                        quantity,
                        fees,
                        time,
                    } => {
                        if let Some(tx) = sender_holder.lock().await.as_ref() {
                            let mut guard = order_ctx.lock().await;
                            if let Some((found_cid, ctx)) = guard.iter_mut().find(|(_cid, c)| {
                                c.order_id.as_ref().map(|id| id.as_str())
                                    == Some(order_id.0.as_str())
                            }) {
                                // Update filled quantity
                                ctx.filled_qty += quantity;
                                // Emit trade event first
                                let trade = Trade::new(
                                    TradeId::new(format!(
                                        "TRADE-{}-{}",
                                        order_id.0,
                                        time.timestamp()
                                    )),
                                    ctx.order_id.clone().unwrap_or_else(|| {
                                        OrderId::from(SmolStr::new_inline("UNKNOWN"))
                                    }),
                                    ctx.instrument.clone(),
                                    ctx.strategy.clone(),
                                    time,
                                    ctx.side,
                                    price,
                                    quantity,
                                    AssetFees::quote_fees(fees),
                                );
                                let trade_event = crate::AccountEvent {
                                    exchange: ExchangeId::B3,
                                    broker: Some(broker.clone()),
                                    account: Some(account.clone()),
                                    kind: crate::AccountEventKind::Trade(trade),
                                };
                                let _ = tx.send(trade_event);
                                // Emit updated order snapshot with accumulated filled quantity
                                if let Some(actual_order_id) = ctx.order_id.clone() {
                                    let open = Open::new(actual_order_id, time, ctx.filled_qty);
                                    let order: Order<
                                        ExchangeId,
                                        String,
                                        crate::order::state::OrderState<String, String>,
                                    > = Order {
                                        key: OrderKey {
                                            exchange: ExchangeId::B3,
                                            instrument: ctx.instrument.clone(),
                                            strategy: ctx.strategy.clone(),
                                            cid: ClientOrderId(SmolStr::from(found_cid.clone())),
                                        },
                                        side: ctx.side,
                                        price: ctx.price,
                                        quantity: ctx.quantity,
                                        kind: ctx.kind,
                                        time_in_force: ctx.tif,
                                        state: crate::order::state::OrderState::active(open),
                                    };
                                    let snapshot = Snapshot(order);
                                    let snapshot_event = crate::AccountEvent {
                                        exchange: ExchangeId::B3,
                                        broker: Some(broker.clone()),
                                        account: Some(account.clone()),
                                        kind: crate::AccountEventKind::OrderSnapshot(snapshot),
                                    };
                                    let _ = tx.send(snapshot_event);
                                }
                            } else {
                                tracing::warn!(order_id=%order_id.0, "Trade sem contexto correspondente");
                            }
                        }
                    }
                    TransportEvent::OrderRejected { client_cid, reason } => {
                        if let Some(tx) = sender_holder.lock().await.as_ref() {
                            let guard = order_ctx.lock().await;
                            if let Some(ctx) = guard.get(&client_cid) {
                                use crate::error::{ApiError, OrderError};
                                use crate::order::state::InactiveOrderState;
                                let failed_state = crate::order::state::OrderState::inactive(
                                    InactiveOrderState::OpenFailed(OrderError::Rejected(
                                        ApiError::OrderRejected(reason.clone()),
                                    )),
                                );
                                let order: Order<
                                    ExchangeId,
                                    String,
                                    crate::order::state::OrderState<String, String>,
                                > = Order {
                                    key: OrderKey {
                                        exchange: ExchangeId::B3,
                                        instrument: ctx.instrument.clone(),
                                        strategy: ctx.strategy.clone(),
                                        cid: ClientOrderId(SmolStr::from(client_cid.clone())),
                                    },
                                    side: ctx.side,
                                    price: ctx.price,
                                    quantity: ctx.quantity,
                                    kind: ctx.kind,
                                    time_in_force: ctx.tif,
                                    state: failed_state,
                                };
                                let snapshot = Snapshot(order);
                                let event = crate::AccountEvent {
                                    exchange: ExchangeId::B3,
                                    broker: Some(broker.clone()),
                                    account: Some(account.clone()),
                                    kind: crate::AccountEventKind::OrderSnapshot(snapshot),
                                };
                                let _ = tx.send(event);
                            } else {
                                tracing::warn!(cid=%client_cid, "OrderRejected sem contexto armazenado");
                            }
                        }
                    }
                    TransportEvent::OrderCancelled {
                        order_id,
                        client_cid,
                        time,
                    } => {
                        if let Some(tx) = sender_holder.lock().await.as_ref() {
                            let guard = order_ctx.lock().await;
                            // find by order id or client cid
                            let ctx_opt = guard
                                .iter()
                                .find(|(cid, ctx)| {
                                    *cid == &client_cid
                                        || ctx.order_id.as_ref().map(|id| id.as_str())
                                            == Some(order_id.0.as_str())
                                })
                                .map(|(_, v)| v.clone());
                            if let Some(ctx) = ctx_opt {
                                use crate::order::request::OrderResponseCancel;
                                use crate::order::state::Cancelled as CancelledState;
                                let id_internal = ctx.order_id.clone().unwrap_or_else(|| {
                                    OrderId::from(SmolStr::from(order_id.0.clone()))
                                });
                                let cancelled = CancelledState::new(id_internal.clone(), time);
                                // Build OrderCancelled event (Ok(Cancelled))
                                let response: OrderResponseCancel<ExchangeId, String, String> =
                                    crate::order::OrderEvent::new(
                                        OrderKey {
                                            exchange: ExchangeId::B3,
                                            instrument: ctx.instrument.clone(),
                                            strategy: ctx.strategy.clone(),
                                            cid: ClientOrderId(SmolStr::from(client_cid)),
                                        },
                                        Ok(cancelled),
                                    );
                                let event = crate::AccountEvent {
                                    exchange: ExchangeId::B3,
                                    broker: Some(broker.clone()),
                                    account: Some(account.clone()),
                                    kind: crate::AccountEventKind::OrderCancelled(response),
                                };
                                let _ = tx.send(event);
                            } else {
                                tracing::warn!(order_id=%order_id.0, "Cancel sem contexto");
                            }
                        }
                    }
                    _ => { /* ignore others for now */ }
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
        let side = match request.state.side {
            Side::Buy => TransportSide::Buy,
            Side::Sell => TransportSide::Sell,
        };
        let kind = match request.state.kind {
            OrderKind::Market => TransportOrderKind::Market,
            OrderKind::Limit => TransportOrderKind::Limit,
        };
        // Map internal TimeInForce to transport variant
        let tif = match request.state.time_in_force {
            crate::order::TimeInForce::GoodUntilCancelled { .. } => TransportTimeInForce::GTC,
            crate::order::TimeInForce::GoodUntilEndOfDay => TransportTimeInForce::Day,
            crate::order::TimeInForce::ImmediateOrCancel => TransportTimeInForce::IOC,
            crate::order::TimeInForce::FillOrKill => TransportTimeInForce::FOK,
        };
        let account = TransportAccountId::new(
            self.config.account_id.clone(),
            self.config.broker_id.clone(),
        );
        let opened = self
            .transport
            .open_order(
                &instrument,
                side,
                request.state.quantity,
                kind,
                Some(request.state.price),
                tif,
                request.key.cid.0.as_str(),
                &account,
            )
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
                    strategy: request.key.strategy.clone(),
                    order_id: Some(OrderId::from(SmolStr::from(opened.id.0.clone()))),
                    filled_qty: Decimal::ZERO,
                },
            );
        }
        tracing::info!(
            "Sending order via transport: instrument={}",
            instrument.symbol
        );
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
                OrderId::from(SmolStr::from(opened.id.0)),
                chrono::Utc::now(),
                Decimal::ZERO,
            )),
        };

        Ok(order)
    }
}

// Re-export for easier access
pub use B3ExecutionClient as B3Client;

// -------------------------------------------------------------------------------------------------
// Test support & unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportOrderId;
    use futures::future::BoxFuture;
    use futures::StreamExt;
    use rust_decimal_macros::dec;
    use tokio::sync::mpsc;
    use tucano_integration::snapshot::Snapshot;
    use tucano_markets::ExchangeId;

    #[derive(Debug)]
    struct DummyTransport {
        events_tx: mpsc::UnboundedSender<TransportEvent>,
        events_rx_once: Mutex<Option<mpsc::UnboundedReceiver<TransportEvent>>>,
        auto_accept: bool,
    }

    impl DummyTransport {
        fn new() -> Arc<Self> {
            Self::new_with_accept(true)
        }
        fn new_with_accept(auto_accept: bool) -> Arc<Self> {
            let (tx, rx) = mpsc::unbounded_channel();
            Arc::new(Self {
                events_tx: tx,
                events_rx_once: Mutex::new(Some(rx)),
                auto_accept,
            })
        }
        fn push(&self, evt: TransportEvent) {
            let _ = self.events_tx.send(evt);
        }
    }

    impl Transport for DummyTransport {
        fn name(&self) -> &'static str {
            "dummy"
        }
        fn connect(&self) -> BoxFuture<'_, Result<(), crate::transport::TransportError>> {
            Box::pin(async { Ok(()) })
        }
        fn account_events(
            &self,
        ) -> BoxFuture<
            '_,
            Result<mpsc::UnboundedReceiver<TransportEvent>, crate::transport::TransportError>,
        > {
            Box::pin(async move {
                Ok(self
                    .events_rx_once
                    .lock()
                    .await
                    .take()
                    .expect("account_events called once"))
            })
        }
        fn open_order<'a>(
            &'a self,
            _instrument: &'a TransportInstrument,
            _side: TransportSide,
            _quantity: Decimal,
            _kind: TransportOrderKind,
            _price: Option<Decimal>,
            _tif: TransportTimeInForce,
            client_cid: &'a str,
            _account: &'a TransportAccountId,
        ) -> BoxFuture<
            'a,
            Result<crate::transport::TransportOpenOrder, crate::transport::TransportError>,
        > {
            Box::pin(async move {
                let id = TransportOrderId(format!("DUMMY-{client_cid}"));
                let opened = crate::transport::TransportOpenOrder {
                    id: id.clone(),
                    submitted_at: Utc::now(),
                    filled_qty: Decimal::ZERO,
                };
                if self.auto_accept {
                    let _ = self.events_tx.send(TransportEvent::OrderAccepted {
                        client_cid: client_cid.to_string(),
                        id,
                    });
                }
                Ok(opened)
            })
        }
        fn cancel_order<'a>(
            &'a self,
            _id: &'a TransportOrderId,
        ) -> BoxFuture<'a, Result<(), crate::transport::TransportError>> {
            Box::pin(async { Ok(()) })
        }
    }

    impl B3ExecutionClient {
        fn with_transport(config: B3Config, transport: Arc<dyn Transport>) -> Self {
            Self {
                config,
                transport,
                event_sender: Arc::new(Mutex::new(None)),
                order_ctx: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[tokio::test]
    async fn order_acceptance_translates_to_order_snapshot_event() {
        let transport = DummyTransport::new();
        let config = B3Config::new("k".into(), "u".into(), "p".into())
            .with_account_id("acct")
            .with_broker_id("broker");
        let client = B3ExecutionClient::with_transport(config, transport.clone());

        // Start stream
        let mut stream = client
            .account_stream(&[], &["PETR4".to_string()])
            .await
            .expect("stream");

        // Open order (will trigger OrderAccepted via DummyTransport.open_order)
        use crate::order::id::{ClientOrderId, StrategyId};
        use crate::order::request::OrderRequestOpen;
        use crate::order::{OrderKind, TimeInForce};
        // Snapshot import not needed here (only pattern matching on enum variant)
        let instrument_name = "PETR4".to_string();
        use crate::order::request::RequestOpen;
        let req = OrderRequestOpen {
            key: crate::order::OrderKey {
                exchange: ExchangeId::B3,
                instrument: &instrument_name,
                strategy: StrategyId(SmolStr::new_inline("strat")),
                cid: ClientOrderId(SmolStr::new_inline("CID1")),
            },
            state: RequestOpen {
                side: Side::Buy,
                price: dec!(10.0),
                quantity: dec!(5),
                kind: OrderKind::Limit,
                time_in_force: TimeInForce::GoodUntilEndOfDay,
            },
        };
        let _order = client.open_order(req).await.expect("open order");

        // Expect one event (OrderSnapshot)
        let evt = stream.next().await.expect("event");
        match evt.kind {
            crate::AccountEventKind::OrderSnapshot(Snapshot(order)) => {
                assert_eq!(order.key.instrument, "PETR4");
                assert_eq!(order.side, Side::Buy);
            }
            other => panic!("unexpected event: {:?}", other),
        }
    }
    #[tokio::test]
    async fn order_rejection_translates_to_failed_order_snapshot_event() {
        let transport = DummyTransport::new_with_accept(false);
        let config = B3Config::new("k".into(), "u".into(), "p".into())
            .with_account_id("acct")
            .with_broker_id("broker");
        let client = B3ExecutionClient::with_transport(config, transport.clone());
        let mut stream = client
            .account_stream(&[], &["PETR4".to_string()])
            .await
            .expect("stream");
        use crate::order::id::{ClientOrderId, StrategyId};
        use crate::order::request::{OrderRequestOpen, RequestOpen};
        use crate::order::{OrderKind, TimeInForce};
        // Snapshot import not needed here
        use rust_decimal_macros::dec;
        let instrument_name = "PETR4".to_string();
        let req = OrderRequestOpen {
            key: crate::order::OrderKey {
                exchange: ExchangeId::B3,
                instrument: &instrument_name,
                strategy: StrategyId(SmolStr::new_inline("strat")),
                cid: ClientOrderId(SmolStr::new_inline("CID2")),
            },
            state: RequestOpen {
                side: Side::Buy,
                price: dec!(10),
                quantity: dec!(5),
                kind: OrderKind::Limit,
                time_in_force: TimeInForce::GoodUntilEndOfDay,
            },
        };
        let _order = client.open_order(req).await.expect("open order");
        transport.push(TransportEvent::OrderRejected {
            client_cid: "CID2".to_string(),
            reason: "PRICE_OUT_OF_RANGE".to_string(),
        });
        let evt = stream.next().await.expect("event");
        match evt.kind {
            crate::AccountEventKind::OrderSnapshot(Snapshot(order)) => {
                use crate::order::state::OrderState;
                if let OrderState::Inactive(inactive) = order.state {
                    use crate::order::state::InactiveOrderState;
                    match inactive {
                        InactiveOrderState::OpenFailed(err) => {
                            assert!(format!("{}", err).contains("PRICE_OUT_OF_RANGE"));
                        }
                        other => panic!("unexpected inactive state: {:?}", other),
                    }
                } else {
                    panic!("expected inactive state");
                }
            }
            other => panic!("unexpected event: {:?}", other),
        }
    }

    #[tokio::test]
    async fn trade_event_translates_to_trade_account_event() {
        let transport = DummyTransport::new(); // auto_accept true
        let config = B3Config::new("k".into(), "u".into(), "p".into())
            .with_account_id("acct")
            .with_broker_id("broker");
        let client = B3ExecutionClient::with_transport(config, transport.clone());
        let mut stream = client
            .account_stream(&[], &["PETR4".to_string()])
            .await
            .expect("stream");
        use crate::order::id::{ClientOrderId, StrategyId};
        use crate::order::request::{OrderRequestOpen, RequestOpen};
        use crate::order::{OrderKind, TimeInForce};
        // Snapshot import not needed here
        use rust_decimal_macros::dec;
        let instrument_name = "PETR4".to_string();
        let req = OrderRequestOpen {
            key: crate::order::OrderKey {
                exchange: ExchangeId::B3,
                instrument: &instrument_name,
                strategy: StrategyId(SmolStr::new_inline("strat")),
                cid: ClientOrderId(SmolStr::new_inline("CID3")),
            },
            state: RequestOpen {
                side: Side::Buy,
                price: dec!(10),
                quantity: dec!(5),
                kind: OrderKind::Limit,
                time_in_force: TimeInForce::GoodUntilEndOfDay,
            },
        };
        let _order = client.open_order(req).await.expect("open order");
        // Consume acceptance snapshot first
        let first_evt = stream.next().await.expect("first event");
        match first_evt.kind {
            crate::AccountEventKind::OrderSnapshot(_) => {}
            other => panic!("expected order snapshot, got {:?}", other),
        }
        // Push trade event
        use chrono::Utc;
        transport.push(TransportEvent::Trade {
            order_id: TransportOrderId("DUMMY-CID3".to_string()),
            price: dec!(10.5),
            quantity: dec!(2),
            fees: dec!(0.01),
            time: Utc::now(),
        });
        let trade_evt = stream.next().await.expect("trade event");
        let snapshot_evt = stream.next().await.expect("snapshot event");
        if let crate::AccountEventKind::Trade(trade) = trade_evt.kind {
            assert_eq!(trade.instrument, "PETR4");
            assert_eq!(trade.quantity, dec!(2));
            assert_eq!(trade.price, dec!(10.5));
        } else {
            panic!("expected trade event");
        }
        if let crate::AccountEventKind::OrderSnapshot(tucano_integration::snapshot::Snapshot(
            order,
        )) = snapshot_evt.kind
        {
            use crate::order::state::{ActiveOrderState, OrderState};
            if let OrderState::Active(ActiveOrderState::Open(open)) = order.state {
                assert_eq!(open.filled_quantity, dec!(2));
            } else {
                panic!("expected active open state");
            }
        } else {
            panic!("expected order snapshot event");
        }
    }

    #[tokio::test]
    async fn multiple_trades_accumulate_filled_qty() {
        let transport = DummyTransport::new();
        let config = B3Config::new("k".into(), "u".into(), "p".into())
            .with_account_id("acct")
            .with_broker_id("broker");
        let client = B3ExecutionClient::with_transport(config, transport.clone());
        let mut stream = client
            .account_stream(&[], &["PETR4".to_string()])
            .await
            .expect("stream");
        use crate::order::id::{ClientOrderId, StrategyId};
        use crate::order::request::{OrderRequestOpen, RequestOpen};
        use crate::order::{OrderKind, TimeInForce};
        use rust_decimal::Decimal;
        use rust_decimal_macros::dec;
        let instrument_name = "PETR4".to_string();
        let req = OrderRequestOpen {
            key: crate::order::OrderKey {
                exchange: ExchangeId::B3,
                instrument: &instrument_name,
                strategy: StrategyId(SmolStr::new_inline("strat")),
                cid: ClientOrderId(SmolStr::new_inline("CID4")),
            },
            state: RequestOpen {
                side: Side::Buy,
                price: dec!(10),
                quantity: dec!(10),
                kind: OrderKind::Limit,
                time_in_force: TimeInForce::GoodUntilEndOfDay,
            },
        };
        let _order = client.open_order(req).await.expect("open order");
        // consume acceptance snapshot
        let _ = stream.next().await.expect("snapshot");
        use chrono::Utc;
        // first trade 3
        transport.push(TransportEvent::Trade {
            order_id: TransportOrderId("DUMMY-CID4".to_string()),
            price: dec!(10),
            quantity: dec!(3),
            fees: dec!(0.005),
            time: Utc::now(),
        });
        let _trade1 = stream.next().await.expect("trade1");
        let snap1 = stream.next().await.expect("snap1");
        // second trade 4
        transport.push(TransportEvent::Trade {
            order_id: TransportOrderId("DUMMY-CID4".to_string()),
            price: dec!(10.1),
            quantity: dec!(4),
            fees: dec!(0.006),
            time: Utc::now(),
        });
        let _trade2 = stream.next().await.expect("trade2");
        let snap2 = stream.next().await.expect("snap2");
        use crate::order::state::{ActiveOrderState, OrderState};
        fn extract_filled(
            evt: &crate::AccountEvent<ExchangeId, String, String>,
        ) -> Option<Decimal> {
            if let crate::AccountEventKind::OrderSnapshot(tucano_integration::snapshot::Snapshot(
                order,
            )) = &evt.kind
            {
                if let OrderState::Active(ActiveOrderState::Open(open)) = &order.state {
                    return Some(open.filled_quantity);
                }
            }
            None
        }
        let f1 = extract_filled(&snap1).expect("filled1");
        let f2 = extract_filled(&snap2).expect("filled2");
        assert_eq!(f1, dec!(3));
        assert_eq!(f2, dec!(7));
    }

    #[tokio::test]
    async fn order_cancel_translates_to_cancel_event() {
        let transport = DummyTransport::new();
        let config = B3Config::new("k".into(), "u".into(), "p".into())
            .with_account_id("acct")
            .with_broker_id("broker");
        let client = B3ExecutionClient::with_transport(config, transport.clone());
        let mut stream = client
            .account_stream(&[], &["PETR4".to_string()])
            .await
            .expect("stream");
        use crate::order::id::{ClientOrderId, StrategyId};
        use crate::order::request::{OrderRequestOpen, RequestOpen};
        use crate::order::{OrderKind, TimeInForce};
        use chrono::Utc;
        use rust_decimal_macros::dec;
        let instrument_name = "PETR4".to_string();
        let req = OrderRequestOpen {
            key: crate::order::OrderKey {
                exchange: ExchangeId::B3,
                instrument: &instrument_name,
                strategy: StrategyId(SmolStr::new_inline("strat")),
                cid: ClientOrderId(SmolStr::new_inline("CID5")),
            },
            state: RequestOpen {
                side: Side::Buy,
                price: dec!(10),
                quantity: dec!(5),
                kind: OrderKind::Limit,
                time_in_force: TimeInForce::GoodUntilEndOfDay,
            },
        };
        let _order = client.open_order(req).await.expect("open order");
        // consume acceptance snapshot
        let _ = stream.next().await.expect("snapshot");
        // push cancel
        transport.push(TransportEvent::OrderCancelled {
            order_id: crate::transport::TransportOrderId("DUMMY-CID5".to_string()),
            client_cid: "CID5".to_string(),
            time: Utc::now(),
        });
        let evt = stream.next().await.expect("cancel event");
        match evt.kind {
            crate::AccountEventKind::OrderCancelled(cancel_resp) => {
                // state holds Result<Cancelled, OrderError>
                match cancel_resp.state {
                    Ok(cancelled) => assert_eq!(cancelled.id.as_str(), "DUMMY-CID5"),
                    Err(e) => panic!("unexpected cancel error: {:?}", e),
                }
            }
            other => panic!("expected OrderCancelled event, got {:?}", other),
        }
    }
}

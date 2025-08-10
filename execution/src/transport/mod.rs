//! Transport layer abstractions for execution clients.
//!
//! Goal: isolate connectivity / protocol (DLLs, HTTP, WS) from higher-level
//! execution client logic. This enables:
//! * Swapping implementations (mock, ProfitDLL, future REST) without touching business logic
//! * Easier testing (inject a Transport mock)
//! * Clear separation of responsibilities (Phase 2 of roadmap)
//!
//! Design principles:
//! * Async trait object friendly (dyn Transport + Send + Sync)
//! * Event stream decoupled via mpsc::UnboundedReceiver
//! * Minimal surface: connect, subscribe instruments, send order, cancel order
//! * Domain-neutral: uses plain symbols & account identifiers; mapping to internal indices
//!   remains in higher layers.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use futures::future::BoxFuture;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportOrderId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportAccountId {
    pub account: String,
    pub broker: String,
}

impl TransportAccountId {
    pub fn new(account: impl Into<String>, broker: impl Into<String>) -> Self {
        Self { account: account.into(), broker: broker.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportInstrument {
    pub symbol: String,   // e.g. "PETR4"
    pub exchange: String, // e.g. "B3" / venue code
}

impl TransportInstrument {
    pub fn new(symbol: impl Into<String>, exchange: impl Into<String>) -> Self {
        Self { symbol: symbol.into(), exchange: exchange.into() }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransportSide { Buy, Sell }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransportOrderKind { Market, Limit }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransportTimeInForce { Day, IOC, GTC, FOK }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportOpenOrder {
    pub id: TransportOrderId,
    pub submitted_at: DateTime<Utc>,
    pub filled_qty: Decimal,
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("connectivity: {0}")] Connectivity(String),
    #[error("protocol: {0}")] Protocol(String),
    #[error("rejected: {0}")] Rejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportEvent {
    Connected,
    Disconnected,
    OrderAccepted { client_cid: String, id: TransportOrderId },
    OrderRejected { client_cid: String, reason: String },
    Trade {
        order_id: TransportOrderId,
        price: Decimal,
        quantity: Decimal,
        fees: Decimal,
        time: DateTime<Utc>,
    },
    Heartbeat,
}

pub trait Transport: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn connect(&self) -> BoxFuture<'_, Result<(), TransportError>>;
    fn account_events(&self) -> BoxFuture<'_, Result<mpsc::UnboundedReceiver<TransportEvent>, TransportError>>;
    fn open_order<'a>(
        &'a self,
        instrument: &'a TransportInstrument,
        side: TransportSide,
        quantity: Decimal,
        kind: TransportOrderKind,
        price: Option<Decimal>,
        tif: TransportTimeInForce,
        client_cid: &'a str,
        account: &'a TransportAccountId,
    ) -> BoxFuture<'a, Result<TransportOpenOrder, TransportError>>;
    fn cancel_order<'a>(&'a self, id: &'a TransportOrderId) -> BoxFuture<'a, Result<(), TransportError>>;
}

/// A mock transport for tests and initial integration.
#[derive(Debug, Default)]
pub struct MockTransport;

impl Transport for MockTransport {
    fn name(&self) -> &'static str { "mock" }
    fn connect(&self) -> BoxFuture<'_, Result<(), TransportError>> { Box::pin(async { Ok(()) }) }
    fn account_events(&self) -> BoxFuture<'_, Result<mpsc::UnboundedReceiver<TransportEvent>, TransportError>> {
        Box::pin(async { let (_tx, rx) = mpsc::unbounded_channel(); Ok(rx) })
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
    ) -> BoxFuture<'a, Result<TransportOpenOrder, TransportError>> {
        Box::pin(async move { Ok(TransportOpenOrder { id: TransportOrderId(format!("MOCK-{client_cid}")), submitted_at: Utc::now(), filled_qty: Decimal::ZERO }) })
    }
    fn cancel_order<'a>(&'a self, _id: &'a TransportOrderId) -> BoxFuture<'a, Result<(), TransportError>> { Box::pin(async { Ok(()) }) }
}

// -----------------------------------------------------------------------------
// ProfitDLL transport (thin wrapper) - experimental
// -----------------------------------------------------------------------------
use markets::profit_dll::{CallbackEvent, ProfitConnector, ProfitError, OrderSide};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use std::sync::Arc;

#[derive(Debug)]
pub struct ProfitDLLTransport {
    connector: Arc<Mutex<Option<ProfitConnector>>>,
    dll_path: Option<String>,
    activation_key: String,
    user: String,
    password: String,
    events_tx: Arc<Mutex<Option<UnboundedSender<TransportEvent>>>>,
}

impl ProfitDLLTransport {
    pub fn new(
        dll_path: Option<String>,
        activation_key: String,
        user: String,
        password: String,
    ) -> Self {
        Self {
            connector: Arc::new(Mutex::new(None)),
            dll_path,
            activation_key,
            user,
            password,
            events_tx: Arc::new(Mutex::new(None)),
        }
    }

    async fn ensure_connected_inner(&self) -> Result<(), TransportError> {
        let mut guard = self.connector.lock().await;
        if guard.is_none() {
            let connector = ProfitConnector::new(self.dll_path.as_deref())
                .map_err(|e| TransportError::Connectivity(e))?;
            let rx = connector.initialize_login(&self.activation_key, &self.user, &self.password)
                .await
                .map_err(|e| TransportError::Connectivity(e))?;
            self.spawn_event_loop(rx);
            *guard = Some(connector);
        }
        Ok(())
    }

    fn spawn_event_loop(&self, mut rx: tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>) {
        let tx_holder = self.events_tx.clone();
        tokio::spawn(async move {
            while let Some(evt) = rx.recv().await {
                let mapped = match evt {
                    CallbackEvent::StateChanged { .. } => Some(TransportEvent::Heartbeat),
                    // TODO: Map real trade / order events once ProfitConnector produces them
                    _ => None,
                };
                if let Some(ev) = mapped {
                    if let Some(tx) = tx_holder.lock().await.as_ref() { let _ = tx.send(ev); }
                }
            }
        });
    }
}

impl Transport for ProfitDLLTransport {
    fn name(&self) -> &'static str { "profit_dll" }
    fn connect(&self) -> BoxFuture<'_, Result<(), TransportError>> { Box::pin(async { self.ensure_connected_inner().await }) }
    fn account_events(&self) -> BoxFuture<'_, Result<mpsc::UnboundedReceiver<TransportEvent>, TransportError>> {
        Box::pin(async {
            self.ensure_connected_inner().await?;
            let (tx, rx) = mpsc::unbounded_channel();
            *self.events_tx.lock().await = Some(tx);
            Ok(rx)
        })
    }
    fn open_order<'a>(
        &'a self,
        instrument: &'a TransportInstrument,
        _side: TransportSide,
        _quantity: Decimal,
        _kind: TransportOrderKind,
        _price: Option<Decimal>,
        _tif: TransportTimeInForce,
        client_cid: &'a str,
        _account: &'a TransportAccountId,
    ) -> BoxFuture<'a, Result<TransportOpenOrder, TransportError>> {
        Box::pin(async move {
            self.ensure_connected_inner().await?;
            let order_id = format!("DLL-{}-{}", instrument.symbol, client_cid);
            let id = TransportOrderId(order_id);
            if let Some(tx) = self.events_tx.lock().await.as_ref() {
                let _ = tx.send(TransportEvent::OrderAccepted { client_cid: client_cid.to_string(), id: id.clone() });
            }
            Ok(TransportOpenOrder { id, submitted_at: Utc::now(), filled_qty: Decimal::ZERO })
        })
    }
    fn cancel_order<'a>(&'a self, id: &'a TransportOrderId) -> BoxFuture<'a, Result<(), TransportError>> {
        Box::pin(async move { self.ensure_connected_inner().await?; let _ = id; Ok(()) })
    }
}

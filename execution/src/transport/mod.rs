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
use futures::future::BoxFuture;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportOrderId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportAccountId {
    pub account: String,
    pub broker: String,
}

impl TransportAccountId {
    pub fn new(account: impl Into<String>, broker: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            broker: broker.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportInstrument {
    pub symbol: String,   // e.g. "PETR4"
    pub exchange: String, // e.g. "B3" / venue code
}

impl TransportInstrument {
    pub fn new(symbol: impl Into<String>, exchange: impl Into<String>) -> Self {
        Self {
            symbol: symbol.into(),
            exchange: exchange.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransportSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransportOrderKind {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransportTimeInForce {
    Day,
    IOC,
    GTC,
    FOK,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportOpenOrder {
    pub id: TransportOrderId,
    pub submitted_at: DateTime<Utc>,
    pub filled_qty: Decimal,
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("connectivity: {0}")]
    Connectivity(String),
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("rejected: {0}")]
    Rejected(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportEvent {
    Connected,
    Disconnected,
    OrderAccepted {
        client_cid: String,
        id: TransportOrderId,
    },
    OrderRejected {
        client_cid: String,
        reason: String,
    },
    Trade {
        order_id: TransportOrderId,
        price: Decimal,
        quantity: Decimal,
        fees: Decimal,
        time: DateTime<Utc>,
    },
    OrderCancelled {
        order_id: TransportOrderId,
        client_cid: String,
        time: DateTime<Utc>,
    },
    Heartbeat,
}

pub trait Transport: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn connect(&self) -> BoxFuture<'_, Result<(), TransportError>>;
    fn account_events(
        &self,
    ) -> BoxFuture<'_, Result<mpsc::UnboundedReceiver<TransportEvent>, TransportError>>;
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
    fn cancel_order<'a>(
        &'a self,
        id: &'a TransportOrderId,
    ) -> BoxFuture<'a, Result<(), TransportError>>;
}

/// A mock transport for tests and initial integration.
#[derive(Debug, Default)]
pub struct MockTransport;

impl Transport for MockTransport {
    fn name(&self) -> &'static str {
        "mock"
    }
    fn connect(&self) -> BoxFuture<'_, Result<(), TransportError>> {
        Box::pin(async { Ok(()) })
    }
    fn account_events(
        &self,
    ) -> BoxFuture<'_, Result<mpsc::UnboundedReceiver<TransportEvent>, TransportError>> {
        Box::pin(async {
            let (_tx, rx) = mpsc::unbounded_channel();
            Ok(rx)
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
    ) -> BoxFuture<'a, Result<TransportOpenOrder, TransportError>> {
        Box::pin(async move {
            Ok(TransportOpenOrder {
                id: TransportOrderId(format!("MOCK-{client_cid}")),
                submitted_at: Utc::now(),
                filled_qty: Decimal::ZERO,
            })
        })
    }
    fn cancel_order<'a>(
        &'a self,
        _id: &'a TransportOrderId,
    ) -> BoxFuture<'a, Result<(), TransportError>> {
        Box::pin(async { Ok(()) })
    }
}

// -----------------------------------------------------------------------------
// ProfitDLL transport (thin wrapper) - experimental
// -----------------------------------------------------------------------------
// ProfitDLL integration extracted to external crate `profitdll`.
// When reintroducing a concrete transport, implement it in that crate and depend on it here.

// NOTE: Previous inline ProfitDLLTransport removed. See external crate for future implementation.

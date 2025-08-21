
//! # Core Error Types
//!
//! This module defines the main error types used in the core module of the Toucan framework.
//! It provides a centralized error handling system aggregating errors from various subsystems:
//! execution, market data, and indexing.
//!
//! ## Error Hierarchy
//!
//! The main type `ToucanError` includes:
//! - **IndexError**: Indexing errors for asset / instrument / exchange
//! - **ExecutionBuilder**: Errors during initialization of the execution subsystem
//! - **ExecutionRxDropped**: Communication channel whose receiver was dropped
//! - **MarketData**: Errors from the data module (streaming, parsing, subscription)
//! - **Execution**: Execution errors (orders, balances, liquidations)
//! - **JoinError**: Failures when awaiting async tasks (join)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use toucan_core::error::ToucanError;
//!
//! fn handle_trading_error(error: ToucanError) {
//!     match error {
//!         ToucanError::MarketData(data_err) => {
//!             eprintln!("Market data problem: {}", data_err);
//!         }
//!         ToucanError::Execution(exec_err) => {
//!             eprintln!("Execution problem: {}", exec_err);
//!         }
//!         _ => eprintln!("Other error: {}", error),
//!     }
//! }
//! ```

use crate::execution::error::ExecutionError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use toucan_data::error::DataError;
use toucan_execution::IndexError;

/// Central error type for the core module of the Toucan framework.
///
/// Enum that aggregates all possible errors of the core trading system,
/// providing a unified handling interface between subsystems.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Error)]
pub enum ToucanError {
    /// Indexing errors for asset, instrument, or exchange
    #[error("IndexError: {0}")]
    IndexError(#[from] IndexError),

    /// Configuration errors (builder) of the execution system
//! This module defines the main error types used in the core module of the Toucan framework.
//! The main type `ToucanError` includes:
//! use toucan_core::error::ToucanError;
//! use toucan_core::error::ToucanError;
//! fn handle_trading_error(error: ToucanError) {
//!     match error {
//!         ToucanError::MarketData(data_err) => {
//!             eprintln!("Market data problem: {}", data_err);
//!         }
//!         ToucanError::Execution(exec_err) => {
//!             eprintln!("Execution problem: {}", exec_err);
//!         }
//!         _ => eprintln!("Other error: {}", error),
//!     }
//! }
/// Central error type for the core module of the Toucan framework.
    #[error("JoinError: {0}")]
    JoinError(String),
}
/// Indicates that the receiver side of a communication channel was dropped.
///
/// Typical scenarios:
/// - ExecutionRequest receiver from ExchangeManager dropped
/// - Channels between engine components were closed
/// - Asynchronous tasks ended unexpectedly, dropping receivers
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Error)]
#[error("RxDropped")]
pub struct RxDropped;

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for RxDropped {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ToucanError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ExecutionRxDropped(RxDropped)
    }
}

impl From<tokio::task::JoinError> for ToucanError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::JoinError(format!("{value:?}"))
    }
}

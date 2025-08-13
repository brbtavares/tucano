//! # Core Error Types
//!
//! This module defines the primary error types used throughout the Tucano trading framework's core module.
//! It provides a centralized error handling system that aggregates errors from various subsystems including
//! execution, market data, and indexing.
//!
//! ## Error Hierarchy
//!
//! The main error type `TucanoError` encompasses:
//! - **IndexError**: Errors related to asset/instrument/exchange indexing
//! - **ExecutionBuilder**: Errors during execution system initialization
//! - **ExecutionRxDropped**: Communication channel errors when receivers are dropped
//! - **MarketData**: Errors from the data module (market data streaming, parsing, etc.)
//! - **Execution**: Errors from the execution module (order management, balance tracking, etc.)
//! - **JoinError**: Async task join errors in concurrent operations
//!
//! ## Usage
//!
//! ```rust
//! use core::error::TucanoError;
//!
//! fn handle_trading_error(error: TucanoError) {
//!     match error {
//!         TucanoError::MarketData(data_err) => {
//!             eprintln!("Market data issue: {}", data_err);
//!         }
//!         TucanoError::Execution(exec_err) => {
//!             eprintln!("Execution issue: {}", exec_err);
//!         }
//!         _ => eprintln!("Other error: {}", error),
//!     }
//! }
//! ```

use crate::execution::error::ExecutionError;
use data::error::DataError;
use execution::IndexError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Central error type for the Tucano trading framework's core module.
///
/// This enum aggregates all possible errors that can occur within the core trading system,
/// providing a unified interface for error handling across different subsystems.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Error)]
pub enum TucanoError {
    /// Asset, instrument, or exchange indexing errors
    #[error("IndexError: {0}")]
    IndexError(#[from] IndexError),

    /// Errors during execution system builder configuration
    #[error("ExecutionBuilder: {0}")]
    ExecutionBuilder(String),

    /// Communication channel receiver was dropped unexpectedly
    #[error("ExchangeManager dropped it's ExecutionRequest receiver")]
    ExecutionRxDropped(#[from] RxDropped),

    /// Market data streaming, parsing, or subscription errors
    #[error("market data: {0}")]
    MarketData(#[from] DataError),

    /// Order execution, balance tracking, or trade settlement errors
    #[error("execution: {0}")]
    Execution(#[from] ExecutionError),

    /// Async task join failures in concurrent operations
    #[error("JoinError: {0}")]
    JoinError(String),
}
/// Error indicating that a receiver end of a communication channel was dropped.
///
/// This typically occurs when:
/// - An ExchangeManager's ExecutionRequest receiver is dropped
/// - Communication channels between engine components are severed
/// - Async tasks terminate unexpectedly, dropping their receivers
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Error)]
#[error("RxDropped")]
pub struct RxDropped;

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for RxDropped {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for TucanoError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::ExecutionRxDropped(RxDropped)
    }
}

impl From<tokio::task::JoinError> for TucanoError {
    fn from(value: tokio::task::JoinError) -> Self {
        Self::JoinError(format!("{value:?}"))
    }
}

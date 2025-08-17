// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! # Core Error Types
//!
//! This module defines the main error types used in the core module of the Tucano framework.
//! It provides a centralized error handling system aggregating errors from various subsystems:
//! execution, market data, and indexing.
//!
//! ## Error Hierarchy
//!
//! The main type `TucanoError` includes:
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
//! use tucano_core::error::TucanoError;
//!
//! fn handle_trading_error(error: TucanoError) {
//!     match error {
//!         TucanoError::MarketData(data_err) => {
//!             eprintln!("Market data problem: {}", data_err);
//!         }
//!         TucanoError::Execution(exec_err) => {
//!             eprintln!("Execution problem: {}", exec_err);
//!         }
//!         _ => eprintln!("Other error: {}", error),
//!     }
//! }
//! ```

use crate::execution::error::ExecutionError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tucano_data::error::DataError;
use tucano_execution::IndexError;

/// Tipo central de erro do módulo core do framework Tucano.
///
/// Enum que agrega todos os erros possíveis do sistema de trading core,
/// fornecendo uma interface unificada de tratamento entre subsistemas.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Error)]
pub enum TucanoError {
    /// Erros de indexação de ativo, instrumento ou exchange
    #[error("IndexError: {0}")]
    IndexError(#[from] IndexError),

    /// Erros na configuração (builder) do sistema de execução
    #[error("ExecutionBuilder: {0}")]
    ExecutionBuilder(String),

    /// Receiver de canal de comunicação foi descartado inesperadamente
    #[error("ExchangeManager dropped it's ExecutionRequest receiver")]
    ExecutionRxDropped(#[from] RxDropped),

    /// Erros de streaming, parsing ou assinatura de dados de mercado
    #[error("market data: {0}")]
    MarketData(#[from] DataError),

    /// Erros de execução de ordens, rastreamento de saldo ou liquidação de trades
    #[error("execution: {0}")]
    Execution(#[from] ExecutionError),

    /// Falhas ao fazer join de tasks assíncronas
    #[error("JoinError: {0}")]
    JoinError(String),
}
/// Indica que o lado receiver de um canal de comunicação foi descartado.
///
/// Típicos cenários:
/// - Receiver de ExecutionRequest do ExchangeManager caiu
/// - Canais entre componentes do engine foram encerrados
/// - Tasks assíncronas terminaram inesperadamente descartando receivers
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

//! # Tipos de Erro do Core
//!
//! Este módulo define os principais tipos de erro usados no módulo core do framework Tucano.
//! Fornece um sistema centralizado de tratamento agregando erros de vários subsistemas:
//! execução, dados de mercado e indexação.
//!
//! ## Hierarquia de Erros
//!
//! O tipo principal `TucanoError` engloba:
//! - **IndexError**: Erros de indexação de ativo / instrumento / exchange
//! - **ExecutionBuilder**: Erros durante inicialização do subsistema de execução
//! - **ExecutionRxDropped**: Canal de comunicação cujo receiver foi descartado
//! - **MarketData**: Erros do módulo de dados (streaming, parsing, assinatura)
//! - **Execution**: Erros de execução (ordens, saldos, liquidações)
//! - **JoinError**: Falhas ao aguardar tasks assíncronas (join)
//!
//! ## Uso
//!
//! ```rust
//! use core::error::TucanoError;
//!
//! fn tratar_erro_trading(error: TucanoError) {
//!     match error {
//!         TucanoError::MarketData(data_err) => {
//!             eprintln!("Problema de dados de mercado: {}", data_err);
//!         }
//!         TucanoError::Execution(exec_err) => {
//!             eprintln!("Problema de execução: {}", exec_err);
//!         }
//!         _ => eprintln!("Outro erro: {}", error),
//!     }
//! }
//! ```

use crate::execution::error::ExecutionError;
use tucano_data::error::DataError;
use tucano_execution::IndexError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

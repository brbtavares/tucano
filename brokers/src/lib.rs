//! Broker abstraction layer
//!
//! Fornece tipos para identificar corretoras, associar metadados de certificação
//! e modelos de custo aplicáveis a instrumentos negociados. Esta camada é
//! independente de `markets` (instrumentos) e `execution` (ordens) para permitir
//! evolução incremental. Futuramente poderemos ligar Brokers a Accounts e
//! Exchanges.

pub mod model;
pub mod registry;

pub use model::*;
pub use registry::*;

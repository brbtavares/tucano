// Tipos de compatibilidade para migração da arquitetura markets
// TODO: Substituir por implementações baseadas nos traits markets

pub type AssetIndex = String;
pub type InstrumentIndex = String;
pub type AssetNameExchange = String;
pub type InstrumentNameExchange = String;
pub type QuoteAsset = String;
pub type ExchangeIndex = String;
pub type ExchangeKey = String;
pub type AssetKey = String;
pub type InstrumentKey = String;

// Re-export do markets - mantendo ExchangeId como enum original
pub use markets::{Side, ExchangeId};

// Import dos tipos de order necessários
use crate::order::OrderKey;

// Tipos de response compatíveis  
pub type UnindexedOrderKey = OrderKey<String>;

// Para compatibilidade com código antigo que esperava IndexError
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("Asset index error: {0}")]
    AssetIndex(String),
    #[error("Instrument index error: {0}")]
    InstrumentIndex(String),
    #[error("Exchange index error: {0}")]
    ExchangeIndex(String),
}

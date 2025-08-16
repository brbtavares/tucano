//! DISCLAIMER (resumo): Uso educacional/experimental. Sem recomendação de investimento.
//! Sem afiliação institucional ou remuneração de terceiros. Profit/ProfitDLL são
//! propriedade da Nelógica; integração meramente técnica (FFI dinâmico). Consulte
//! README & DISCLAIMER completos.
//!
//! Integração isolada com ProfitDLL. Fornece tipos, eventos e (opcionalmente)
//! bindings FFI reais via feature `real_dll`.

mod api;
mod error;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
mod ffi;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
mod ffi_types;
mod mock;

// Evita ambiguidade de glob: exporta NResult apenas de um lugar
pub use error::*;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi::NResult; // contém ProfitError etc.

pub use api::*;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi::ProfitConnector as RealProfitConnector;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi_types::*;
pub use mock::ProfitConnector as MockProfitConnector;
pub use mock::*; // inclui CallbackEvent, HistoryTradeSource etc.

// Compat: módulo `profitdll` interno para permitir `use profitdll::*` mesmo quando o crate se chama tucano-profitdll
pub mod profitdll {
    pub use super::*;
}

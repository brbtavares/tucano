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
mod mock;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
mod ffi_types;

pub use api::*;
pub use error::*;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi::*;
pub use mock::*; // inclui CallbackEvent, HistoryTradeSource etc.
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi_types::*;

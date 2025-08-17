//! DISCLAIMER (summary): Educational/experimental use. Not investment advice.
//! No institutional affiliation or third-party compensation. Proprietary APIs/libraries belong to their respective owners; integration is purely technical (dynamic FFI). See full README & DISCLAIMER.
//!
//! Isolated integration with ProfitDLL. Provides types, events, and (optionally)
//! real FFI bindings via the `real_dll` feature.

mod api;
mod error;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
mod ffi;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
mod ffi_types;
mod mock;

// Avoid glob ambiguity: export NResult from a single place only
pub use error::*;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi::NResult; // contains ProfitError etc.

pub use api::*;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi::ProfitConnector as RealProfitConnector;
#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi_types::*;
pub use mock::ProfitConnector as MockProfitConnector;
pub use mock::*; // includes CallbackEvent, HistoryTradeSource etc.

// Compat: internal `profitdll` module to allow `use profitdll::*` even when the crate is named tucano-profitdll
pub mod profitdll {
    pub use super::*;
}

//! Integração isolada com ProfitDLL.
//! Fornece tipos, eventos e (opcionalmente) bindings FFI reais via feature `real_dll`.

#[cfg(all(target_os = "windows", feature = "real_dll"))]
mod ffi;
mod mock;
mod error;

#[cfg(all(target_os = "windows", feature = "real_dll"))]
pub use ffi::*;
pub use mock::*;
pub use error::*;

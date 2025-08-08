//! Broker abstraction layer
//!
//! Provides unified interfaces for interacting with different trading brokers
//! and market data providers. Currently supports ProfitDLL for B3 integration.

pub mod traits;
pub mod profit_dll;

pub use traits::*;
pub use profit_dll::*;


#![forbid(unsafe_code)]
#![warn(
    unused,
    clippy::cognitive_complexity,
    unused_extern_crates,
    clippy::unused_self,
    clippy::useless_let_if_seq,
    missing_debug_implementations,
    rust_2018_idioms
)]
// Dependencies are used by downstream crates via generic types; suppress to avoid false positives
#![allow(unused_crate_dependencies)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, type_alias_bounds)]

//! Core abstractions for trading strategies (traits + types). Concrete implementations go to the `strategies` crate.

pub mod algo;
pub mod close_positions;
pub mod on_disconnect;
pub mod on_trading_disabled;

pub use algo::AlgoStrategy;
pub use close_positions::ClosePositionsStrategy;
pub use on_disconnect::OnDisconnectStrategy;
pub use on_trading_disabled::OnTradingDisabled;

#![forbid(unsafe_code)]
#![warn(
    unused,
    clippy::cognitive_complexity,
    unused_crate_dependencies,
    unused_extern_crates,
    clippy::unused_self,
    clippy::useless_let_if_seq,
    missing_debug_implementations,
    rust_2018_idioms,
    rust_2024_compatibility
)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, type_alias_bounds)]

//! # Toucan Strategy
//!
//! Strategy interfaces and implementations for the Toucan trading ecosystem.
//!
//! This crate provides the core strategy traits and default implementations that can be used
//! with the Toucan trading engine:
//!
//! - [`AlgoStrategy`] - For generating algorithmic orders
//! - [`ClosePositionsStrategy`] - For closing open positions
//! - [`OnDisconnectStrategy`] - For handling exchange disconnections
//! - [`OnTradingDisabled`] - For handling trading disabled events
//!
//! ## Overview
//!
//! Strategies in the Toucan ecosystem are designed to be pluggable components that can be
//! combined with different engines, risk managers, and execution systems. Each strategy
//! type handles a specific aspect of trading logic.

/// Defines a strategy interface for generating algorithmic open and cancel order requests based
/// on the current `EngineState`.
pub mod algo;

/// Defines a strategy interface for generating open and cancel order requests that close open
/// positions.
pub mod close_positions;

/// Defines a strategy interface enables custom actions to be performed in the event of an
/// exchange disconnection.
pub mod on_disconnect;

/// Defines a strategy interface enables custom actions to be performed in the event that the
/// `TradingState` gets set to `TradingState::Disabled`.
pub mod on_trading_disabled;

/// Default strategy implementations
pub mod default;

// Re-export the main traits for convenience
pub use algo::AlgoStrategy;
pub use close_positions::ClosePositionsStrategy;
pub use on_disconnect::OnDisconnectStrategy;
pub use on_trading_disabled::OnTradingDisabled;
pub use default::DefaultStrategy;

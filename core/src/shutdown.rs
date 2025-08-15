//! # Shutdown Management
//!
//! This module provides traits and types for managing graceful shutdown of trading system components.
//! It supports both synchronous and asynchronous shutdown patterns to accommodate different component types.
//!
//! ## Shutdown Patterns
//!
//! ### Synchronous Shutdown
//! For components that can shut down immediately without async operations:
//! ```rust,ignore
//! use tucano_core::shutdown::SyncShutdown;
//!
//! struct SimpleComponent;
//!
//! impl SyncShutdown for SimpleComponent {
//!     type Result = ();
//!
//!     fn shutdown(&mut self) -> Self::Result {
//!         // Cleanup resources synchronously
//!         println!("Component shut down");
//!     }
//! }
//! ```
//!
//! ### Asynchronous Shutdown
//! For components that need to perform async operations during shutdown:
//! ```rust,ignore
//! use tucano_core::shutdown::AsyncShutdown;
//!
//! struct AsyncComponent;
//!
//! impl AsyncShutdown for AsyncComponent {
//!     type Result = Result<(), String>;
//!
//!     fn shutdown(&mut self) -> impl Future<Output = Self::Result> {
//!         async move {
//!             // Async cleanup operations
//!             tokio::time::sleep(std::time::Duration::from_millis(100)).await;
//!             Ok(())
//!         }
//!     }
//! }
//! ```
//!
//! ## Shutdown Signal
//!
//! The `Shutdown` type serves as a signal that can be sent through the event system
//! to trigger graceful shutdown of the entire trading system.

use serde::{Deserialize, Serialize};
use std::future::Future;

/// Trait for components that can be shut down synchronously.
///
/// Implement this trait for components that can complete their shutdown
/// operations immediately without requiring async operations.
pub trait SyncShutdown {
    /// The result type returned by the shutdown operation
    type Result;
    /// Performs synchronous shutdown of the component
    fn shutdown(&mut self) -> Self::Result;
}

/// Trait for components that require asynchronous shutdown operations.
///
/// Implement this trait for components that need to perform async operations
/// during shutdown, such as flushing buffers, closing network connections,
/// or waiting for pending operations to complete.
pub trait AsyncShutdown {
    /// The result type returned by the shutdown operation
    type Result;
    /// Performs asynchronous shutdown of the component
    fn shutdown(&mut self) -> impl Future<Output = Self::Result>;
}

/// A shutdown signal that can be sent through the event system.
///
/// This type serves as a marker indicating that a graceful shutdown
/// should be initiated. It can be used in event streams to signal
/// that all components should begin their shutdown procedures.
///
/// ## Usage
/// ```rust,ignore
/// use tucano_core::{EngineEvent, shutdown::Shutdown};
///
/// // Create a shutdown event
/// let shutdown_event = EngineEvent::shutdown();
///
/// // Or create the shutdown signal directly
/// let shutdown_signal = Shutdown;
/// ```
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize, Serialize,
)]
pub struct Shutdown;

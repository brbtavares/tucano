//! # Logging Configuration
//!
//! This module provides standardized logging configuration for the Toucan trading framework.
//! It sets up structured logging with proper filtering to avoid noise from internal audit operations.
//!
//! ## Features
//!
//! - **Configurable Output**: Supports both human-readable and JSON log formats
//! - **Environment-based Filtering**: Uses `RUST_LOG` environment variable for log level control
//! - **Audit Noise Filtering**: Automatically filters out verbose audit replica state updates
//! - **Default INFO Level**: Provides sensible defaults while allowing customization
//!
//! ## Usage
//!
//! ### Standard Logging (Human-readable)
//! ```rust
//! use core::logging::init_logging;
//!
//! fn main() {
//!     init_logging();
//!     tracing::info!("Trading system started");
//! }
//! ```
//!
//! ### JSON Logging (For log aggregation systems)
//! ```rust
//! use core::logging::init_json_logging;
//!
//! fn main() {
//!     init_json_logging();
//!     tracing::info!("Trading system started");
//! }
//! ```
//!
//! ### Environment Configuration
//! ```bash
//! # Set log level to debug for all modules
//! export RUST_LOG=debug
//!
//! # Set specific log levels per module
//! export RUST_LOG=core=info,execution=debug,data=warn
//! ```

use crate::engine::audit::state_replica::AUDIT_REPLICA_STATE_UPDATE_SPAN_NAME;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialise default non-JSON `Toucan` logging.
///
/// Note that this filters out duplicate logs produced by the `AuditManager` updating its replica
/// `EngineState`.
pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(AuditSpanFilter)
        .init()
}

/// Initialise default JSON `Toucan` logging.
///
/// Note that this filters out duplicate logs produced by the `AuditManager` updating its replica
/// `EngineState`.
pub fn init_json_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
        .with(AuditSpanFilter)
        .init()
}

struct AuditSpanFilter;

impl<S> tracing_subscriber::layer::Layer<S> for AuditSpanFilter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn event_enabled(
        &self,
        _: &tracing::Event<'_>,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        if let Some(span) = ctx.lookup_current() {
            if span.name() == AUDIT_REPLICA_STATE_UPDATE_SPAN_NAME {
                false
            } else {
                true
            }
        } else {
            true
        }
    }
}

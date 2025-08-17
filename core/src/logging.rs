// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! # Logging Configuration
//!
//! Standardized logging configuration module for the Tucano framework (formerly Toucan).
//! Provides structured logs with filters to reduce internal audit noise.
//!
//! ## Features
//!
//! - **Configurable Output**: Human-readable or JSON format
//! - **Environment Filtering**: Uses `RUST_LOG` variable for levels
//! - **Audit Noise Filter**: Removes verbose state replica updates
//! - **Default INFO Level**: Adjustable as needed
//!
//! ## Usage
//!
//! ### Standard Logging (human-readable)
//! ```rust,ignore
//! use tucano_core::logging::init_logging; // crate path when used externally
//!
//! fn main() {
//!     init_logging();
//!     tracing::info!("Trading system started");
//! }
//! ```
//!
//! ### JSON Logging (aggregators / observability)
//! ```rust,ignore
//! use tucano_core::logging::init_json_logging; // external crate path
//!
//! fn main() {
//!     init_json_logging();
//!     tracing::info!("Trading system started");
//! }
//! ```
//!
//! ### Environment Configuration
//! ```bash
//! # Debug level for all modules
//! export RUST_LOG=debug
//!
//! # Níveis específicos por módulo
//! export RUST_LOG=core=info,execution=debug,data=warn
//! ```

use crate::engine::audit::state_replica::AUDIT_REPLICA_STATE_UPDATE_SPAN_NAME;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Inicializa logging padrão não‑JSON do `Tucano`.
///
/// Filtra logs duplicados gerados pelo `AuditManager` ao atualizar sua réplica de `EngineState`.
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

/// Inicializa logging JSON do `Tucano`.
///
/// Filtra logs duplicados gerados pelo `AuditManager` ao atualizar sua réplica de `EngineState`.
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
            span.name() != AUDIT_REPLICA_STATE_UPDATE_SPAN_NAME
        } else {
            true
        }
    }
}

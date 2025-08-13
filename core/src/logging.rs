//! # Configuração de Logging
//!
//! Módulo de configuração padronizada de logging do framework Tucano (ex-Toucan).
//! Fornece logs estruturados com filtros para reduzir ruído de auditoria interna.
//!
//! ## Funcionalidades
//!
//! - **Saída Configurável**: Formato humano ou JSON
//! - **Filtro por Ambiente**: Usa variável `RUST_LOG` para níveis
//! - **Filtro de Ruído de Auditoria**: Remove atualizações verbosas de réplica de estado
//! - **Nível INFO Padrão**: Ajustável conforme necessidade
//!
//! ## Uso
//!
//! ### Logging Padrão (legível humano)
//! ```rust
//! use core::logging::init_logging;
//!
//! fn main() {
//!     init_logging();
//!     tracing::info!("Sistema de trading iniciado");
//! }
//! ```
//!
//! ### Logging JSON (agregadores / observabilidade)
//! ```rust
//! use core::logging::init_json_logging;
//!
//! fn main() {
//!     init_json_logging();
//!     tracing::info!("Sistema de trading iniciado");
//! }
//! ```
//!
//! ### Configuração via Ambiente
//! ```bash
//! # Nível debug para todos os módulos
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

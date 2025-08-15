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
// Dependências são usadas por crates downstream via tipos genéricos; suprimir para evitar falso positivo
#![allow(unused_crate_dependencies)]
#![allow(clippy::type_complexity, clippy::too_many_arguments, type_alias_bounds)]

//! DISCLAIMER (resumo): Uso educacional/experimental. Sem recomendação de investimento.
//! Sem afiliação institucional ou remuneração de terceiros. Profit/ProfitDLL © Nelógica.
//! Integração técnica. README & DISCLAIMER.
//! Core abstractions para estratégias de trading (traits + tipos). Implementações concretas vão
//! para a crate `strategies`.

pub mod algo;
pub mod close_positions;
pub mod on_disconnect;
pub mod on_trading_disabled;

pub use algo::AlgoStrategy;
pub use close_positions::ClosePositionsStrategy;
pub use on_disconnect::OnDisconnectStrategy;
pub use on_trading_disabled::OnTradingDisabled;

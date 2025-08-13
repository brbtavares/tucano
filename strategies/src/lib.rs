//! DISCLAIMER: Uso experimental/educacional. Não é recomendação de investimento. Veja README e DISCLAIMER.md.
//! Estratégias concretas agrupadas (ativadas via feature flags).

pub mod shared;

pub mod order_book_imbalance; // sempre disponível: exemplo simples reutilizável

#[cfg(feature = "momentum")]
pub mod momentum;

#[cfg(feature = "mean_rev")]
pub mod mean_reversion;

#[cfg(feature = "microstructure")]
pub mod microstructure;

#[cfg(feature = "options")]
pub mod options;

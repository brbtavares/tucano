//! Estratégias concretas agrupadas (ativadas via feature flags).

pub mod shared;

#[cfg(feature = "momentum")]
pub mod momentum;

#[cfg(feature = "mean_rev")]
pub mod mean_reversion;

#[cfg(feature = "microstructure")]
pub mod microstructure;

#[cfg(feature = "options")]
pub mod options;

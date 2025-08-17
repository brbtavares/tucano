//! DISCLAIMER (summary): Educational/experimental use only. No investment advice.
//! No institutional affiliation or third-party compensation. Profit/ProfitDLL © Nelógica.
//! Technical integration. See README & DISCLAIMER.
//! Grouped concrete strategies (enabled via feature flags).

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

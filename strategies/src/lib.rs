// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
//! Grouped concrete strategies (enabled via feature flags).

pub mod shared;

pub mod order_book_imbalance; // always available: simple reusable example

#[cfg(feature = "momentum")]
pub mod momentum;

#[cfg(feature = "mean_rev")]
pub mod mean_reversion;

#[cfg(feature = "microstructure")]
pub mod microstructure;

#[cfg(feature = "options")]
pub mod options;

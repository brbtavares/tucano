pub mod util;

pub use util::*;

use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// General interface for implementing simple RiskManager checks.
///
/// See [`CheckHigherThan`] for a simple example.
///
/// # Associated Types
/// * `Input` - The type of data being validated (e.g., `Decimal` for price checks)
/// * `Error` - The error type returned when validation fails
pub trait RiskCheck {
    type Input;
    type Error;

    /// Returns the name of the risk check.
    fn name() -> &'static str;

    /// Performs the risk check on the provided `Input`.
    fn check(&self, input: &Self::Input) -> Result<(), Self::Error>;
}

/// General risk check that validates if an input value exceeds an upper limit.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, Constructor)]
pub struct CheckHigherThan<T> {
    /// The upper limit value; check passes if input is <= limit.
    pub limit: T,
}

impl<T> RiskCheck for CheckHigherThan<T>
where
    T: Clone + PartialOrd,
{
    type Input = T;
    type Error = CheckHigherThanError<T>;

    fn name() -> &'static str {
        "CheckHigherThan"
    }

    fn check(&self, input: &Self::Input) -> Result<(), Self::Error> {
        if input > &self.limit {
            Err(CheckHigherThanError {
                input: input.clone(),
                limit: self.limit.clone(),
            })
        } else {
            Ok(())
        }
    }
}

/// Error returned when a [`CheckHigherThan`] validation fails.
#[derive(
    Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Error, Constructor,
)]
#[error("CheckHigherThan failed: input {input:?} > limit {limit:?}")]
pub struct CheckHigherThanError<T> {
    pub input: T,
    pub limit: T,
}

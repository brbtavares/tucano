/// Statistical algorithms for analysing datasets.
pub mod algorithm;

/// Financial metrics and the means to calculate them over different
/// [`TimeIntervals`](time::TimeInterval).
pub mod metric;

/// Statistical summaries for financial datasets.
///
/// For example, `TradingSummary`, `TearSheet`, `TearSheetAsset`, `PnLReturns`, etc.
pub mod summary;

/// TimeInterval definitions used for financial calculations.
///
/// For example, `Annual365`, `Annual252`, `Daily`, etc.
pub mod time;

use chrono::{DateTime, Utc};

/// Trait for types that have a timestamp.
pub trait Timed {
    /// Returns the timestamp of this item.
    fn timestamp(&self) -> DateTime<Utc>;
}

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

/// A wrapper struct that combines a value with a timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct TimedValue<T> {
    pub value: T,
    pub timestamp: DateTime<Utc>,
}

impl<T> TimedValue<T> {
    pub fn new(value: T, timestamp: DateTime<Utc>) -> Self {
        Self { value, timestamp }
    }
}

impl<T> Timed for TimedValue<T> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

#[cfg(test)]
pub mod test_utils {
    use chrono::{DateTime, Utc};

    pub fn time_plus_days(base: DateTime<Utc>, plus: u64) -> DateTime<Utc> {
        base + chrono::Duration::days(plus as i64)
    }
}


//!
//! # 📊 Analytics - Financial Analysis Module
//!
//! This module provides comprehensive tools for quantitative analysis of financial data,
//! including performance metrics, statistical algorithms, and report generation.
//!
//! ## 🎯 Main Features
//!
//! - **Financial Metrics**: Sharpe, Sortino, Calmar, Win Rate, Profit Factor
//! - **Drawdown Analysis**: Calculation of maximum and average drawdown
//! - **Statistical Algorithms**: Processing of financial datasets
//! - **Automated Reports**: Generation of summaries and tear sheets
//! - **Time Intervals**: Support for different analysis periods
//!
//! ## 🏗️ Structure (simplified)
//! Main files: `algorithm.rs`, `metric/` directory, `summary/` directory, `time.rs`.
//! ## 🏗️ Module Structure
//!
//! (Illustrative diagram – not executable code)
//!
//! ```text
//! analytics/
//!  ├─ algorithm.rs     # Statistical algorithms for dataset analysis
//!  ├─ metric/          # Financial metrics (Sharpe, Sortino, etc.)
//!  ├─ summary/         # Financial reports and summaries
//!  └─ time.rs          # Time interval definitions
//! ```
//!
//! ## 📈 Simplified Usage Example
//!
//! Sharpe Ratio calculation with hypothetical values (returns already aggregated).
//!
//! Simple Sharpe Ratio calculation using pre-computed statistics from a return series:
//!
//! ```rust
//! use toucan_analytics::metric::sharpe::SharpeRatio;
//! use rust_decimal_macros::dec;
//! use chrono::TimeDelta;
//!
//! // Return statistics (fictitious example)
//! let risk_free_return = dec!(0.0015);    // 0.15%
//! let mean_return      = dec!(0.0025);    // 0.25%
//! let std_dev_returns  = dec!(0.02);      // 2%
//! let interval = TimeDelta::hours(2);     // analyzed period
//!
//! let sharpe = SharpeRatio::calculate(risk_free_return, mean_return, std_dev_returns, interval);
//! assert!(sharpe.value != rust_decimal::Decimal::ZERO);
//! ```
//!
//! Another example calculating Sharpe on a daily basis:
//!
//! ```rust
//! use toucan_analytics::metric::sharpe::SharpeRatio;
//! use toucan_analytics::time::Daily;
//! use rust_decimal_macros::dec;
//!
//! let risk_free = dec!(0.0015);    // 0.15%
//! let mean_ret  = dec!(0.0025);    // 0.25%
//! let std_dev   = dec!(0.0200);    // 2.00%
//!
//! let sharpe = SharpeRatio::calculate(risk_free, mean_ret, std_dev, Daily);
//! assert_eq!(sharpe.value, dec!(0.05));
//! ```
//!
//! ## 🔍 Available Metrics
//!
//! - **Sharpe Ratio**: Risk-adjusted return
//! - **Sortino Ratio**: Sharpe considering only downside risk
//! - **Calmar Ratio**: Annualized return / maximum drawdown
//! - **Win Rate**: Percentage of winning trades
//! - **Profit Factor**: Gross profit / gross loss
//! - **Drawdown**: Analysis of maximum and average losses

/// Statistical algorithms for financial dataset analysis.
///
/// Contains implementations of algorithms for processing and analyzing
/// financial data, including calculations of volatility, correlation,
/// and other fundamental statistical metrics.
pub mod algorithm;

/// Financial metrics and methods for calculating them in different
/// [`TimeIntervals`](time::TimeInterval).
///
/// Includes all essential metrics for quantitative analysis:
/// Sharpe, Sortino, Calmar ratios, Win Rate, Profit Factor, and drawdown
/// analyses for evaluating strategy performance.
pub mod metric;

/// Statistical summaries for financial datasets.
///
/// Provides structures for generating comprehensive reports such as
/// `TradingSummary`, `TearSheet`, `TearSheetAsset`, `PnLReturns`, etc.
/// Essential for performance analysis and automated reporting.
pub mod summary;

/// Definitions of time intervals used in financial calculations.
///
/// Supports different financial time conventions such as `Annual365`,
/// `Annual252` (business days), `Daily`, etc. for precise calculations
/// of annualized and periodic metrics.
pub mod time;

use chrono::{DateTime, Utc};

/// Trait for types that have a timestamp.
///
/// Defines the standard interface for objects that carry temporal information,
/// essential for time-based analysis and chronological ordering.
pub trait Timed {
    /// Returns the timestamp of this item.
    fn timestamp(&self) -> DateTime<Utc>;
}

/// Wrapper structure that combines a value with a timestamp.
///
/// Useful for associating financial data with their specific timestamps,
/// allowing precise temporal analysis and chronological ordering.
///
/// # Example
/// ```rust
/// use toucan_analytics::{TimedValue, Timed};
/// use chrono::Utc;
///
/// let price = TimedValue::new(100.50_f64, Utc::now());
/// assert!(price.timestamp() <= Utc::now());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TimedValue<T> {
    /// The value associated with the timestamp
    pub value: T,
    /// UTC timestamp of the value
    pub timestamp: DateTime<Utc>,
}

impl<T> TimedValue<T> {
    /// Creates a new `TimedValue` with the provided value and timestamp.
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
    //! Utilities for testing the analytics module.
    //!
    //! Provides helper functions for creating test data
    //! and manipulating time in test scenarios.

    use chrono::{DateTime, Utc};

    /// Adds days to a base date for creating test data.
    ///
    /// Useful for generating test time series with specific intervals
    /// between observations.
    pub fn time_plus_days(base: DateTime<Utc>, plus: u64) -> DateTime<Utc> {
        base + chrono::Duration::days(plus as i64)
    }
}

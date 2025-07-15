# Toucan Analytics

Statistical analysis and metrics module for the Toucan trading engine.

## Overview

This crate provides statistical algorithms, financial metrics, and analytical tools for the Toucan trading system. It includes:

- Statistical algorithms for dataset analysis
- Financial metrics (Sharpe ratio, Sortino ratio, Calmar ratio, etc.)
- Trading performance summaries and tear sheets
- Time interval definitions for financial calculations
- Drawdown analysis tools

## Features

- **Metrics**: Comprehensive financial metrics for portfolio analysis
- **Summaries**: Trading performance summaries and tear sheets
- **Algorithms**: Statistical algorithms for data analysis
- **Time Intervals**: Support for various time periods (Daily, Annual, etc.)

## Usage

```rust
use toucan_analytics::{
    metric::sharpe::SharpeRatio,
    time::Daily,
    summary::TradingSummary,
};

// Calculate Sharpe ratio
let sharpe = SharpeRatio::calculate(&returns, &Daily::default(), risk_free_rate)?;

// Generate trading summary
let summary = TradingSummary::new(&trades);
```

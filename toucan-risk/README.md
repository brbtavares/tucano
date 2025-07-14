# Toucan Risk

Risk management module for the Toucan trading engine.

## Overview

This crate provides the risk management interfaces and utilities for order validation in the Toucan trading system. It includes:

- `RiskManager` trait for implementing risk checks
- `RiskApproved` and `RiskRefused` wrapper types for risk decisions
- Validation utilities for order checking

## Usage

```rust
use toucan_risk::{RiskManager, RiskApproved, RiskRefused};

// Implement your risk manager
struct MyRiskManager;

impl RiskManager for MyRiskManager {
    // ... implementation
}
```

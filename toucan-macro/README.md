# Toucan Macro

Procedural macros for the Toucan trading framework.

## Overview

This crate provides derive macros for common serialization and deserialization patterns used throughout the Toucan ecosystem, particularly for exchange and subscription kind handling.

## Available Macros

### Exchange Macros

- **`#[derive(DeExchange)]`** - Implements `serde::Deserialize` for exchange types
- **`#[derive(SerExchange)]`** - Implements `serde::Serialize` for exchange types

These macros expect the target struct to have an `ID` constant that provides the string representation.

### Subscription Kind Macros

- **`#[derive(DeSubKind)]`** - Implements `serde::Deserialize` for subscription kind types
- **`#[derive(SerSubKind)]`** - Implements `serde::Serialize` for subscription kind types

These macros automatically convert between PascalCase type names and snake_case string representations.

## Usage

```rust
use toucan_macro::{DeExchange, SerExchange, DeSubKind, SerSubKind};

#[derive(DeExchange, SerExchange)]
struct Binance;

impl Binance {
    const ID: &'static str = "binance";
}

#[derive(DeSubKind, SerSubKind)]
struct OrderBookL1;
```

## Dependencies

- `proc-macro2` - Token manipulation
- `quote` - Code generation
- `syn` - Rust AST parsing
- `convert_case` - Case conversion utilities
- `serde` - Serialization framework

## License

This project is part of the Toucan trading framework.

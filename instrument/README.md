# toucan-instrument

[![Crates.io](https://img.shields.io/crates/v/toucan-instrument.svg)](https://crates.io/crates/toucan-instrument)
[![Docs.rs](https://docs.rs/toucan-instrument/badge.svg)](https://docs.rs/toucan-instrument)

Instrument and exchange key abstractions for the Toucan trading framework.

---

## Overview
This crate provides core types and traits for representing financial instruments, exchanges, and related keys. It is a foundational building block for other Toucan ecosystem crates, ensuring consistent modeling of assets, instruments, and exchange identifiers.

## Features
- Instrument, Exchange, and Asset key types
- Serialization and deserialization support (via serde)
- Utilities for working with instrument identifiers

## Example
```rust
use toucan_instrument::{Instrument, ExchangeKey, AssetKey};

let exchange = ExchangeKey::from("binance");
let asset = AssetKey::from("btc");
let instrument = Instrument::new(exchange.clone(), asset.clone(), AssetKey::from("usdt"));
```

## Usage
Add to your `Cargo.toml`:

```toml
[dependencies]
toucan-instrument = { path = "../instrument" }
```

## License
Apache-2.0 OR MIT

See the [root README](../README.md) and [DISCLAIMER](../DISCLAIMER.md) for more information.

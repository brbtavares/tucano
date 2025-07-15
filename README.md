# Toucan Trading Framework

A comprehensive Rust framework for building high-performance live-trading, paper-trading, and back-testing systems for cryptocurrency markets.

[![Documentation](https://img.shields.io/badge/docs-wiki-blue)](https://github.com/brbtavares/toucan/wiki)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.82+-orange)](https://www.rust-lang.org/)

## Overview

Toucan is a modular, high-performance trading framework designed for cryptocurrency markets. It provides a complete ecosystem for:

- **Live Trading**: Real-time order execution across multiple exchanges
- **Paper Trading**: Risk-free strategy testing with simulated execution
- **Backtesting**: Historical strategy validation with comprehensive analytics
- **Market Data**: Real-time WebSocket streams from leading exchanges
- **Risk Management**: Configurable risk controls and position management
- **Analytics**: Comprehensive performance metrics and reporting

## Key Features

- **🚀 High Performance**: Built in Rust with zero-cost abstractions and minimal allocations
- **🔒 Type Safety**: Strongly typed architecture preventing runtime errors
- **🔄 Multi-Exchange**: Unified interface for trading across different exchanges
- **📊 Real-Time Data**: WebSocket integrations for live market data
- **🎯 Modular Design**: Plugin architecture for strategies, risk managers, and execution
- **� Comprehensive Analytics**: Built-in performance metrics and reporting
- **🛡️ Risk Management**: Configurable risk controls and position limits
- **🔧 Extensible**: Easy to add new exchanges, strategies, and features

## Architecture

The Toucan framework consists of several interconnected crates:

### Core Components

- **[`core`](https://github.com/brbtavares/toucan/wiki/Core)** - Main trading engine and orchestration
- **[`data`](https://github.com/brbtavares/toucan/wiki/Data)** - Market data streaming and normalization
- **[`execution`](https://github.com/brbtavares/toucan/wiki/Execution)** - Order execution and exchange connectivity
- **[`analytics`](https://github.com/brbtavares/toucan/wiki/Analytics)** - Performance metrics and statistical analysis

### Supporting Components

- **[`instrument`](https://github.com/brbtavares/toucan/wiki/Instrument)** - Financial instrument definitions and indexing
- **[`integration`](https://github.com/brbtavares/toucan/wiki/Integration)** - WebSocket and HTTP client utilities
- **[`strategy`](https://github.com/brbtavares/toucan/wiki/Strategy)** - Trading strategy interfaces and implementations
- **[`risk`](https://github.com/brbtavares/toucan/wiki/Risk)** - Risk management and position controls
- **[`macro`](https://github.com/brbtavares/toucan/wiki/Macro)** - Procedural macros for code generation

## Supported Exchanges

### Market Data Streaming

| Exchange | Spot | Futures | Options | Perpetuals |
|----------|------|---------|---------|------------|
| **Binance** | ✅ | ✅ | ❌ | ✅ |
| **Bybit** | ✅ | ❌ | ❌ | ✅ |
| **Coinbase** | ✅ | ❌ | ❌ | ❌ |
| **Kraken** | ✅ | ❌ | ❌ | ❌ |
| **OKX** | ✅ | ✅ | ✅ | ✅ |
| **Gate.io** | ✅ | ✅ | ✅ | ✅ |
| **Bitmex** | ❌ | ❌ | ❌ | ✅ |
| **Bitfinex** | ✅ | ❌ | ❌ | ❌ |

### Order Execution

| Exchange | Trading | Authentication | WebSocket | REST API |
|----------|---------|----------------|-----------|----------|
| **Binance** | ✅ | ✅ | ✅ | ✅ |

> **Note**: More exchanges are actively being added. See the [Exchange Integration Guide](https://github.com/brbtavares/toucan/wiki/Exchange-Integration) for details.

## Quick Start

### Installation

Add Toucan to your `Cargo.toml`:

```toml
[dependencies]
toucan-core = "0.12"
toucan-data = "0.10"
toucan-execution = "0.5"
toucan-analytics = "0.1"
```

### Basic Usage

```rust
use toucan_core::prelude::*;
use toucan_data::exchange::binance::BinanceSpot;
use toucan_strategy::AlgoStrategy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the trading engine
    let mut engine = Engine::builder()
        .with_exchange(BinanceSpot::default())
        .with_strategy(MyStrategy::default())
        .build()
        .await?;

    // Start live trading
    engine.run().await?;
    
    Ok(())
}
```

### Examples

The framework includes comprehensive examples:

- **[Live Trading](core/examples/engine_sync_with_live_market_data_and_mock_execution_and_audit.rs)** - Real-time trading with live market data
- **[Backtesting](core/examples/backtests_concurrent.rs)** - Historical strategy validation
- **[Market Data Streaming](data/examples/multi_stream_multi_exchange.rs)** - Multi-exchange data feeds
- **[Risk Management](core/examples/engine_sync_with_risk_manager_open_order_checks.rs)** - Risk controls implementation
- **[Strategy Development](core/examples/engine_sync_with_multiple_strategies.rs)** - Custom strategy examples

## Documentation

Comprehensive documentation is available in the [Project Wiki](https://github.com/brbtavares/toucan/wiki):

### Getting Started
- [Installation Guide](https://github.com/brbtavares/toucan/wiki/Installation)
- [Quick Start Tutorial](https://github.com/brbtavares/toucan/wiki/Quick-Start)
- [Architecture Overview](https://github.com/brbtavares/toucan/wiki/Architecture)

### Components
- [Core Engine](https://github.com/brbtavares/toucan/wiki/Core)
- [Market Data](https://github.com/brbtavares/toucan/wiki/Data)
- [Order Execution](https://github.com/brbtavares/toucan/wiki/Execution)
- [Analytics & Metrics](https://github.com/brbtavares/toucan/wiki/Analytics)

### Development
- [Strategy Development](https://github.com/brbtavares/toucan/wiki/Strategy-Development)
- [Risk Management](https://github.com/brbtavares/toucan/wiki/Risk-Management)
- [Exchange Integration](https://github.com/brbtavares/toucan/wiki/Exchange-Integration)
- [Contributing Guide](https://github.com/brbtavares/toucan/wiki/Contributing)

## Performance

Toucan is designed for high-performance trading with:

- **Low Latency**: Sub-millisecond order processing
- **High Throughput**: Thousands of orders per second
- **Memory Efficient**: Minimal allocations and zero-copy operations
- **Scalable**: Multi-threaded architecture with async I/O

## Development

### Building

```bash
# Clone the repository
git clone https://github.com/brbtavares/toucan.git
cd toucan

# Build all crates
cargo build --release

# Run tests
cargo test

# Run examples
cargo run --example engine_sync_with_live_market_data_and_mock_execution_and_audit
```

### Project Structure

```
toucan/
├── analytics/          # Performance metrics and analytics
├── core/               # Main trading engine
├── data/               # Market data streaming
├── execution/          # Order execution and exchange APIs
├── instrument/         # Financial instrument definitions
├── integration/        # WebSocket and HTTP utilities
├── macro/              # Procedural macros
├── risk/               # Risk management
├── strategy/           # Trading strategy interfaces
└── examples/           # Usage examples
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/brbtavares/toucan/wiki/Contributing) for details on:

- Code of Conduct
- Development Process
- Testing Requirements
- Documentation Standards

## Support

- **Documentation**: [Project Wiki](https://github.com/brbtavares/toucan/wiki)
- **Issues**: [GitHub Issues](https://github.com/brbtavares/toucan/issues)
- **Discussions**: [GitHub Discussions](https://github.com/brbtavares/toucan/discussions)

## Roadmap

See our [Project Roadmap](https://github.com/brbtavares/toucan/wiki/Roadmap) for upcoming features and improvements.

## Credits

This project is based on the excellent [Barter-rs](https://github.com/barter-rs/barter-rs) project. See [CREDITS.md](CREDITS.md) for full attribution and acknowledgments to the original developers.

---

**⚠️ Disclaimer**: This software is for educational and research purposes. Always test thoroughly before using in production trading environments. Trading involves significant risk of loss.

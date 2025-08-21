<div align="center">

# Toucan - Algorithmic Trading Framework for Global Markets


<table>
    <tr>
        <td align="center" valign="middle" style="border: none;">
            <img src="assets/logo.png" alt="Logo Toucan" />
        </td>
    </tr>
    <tr>
        <td align="left" valign="middle" style="border: none; padding-left: 16px;">
            <li><strong>Keen vision & strategic reach</strong>: monitors multiple markets/order books in real time to anticipate movement.</li>
            <li><strong>Precision & efficiency</strong>: sends lean orders, avoids latency, and reduces operational friction.</li>
            <li><strong>Navigation in complex environments</strong>: abstracts protocols, streams, and heterogeneous formats.</li>
            <li><strong>Adaptive intelligence</strong>: adjusts parameters & strategies according to market regime.</li>
            <li><strong>Panoramic view</strong>: consolidates multi-source data for holistic decision-making (price, volume, risk, PnL).</li>
            <li><strong>Fast response</strong>: optimized event loop to react to micro-variations before the competition.</li>
        </td>
    </tr>
</table>




[![Rust Version](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Docs](https://img.shields.io/docsrs/toucan)](https://docs.rs/toucan)
[![CI](https://github.com/brbtavares/toucan/actions/workflows/ci.yml/badge.svg)](https://github.com/brbtavares/toucan/actions)

</div>

## 🎯 Key Features

    **🌐 Multi-market Ready**: Designed for integration with any exchange or broker (WebSocket, REST, DLL, etc.)
| `toucan` | 0.1.x | Unified façade (re-export) | [docs.rs](https://docs.rs/toucan) |
| `toucan-core` | 0.12.x | Engine, unified execution/backtest | [docs.rs](https://docs.rs/toucan-core) |
| `toucan-markets` | 0.3.x | Instruments & exchanges (multi-market) | [docs.rs](https://docs.rs/toucan-markets) |
| `toucan-data` | 0.10.x | Events & data pipeline (streams, books, trades) | [docs.rs](https://docs.rs/toucan-data) |
| `toucan-execution` | 0.5.x | Orders, fills, routing & clients | [docs.rs](https://docs.rs/toucan-execution) |
| `toucan-trader` | 0.1.x | Core strategy traits & types | [docs.rs](https://docs.rs/toucan-trader) |
| `toucan-risk` | 0.1.x | Risk management (limits, validations) | [docs.rs](https://docs.rs/toucan-risk) |
| `toucan-strategies` | 0.1.x | Example/reference strategies | [docs.rs](https://docs.rs/toucan-strategies) |
| `toucan-analytics` | 0.1.x | Metrics, summaries, performance & PnL | [docs.rs](https://docs.rs/toucan-analytics) |
| `toucan-integration` | 0.9.x | External protocols, channels & snapshots | [docs.rs](https://docs.rs/toucan-integration) |
| `toucan-macros` | 0.2.x | Internal procedural macros | [docs.rs](https://docs.rs/toucan-macros) |
| `toucan-risk` | 0.1.x | Risk management (limits, validations) | [docs.rs](https://docs.rs/toucan-risk) |
| `toucan-strategies` | 0.1.x | Example/reference strategies | [docs.rs](https://docs.rs/toucan-strategies) |
| `toucan-analytics` | 0.1.x | Metrics, summaries, performance & PnL | [docs.rs](https://docs.rs/toucan-analytics) |
| `toucan-integration` | 0.9.x | External protocols, channels & snapshots | [docs.rs](https://docs.rs/toucan-integration) |
| `toucan-macros` | 0.2.x | Internal procedural macros | [docs.rs](https://docs.rs/toucan-macros) |

Convention: use `major.minor.x` range in docs; specify patch for reproducibility if needed.


## 🏗️ System Architecture

Toucan is organized as a modular Rust workspace, where each crate is responsible for a specific domain of algorithmic trading. This design enables easy extension, testing, and integration with new markets, protocols, and strategies. Below is an overview of the main components and their roles:

**Workspace Structure:**

```
├── toucan/                # Unified façade crate (re-exports)
├── core/                  # Core engine: event loop, unified execution/backtest logic
├── execution/             # Order management, routing, fills, and client abstractions
├── data/                  # Market data events, streaming, books, trades
├── markets/               # Instrument definitions, exchange adapters, market-specific logic
├── analytics/             # Metrics, summaries, performance, PnL calculations
├── risk/                  # Risk management: limits, validations, risk checks
├── trader/                # Core traits and types for strategy development
├── strategies/            # Example/reference strategies (plug-and-play)
├── integration/           # External protocol adapters, channels, snapshots
├── macros/                # Internal procedural macros for code generation

├── examples/              # Usage examples, integration demos
├── devkit/                # Developer scripts and utilities
```


**Component Roles:**

- **toucan**: The main entry point. Re-exports all core modules for easy consumption.
- **core**: The heart of the framework. Implements the event-driven engine, supporting both live trading and backtesting with the same codebase.
- **execution**: Handles order creation, routing, execution, and client abstraction for different exchanges.
- **data**: Manages all market data streams, order books, trades, event normalization, and now contains all concrete exchange/broker integrations.
- **markets**: Defines financial instruments (stocks, futures, options, crypto, etc.) and provides only abstractions (traits, enums, types) for supported exchanges/markets.
- **analytics**: Provides performance metrics, summaries, and reporting tools for strategies and portfolios.
- **risk**: Centralizes risk checks, position limits, and validation logic to ensure safe trading.
- **trader**: Contains the main traits and types for implementing trading strategies in a generic, engine-agnostic way.
- **strategies**: Houses reusable and reference strategies, which can be used as templates or directly in production.
- **integration**: Adapters for external protocols (WebSocket, REST, FIX, DLL, etc.), channels, and snapshotting tools. Concrete adapters are now implemented here or in `data/` as appropriate.
- **macros**: Internal procedural macros to reduce boilerplate and enable advanced code generation.
- **examples**: Real-world usage examples, integration tests, and demos for new users.
- **devkit**: Scripts and utilities to help with development, CI/CD, and code quality.

**Extending the Framework:**

- To add support for a new market or protocol, create a new crate (e.g., `toucan-binance`, `toucan-ibrk`, `toucan-kraken`) following the same modular pattern. Implement the required traits from `core`, `execution`, and `data`.
- New strategies can be added to the `strategies/` crate or as separate crates for better isolation.
- All components communicate via strongly-typed events and traits, making it easy to plug in new modules without breaking existing code.


**Note:** All concrete exchange/broker integrations (such as B3/ProfitDLL) are now implemented as local modules in `toucan-data` or `toucan-integration`. The `markets` crate contains only abstractions (traits, enums, types).

This architecture allows you to build, test, and deploy algorithmic trading systems for any market or protocol, while keeping code maintainable and extensible.

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (version 1.75 or higher)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/brbtavares/toucan.git
cd toucan
```

### Add Dependency (façade)

In your `Cargo.toml`, add the unified crate (re-export) — recommended to start:

```toml
[dependencies]
toucan = { version = "0.1", features = ["full" ] }
```

Or, if you want more granularity/compile less, use individual crates:

```toml
[dependencies]
[dependencies]
toucan-core = "0.1"
toucan-markets = "0.1"
toucan-data = "0.1"
toucan-execution = "0.1"
toucan-trader = "0.1"
toucan-risk = "0.1"
toucan-strategies = "0.1"
toucan-analytics = "0.1"
```

Then import via façade:

```rust
use toucan::prelude::*; // Engine, ExchangeId, Side, etc.
```

### Build

```bash
# Build the entire workspace
cargo build --release

# Run tests
cargo test

# Code formatting (uses rustfmt.toml)
cargo fmt

# Check formatting (CI/CD)
cargo fmt --check

# Code linting (uses .config/clippy.toml)
cargo clippy -- -D warnings

# Custom formatting script (devkit crate is not functional yet)
./devkit/scripts/format.sh
./devkit/scripts/format.sh --check

# Run a basic example
cargo run --example basic_usage

# Generate documentation
cargo doc --open
```

## 💡 Usage Example

### Reusable Strategy: Order Book Imbalance

A simple strategy that observes the imbalance between BID and ASK volumes at the best book level. The same implementation can be plugged into both a live engine and a backtest engine without changing the logic.

```rust
use toucan_trader::AlgoStrategy;
use toucan_strategies::{
    order_book_imbalance::OrderBookImbalanceStrategy,
    shared::NoOpState,
};
use toucan_execution::{ExchangeIndex, InstrumentIndex};
use toucan_execution::order::request::{OrderRequestCancel, OrderRequestOpen};

// Lightweight wrapper to demonstrate the trait (would delegate internally to the real strategy).
struct MyImbalance(OrderBookImbalanceStrategy);

impl AlgoStrategy for MyImbalance {
    type State = NoOpState; // engine state (placeholder)

    fn generate_algo_orders(
        &self,
        _state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        // Here we would call self.0.generate_algo_orders(...) when integrated with the real state
        (Vec::<OrderRequestCancel<_, _>>::new(), Vec::<OrderRequestOpen<_, _>>::new())
    }
}
```

### Usage in Live vs Backtest (same strategy)

```rust
// Live
let strategy = MyImbalance(OrderBookImbalanceStrategy::new(Default::default()));
let engine_live = Engine::new(clock, live_state, live_exec_txs, strategy, risk_manager);

// Backtest
let strategy_bt = MyImbalance(OrderBookImbalanceStrategy::new(Default::default()));
let engine_bt = BacktestEngine::new(bt_config, bt_state, bt_exec_txs, strategy_bt, risk_manager_bt);
```

Only the data components (streaming vs historical) and execution (real client vs simulated) change; the strategy remains identical.


## 🛠️ Development

### Useful Commands

```bash
# Code formatting
cargo fmt

# Lint
cargo clippy -- -D warnings

# Generate documentation
cargo doc --open

# Benchmarks
cargo bench

# Mini-disclaimer check (CI fails if missing)
./devkit/scripts/verify_disclaimers.sh
./devkit/scripts/verify_disclaimers.sh --fix  # injects where missing

# Specific tests
cargo test -p core --test engine_tests
```


### Automatic Formatting

The project uses [`rustfmt.toml`](rustfmt.toml) to ensure consistent code style:

- **VS Code**: Auto-format on save (configured in `.vscode/settings.json`)
- **CI/CD**: Automatic check in GitHub Actions
- **Manual**: Run `cargo fmt` to format all code

```bash
# Check if code is formatted (used in CI)
cargo fmt --check

# Format automatically
cargo fmt
```

### Test Structure (example)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_strategy_execution() {
        let mut engine = create_test_engine();
        let market_data = mock_market_data();
        
        let result = engine.process_market_data(market_data).await;
        assert!(result.is_ok());
    }
}
```

### Debugging

```rust
use tracing::{info, warn, error, debug};

// Logging setup
tracing_subscriber::fmt()
    .with_env_filter("toucan=debug")
    .init();

// Logs in code
debug!("Processing order: {:?}", order);
info!("Position updated: {}", position);
warn!("Risk limit approaching: {}", exposure);
```

## 🚀 Deployment

### Production Configuration

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Docker

```dockerfile
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
# (After renaming the binary in the future)
COPY --from=builder /app/target/release/toucan /usr/local/bin/
CMD ["toucan"]
```

### Environment Variables

```bash
# Production
export RUST_ENV=production
export RUST_LOG=info
export B3_USERNAME=usuario_producao
export B3_PASSWORD=senha_producao
export DATABASE_URL=postgresql://user:pass@localhost/toucan
export REDIS_URL=redis://localhost:6379
```

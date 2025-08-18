<div align="center">

# Tucano - Algorithmic Trading Framework for Global Markets


<table>
    <tr>
        <td align="center" valign="middle" style="border: none;">
            <img src="_assets/logo.png" alt="Logo Tucano" />
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
[![Crate](https://img.shields.io/crates/v/tucano.svg)](https://crates.io/crates/tucano)
[![Docs](https://img.shields.io/docsrs/tucano)](https://docs.rs/tucano)
[![CI](https://github.com/brbtavares/tucano/actions/workflows/ci.yml/badge.svg)](https://github.com/brbtavares/tucano/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

</div>

## 🎯 Key Features

    **🌐 Multi-market Ready**: Designed for integration with any exchange or broker (WebSocket, REST, DLL, etc.)

## 📦 Published Crates

| Crate | Version | Description | Docs |
|-------|---------|-------------|------|
| `tucano` | 0.1.x | Unified façade (re-export) | [docs.rs](https://docs.rs/tucano) |
| `tucano-core` | 0.12.x | Engine, unified execution/backtest | [docs.rs](https://docs.rs/tucano-core) |
| `tucano-markets` | 0.3.x | Instruments & exchanges (multi-market) | [docs.rs](https://docs.rs/tucano-markets) |
| `tucano-data` | 0.10.x | Events & data pipeline (streams, books, trades) | [docs.rs](https://docs.rs/tucano-data) |
| `tucano-execution` | 0.5.x | Orders, fills, routing & clients | [docs.rs](https://docs.rs/tucano-execution) |
| `tucano-trader` | 0.1.x | Core strategy traits & types | [docs.rs](https://docs.rs/tucano-trader) |
| `tucano-risk` | 0.1.x | Risk management (limits, validations) | [docs.rs](https://docs.rs/tucano-risk) |
| `tucano-strategies` | 0.1.x | Example/reference strategies | [docs.rs](https://docs.rs/tucano-strategies) |
| `tucano-analytics` | 0.1.x | Metrics, summaries, performance & PnL | [docs.rs](https://docs.rs/tucano-analytics) |
| `tucano-integration` | 0.9.x | External protocols, channels & snapshots | [docs.rs](https://docs.rs/tucano-integration) |
| `tucano-macros` | 0.2.x | Internal procedural macros | [docs.rs](https://docs.rs/tucano-macros) |

Convention: use `major.minor.x` range in docs; specify patch for reproducibility if needed.

## 🏗️ System Architecture

Tucano is organized as a modular Rust workspace, where each crate is responsible for a specific domain of algorithmic trading. This design enables easy extension, testing, and integration with new markets, protocols, and strategies. Below is an overview of the main components and their roles:

**Workspace Structure:**

```
├── tucano/                # Unified façade crate (re-exports)
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

- **tucano**: The main entry point. Re-exports all core modules for easy consumption.
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

- To add support for a new market or protocol, create a new crate (e.g., `tucano-binance`, `tucano-ibrk`, `tucano-kraken`) following the same modular pattern. Implement the required traits from `core`, `execution`, and `data`.
- New strategies can be added to the `strategies/` crate or as separate crates for better isolation.
- All components communicate via strongly-typed events and traits, making it easy to plug in new modules without breaking existing code.


**Note:** All concrete exchange/broker integrations (such as B3/ProfitDLL) are now implemented as local modules in `tucano-data` or `tucano-integration`. The `markets` crate contains only abstractions (traits, enums, types).

This architecture allows you to build, test, and deploy algorithmic trading systems for any market or protocol, while keeping code maintainable and extensible.

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (version 1.75 or higher)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/brbtavares/tucano.git
cd tucano
```

### Add Dependency (façade)

In your `Cargo.toml`, add the unified crate (re-export) — recommended to start:

```toml
[dependencies]
tucano = { version = "0.1", features = ["full" ] }
```

Or, if you want more granularity/compile less, use individual crates:

```toml
[dependencies]
tucano-core = "0.1"
tucano-markets = "0.1"
tucano-data = "0.1"
tucano-execution = "0.1"
tucano-trader = "0.1"
tucano-risk = "0.1"
tucano-strategies = "0.1"
tucano-analytics = "0.1"
```

Then import via façade:

```rust
use tucano::prelude::*; // Engine, ExchangeId, Side, etc.
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
use tucano_trader::AlgoStrategy;
use tucano_strategies::{
    order_book_imbalance::OrderBookImbalanceStrategy,
    shared::NoOpState,
};
use tucano_execution::{ExchangeIndex, InstrumentIndex};
use tucano_execution::order::request::{OrderRequestCancel, OrderRequestOpen};

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
    .with_env_filter("tucano=debug")
    .init();

// Logs em código
debug!("Processando ordem: {:?}", order);
info!("Posição atualizada: {}", position);
warn!("Limite de risco próximo: {}", exposure);
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
# (Após renomear binário no futuro)
COPY --from=builder /app/target/release/tucano /usr/local/bin/
CMD ["tucano"]
```

### Environment Variables

```bash
# Produção
export RUST_ENV=production
export RUST_LOG=info
export B3_USERNAME=usuario_producao
export B3_PASSWORD=senha_producao
export DATABASE_URL=postgresql://user:pass@localhost/tucano
export REDIS_URL=redis://localhost:6379
```


## 🙏 Inspiration & Acknowledgments

This project was strongly **inspired by the architectural design of [barter-rs](https://github.com/barter-rs/barter-rs)**, whose initial structure served as a starting point for organizing modules, core traits, and the streaming/normalization approach. Our sincere thanks to its creator and all other developers and contributors of the **barter-rs** ecosystem – your work helped accelerate the initial phase of this framework.

---


## ⚠️ Disclaimer & Legal Scope

Educational/experimental use. **Not investment advice** nor financial, legal, accounting, or tax consulting. High risk: validate everything in a controlled environment (backtest/simulation) before any real operation. You are fully responsible for configurations, risk limits, regulatory compliance, and continuous monitoring.


### No Third-Party Compensation
There is no receipt of commission, rebate, sponsorship, or any economic advantage from third parties as a result of this project.


### Brands and Proprietary Integrations
All brands, trademarks, and proprietary APIs referenced are the property of their respective owners. Any integration shown here is purely technical and does not imply endorsement, support, or partnership. This repository does not distribute proprietary files or libraries – it only demonstrates how to interoperate when the user already has legitimate usage rights. Always read and respect the license terms of any third-party software or service you integrate.

For the full text, see `DISCLAIMER.md`.

---

**Tucano** – Modern algorithmic trading for global markets  
*MIT License* – see [LICENSE](LICENSE) | [DISCLAIMER](DISCLAIMER.md)

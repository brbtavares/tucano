
# Toucan Analytics

> Metrics, statistics, and performance summarization layer for strategies and backtests on B3.

## 🎯 Role in the Platform
The **analytics** crate provides calculations for return, risk, and operational efficiency consumed by `core` (auditing), `risk` (dynamic limits), and external interfaces (dashboards/reports). It transforms events (trades, positions, accumulated PnL) into consolidated indicators within the Toucan ecosystem.

| Responsibility         | Description                                                                 |
|------------------------|-----------------------------------------------------------------------------|
| Financial Metrics      | Sharpe, Sortino, Calmar, Profit Factor, Win Rate, Max Drawdown/Recovery     |
| Temporal Aggregation   | Series normalization by intervals (`time.rs`)                               |
| Trading Summaries      | Generation of consolidated summaries by instrument/exchange/window          |
| Algorithm Abstraction  | Reusable statistical calculations (`algorithm.rs`)                          |
| Cohesive Interface     | Stable API for other crates without exposing internal details               |

## 🔑 Main Modules / Types
- `algorithm.rs` – Statistical utility functions (means, deviations, return normalization).  
- `metric/` – Specific implementations:  
  - `sharpe.rs` (`SharpeRatio`)  
  - `sortino.rs` (`SortinoRatio`)  
  - `calmar.rs` (`CalmarRatio`)  
  - `profit_factor.rs` (`ProfitFactor`)  
  - `win_rate.rs` (`WinRate`)  
  - `drawdown/` (drawdown series calculation)  
- `time.rs` – Time intervals and granularities for aggregations.  
- `summary/` – Report assembly (by asset, session, window) and display (`display.rs`).

## 🔗 Interdependencies
| Depends on      | Reason                                                        |
|-----------------|---------------------------------------------------------------|
| `core` (events/audit) | Source of trade/order sequences (future)                |
| `execution`     | Source of fill/trade events                                   |
| `markets`       | Instrument and exchange identifiers                           |

| Consumed by     | Usage                                                         |
|-----------------|---------------------------------------------------------------|
| `core`          | Post-backtest consolidation and runtime monitoring            |
| `risk`          | Feeding adaptive limits (e.g., realized volatility)           |
| `strategy`      | Feedback loop for optimization/adaptation                     |

## ✅ Completed
- Classic metrics (Sharpe, Sortino, Calmar, Profit Factor, Win Rate) implemented.
- Initial modular summary structure.
- Basic support for time intervals.

## � Partial / In Progress
- Advanced drawdown (recovery and duration curves) – basic present.
- Multi-strategy / multi-fund composition (hierarchical aggregation missing).
- Incremental persistence of calculations (not implemented).

## � Pending / Roadmap
- Specific B3 microstructure metrics (slippage, execution effectiveness by auction / intraday).
- Latency KPIs (integration with `execution` timestamps).
- Exporters (CSV / Parquet / gRPC).
- Real-time rolling series via asynchronous channel.  

## 🏁 Basic Example (conceptual)
```rust
use analytics::metric::sharpe::SharpeRatio; // assinatura ilustrativa

let returns = vec![0.01, -0.005, 0.012, 0.003];
let sharpe = SharpeRatio::compute(&returns, 0.0);
println!("Sharpe: {:.2}", sharpe.value());
```

## 🇧🇷 B3 Context
The calculations will support typical instrument classes (stocks, index, mini-index, dollar, mini-dollar, gold, bitcoin futures) with return normalization per contract or adjustment factor (to be defined). Specific adjustments (e.g., index and dollar point multipliers) still need to be integrated.

## 📌 Notas
*Some names may change when the public API is stabilized. Until then, avoid rigid external dependencies.*

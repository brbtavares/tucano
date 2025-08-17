# Tucano Markets

> Models market entities (Exchange, Asset, Instrument, Side, OrderType) with specialization for B3.

## 🎯 Role
The **markets** crate provides the taxonomy and semantic types that support execution, data, and risk. It ensures that instruments and assets are consistently identified throughout the platform.

| Responsibility   | Description                                                                 |
|------------------|-----------------------------------------------------------------------------|
| Exchange Model   | `exchange.rs` / `b3.rs` define the `ExchangeId` enum and characteristics    |
| Asset Model      | `asset.rs`, `asset_simplified.rs` and B3 specializations (Stocks, ETFs, REITs, Futures) |
| Instrument       | Standardized construction (name, market, derived symbol)                    |
| Index            | `index/` for efficient keyed collections                                    |
| Broker Abstractions | `broker/` skeleton to unite multiple ProfitDLL brokers                   |

## 🔑 Main Types
- `ExchangeId` – Canonical identifier (e.g., `B3`).
- `Asset` / `B3Asset*` – Implementations by category (stock, ETF, REIT, future).
- `Instrument` – Combination of asset + market + semantics (e.g., mini-index).
- `Side`, `OrderType` – Order direction and modality.

## 🔗 Interdependencies
| Depends on                | Reason                                 |
|---------------------------|----------------------------------------|
| `rust_decimal`, `chrono`  | Monetary precision / timestamps        |

| Consumed by   | Usage                                         |
|---------------|-----------------------------------------------|
| `execution`   | Order identification / routing                |
| `data`        | Market event normalization                    |
| `risk`        | Limit calculations per asset/instrument       |
| `core`        | Global instrument state                       |
| `analytics`   | Aggregation keys by instrument                |

## ✅ Completed
- Enum of exchanges and basic B3 types.
- Basic instruments listed as examples (stocks, some administrative futures).

## 🧪 Parcial

- Multi-broker support (initial structure; robust credential/latency abstraction missing).
- Futures: rollover and adjustment factor calculation not yet implemented.
- B3 listed options: not supported at the moment (design pending).


## 🚧 Pending
- Normalization of derivative symbols (WIN, IND, DOL, WDO) with robust parsing.
- Configurable multipliers / tick size table.
- Dynamic instrument catalog (loading via API/Master file).
- Mapping of corporate actions (dividends / splits) for accurate backtesting.


## 🇧🇷 B3 Context
Will provide a foundation to gradually support the full range of traded instruments, with special attention to: mini-contracts, full contracts, sector ETFs, and listed crypto futures.

## Exemplo (conceitual)
```rust
use markets::exchange::ExchangeId;
let venue = ExchangeId::B3;
```

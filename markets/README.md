# Tucano Markets

> Abstractions for market entities (Exchange, Asset, Instrument, Side, OrderType). This crate now contains only traits, enums, and types; all concrete exchange/broker integrations are implemented in `data` or `integration`.

## 🎯 Role
The **markets** crate provides the taxonomy and semantic types that support execution, data, and risk. It ensures that instruments and assets are consistently identified throughout the platform.

| Responsibility   | Description                                                                 |
|------------------|-----------------------------------------------------------------------------|
| Exchange Model   | `exchange.rs` defines the `ExchangeId` enum and characteristics (abstractions only)    |
| Asset Model      | `asset.rs`, `asset_simplified.rs` and specializations (Stocks, ETFs, REITs, Futures) (abstractions only) |
| Instrument       | Standardized construction (name, market, derived symbol)                    |
| Index            | `index/` for efficient keyed collections                                    |

## 🔑 Main Types
-- `ExchangeId` – Canonical identifier.
-- `Asset` – Implementations by category (stock, ETF, REIT, future).
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
-- Enum of exchanges and basic types.
- Basic instruments listed as examples (stocks, some administrative futures).

## 🧪 Parcial

- Multi-broker abstraction (initial structure; all concrete implementations now live in `data` or `integration`).
- Futures: rollover and adjustment factor calculation not yet implemented.



## 🚧 Pending
- Normalization of derivative symbols (WIN, IND, DOL, WDO) with robust parsing.
- Configurable multipliers / tick size table.
- Dynamic instrument catalog (loading via API/Master file).
- Mapping of corporate actions (dividends / splits) for accurate backtesting.




## Exemplo (conceitual)
```rust
use markets::exchange::ExchangeId;
// let venue = ExchangeId::SomeExchange;
```

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
- Broker multi‑corretora (estrutura inicial; falta abstração robusta de credenciais/latência).
- Futuros: rollover e cálculo de fator de ajuste ainda não implementados.
- Opções listadas B3: não suportadas no momento (design pendente).

## 🚧 Pendências
- Normalização de símbolos de derivativos (WIN, IND, DOL, WDO) com parsing robusto.
- Tabela de multiplicadores / tick size configurável.
- Catálogo dinâmico de instrumentos (carregamento via API/Master file). 
- Mapeamento de corporate actions (dividendos / splits) para backtest fiel.

## 🇧🇷 Contexto B3
Proverá base para suportar gradualmente toda a gama de instrumentos negociados, com especial atenção a: mini-contratos, contratos cheios, ETFs setoriais e futuros de cripto listados.

## Exemplo (conceitual)
```rust
use markets::exchange::ExchangeId;
let venue = ExchangeId::B3;
```

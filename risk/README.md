# Tucano Risk

> Validation and risk control layer before orders reach the venue.

## 🎯 Role
The **risk** crate implements checks to protect capital, ensure compliance with limits, and prevent incorrect execution. It acts as a filter between signal generation (strategy) and submission (execution).

| Responsibility      | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| Atomic Checks       | `check/` contains validators (e.g., max position size)                      |
| Composition         | Structure to combine multiple checks sequentially                            |
| Strongly Typed Result | `RiskApproved<T>` / `RiskRefused<T>` types encapsulate the decision       |
| Semantic Errors     | Clear mapping of blocking reasons                                           |

## 🔑 Main Concepts
- `RiskManager` (future trait/aggregator struct) – Orchestrates evaluation.
- `check::*` – Individual modules for each policy (initial placeholder).
- `RiskApproved<T>` / `RiskRefused<T>` – Wrappers ensuring only approved flows proceed.

## 🔗 Interdependencies
| Depends on   | Reason                                                        |
|--------------|---------------------------------------------------------------|
| `markets`    | Instrument/asset identification                               |
| `execution`  | Access to current orders/positions                            |
| `analytics` (future) | Volatility/dynamic metrics for adaptive limits        |

| Consumed by  | Usage                                                         |
|--------------|---------------------------------------------------------------|
| `core`       | Pre-execution gate                                            |
| `strategy`   | Position adjustment based on risk response                    |

## ✅ Completed
- Initial structure for approved/refused types.
- Skeleton for basic checks.

## 🧪 Partial
- Real list of checks (notional exposure, global stop, daily loss) not implemented.
- Integration with runtime metrics.

## 🚧 Pending
- Política de agregação (primeiro falha vs coletar todos os motivos).
- Modo simulação vs produção (thresholds distintos).
- Auditoria de decisões (log estruturado / métricas de bloqueio).



## Exemplo (conceitual)
```rust
// pseudo-código
if risk_manager.validate(&order).is_ok() {
   submit(order)
}
```

# Tucano Execution

> Order execution and account synchronization layer (B3 via ProfitDLL initially).

## 🎯 Role
The **execution** crate encapsulates interaction with venues for order submission, fill reception, balance and position synchronization, offering a stable interface to `core` and abstracting specific details (latency, proprietary formats).

| Responsibility | Description                                                                 |
|----------------|-----------------------------------------------------------------------------|
| Client Trait   | `client/` abstracts submission, cancellation, state fetching                |
| Order          | `order/` models the lifecycle (request, snapshot, state)                    |
| Trade          | `trade/` represents executions and fees                                     |
| Balance        | `balance/` tracks multi-asset balances                                      |
| Indexing       | `indexer.rs` and map/ for efficient lookup                                  |
| Mock           | `exchange/mock` for deterministic tests and backtests                       |

## 🔑 Main Elements
- `ExecutionClient` (trait) – Contract for any execution integration.
- `MockExchange` / `MockExecutionConfig` – Controlled simulation environment.
- `AccountEvent` / `AccountSnapshot` – Unified account updates.
- `OrderRequest(Open/Cancel)` and `OrderSnapshot` – Complete order flow.
- `map::ExecutionTxMap` – Routing requests to different exchanges.

## 🔗 Interdependencies
| Depends on   | Reason                                                        |
|--------------|---------------------------------------------------------------|
| `markets`    | Exchange/instrument identifiers                               |
| `integration`| Async channels for requests/responses                         |
| `data`       | Coherence between market events and fills (timestamp)         |

| Consumed by  | Usage                                                         |
|--------------|---------------------------------------------------------------|
| `core`       | Order submission and account event ingestion                  |
| `risk`       | Validation before submit/cancel                               |
| `analytics`  | Sourcing trades for metrics                                   |

## ✅ Completed
- Functional mock execution structure.
- Structured account event pipeline (snapshot, open order, cancel, trade).
- Compat layer (String ↔ ExchangeId) stabilized post-refactor.

## 🧪 Parcial
- ProfitDLL real: autenticação e subscrição iniciadas; rota de ordens incompleta.
- Suporte a múltiplas corretoras ProfitDLL (faltando abstração de broker id/latência).
- Gestão de reconexão para execução (apenas esboço).

## 🚧 Pendências
- Implementar cancelamento efetivo / partial fills.
- Time-in-force, tipos avançados (stop, OCO) – roadmap.
- Medição de latência (enfileirar timestamps). 
- Persistência de sequência de ordens para recovery.

## 🇧🇷 Contexto B3
Foco em: ações, índice (IND/MINI WIN), dólar (DOL/WDO), futuros de bitcoin e ouro. Necessário mapear multiplicadores e taxas (emolumentos, corretagem, B3 fees) para PnL realista.

## 🏁 Exemplo Conceitual
```rust
use execution::order::request::OrderRequestOpen; // assinatura ilustrativa

let order = OrderRequestOpen::market_buy("PETR4", 100.0);
execution_tx.send(order)?;
```

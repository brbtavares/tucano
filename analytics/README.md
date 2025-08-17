
# Tucano Analytics

> Metrics, statistics, and performance summarization layer for strategies and backtests on B3.

## 🎯 Role in the Platform
The **analytics** crate provides calculations for return, risk, and operational efficiency consumed by `core` (auditing), `risk` (dynamic limits), and external interfaces (dashboards/reports). It transforms events (trades, positions, accumulated PnL) into consolidated indicators within the Tucano ecosystem.

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

## ✅ Concluído
- Métricas clássicas (Sharpe, Sortino, Calmar, Profit Factor, Win Rate) implementadas.
- Estrutura de summary modular inicial.
- Suporte básico a intervalos de tempo.

## 🧪 Parcial / Em Progresso
- Drawdown avançado (curvas de recuperação e duração) – básico presente.
- Composição multi‑estratégia / multi‑fundo (falta agregação hierárquica).
- Persistência incremental dos cálculos (não implementado).

## 🚧 Pendências / Roadmap
- Métricas específicas de microestrutura B3 (slippage, efetividade de execução por leilão / intraday).  
- KPIs de latência (integração com timestamps de `execution`).  
- Exportadores (CSV / Parquet / gRPC).  
- Séries rolling em tempo real via canal assíncrono.  

## 🏁 Exemplo Básico (conceitual)
```rust
use analytics::metric::sharpe::SharpeRatio; // assinatura ilustrativa

let returns = vec![0.01, -0.005, 0.012, 0.003];
let sharpe = SharpeRatio::compute(&returns, 0.0);
println!("Sharpe: {:.2}", sharpe.value());
```

## 🇧🇷 Contexto B3
Os cálculos irão suportar classes de instrumentos típicos (ações, índice, mini‑índice, dólar, mini‑dólar, ouro, bitcoin futuros) com normalização de retornos por contrato ou fator de ajuste (a definir). Ajustes específicos (ex: multiplicadores de pontos do índice e dólar) ainda precisam ser integrados.

## 📌 Notas
*Alguns nomes podem mudar quando a API pública for estabilizada. Até lá, evitar dependência rígida externa.*

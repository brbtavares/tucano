# Toucan Analytics

> Camada de métricas, estatísticas e sumarização de performance para estratégias e backtests na B3.

## 🎯 Papel na Plataforma
A crate **analytics** fornece cálculos de retorno, risco e eficiência operacional consumidos por `core` (auditoria), `risk` (limites dinâmicos) e interfaces externas (dashboards / relatórios). Ela transforma eventos (trades, posições, PnL acumulado) em indicadores consolidados.

| Responsabilidade | Descrição |
|------------------|-----------|
| Métricas Financeiras | Sharpe, Sortino, Calmar, Profit Factor, Win Rate, Drawdown Máximo / Recuperação |
| Agregação Temporal | Normalização de séries por intervalos (`time.rs`) |
| Sumários de Trading | Geração de resumos consolidados por instrumento / exchange / janela |
| Abstração de Algoritmos | Cálculos estatísticos reutilizáveis (`algorithm.rs`) |
| Interface Coesa | API estável para outras crates sem expor detalhes internos |

## 🔑 Principais Módulos / Tipos
- `algorithm.rs` – Funções utilitárias estatísticas (médias, desvios, normalização de retornos).*  
- `metric/` – Implementações específicas:  
  - `sharpe.rs` (`SharpeRatio`)  
  - `sortino.rs` (`SortinoRatio`)  
  - `calmar.rs` (`CalmarRatio`)  
  - `profit_factor.rs` (`ProfitFactor`)  
  - `win_rate.rs` (`WinRate`)  
  - `drawdown/` (cálculo de séries de drawdown)  
- `time.rs` – Intervalos e granularidades de tempo para agregações.  
- `summary/` – Montagem de relatórios (por ativo, sessão, janela) e exibição (`display.rs`).

## 🔗 Interdependências
| Depende de | Motivo |
|------------|--------|
| `core` (eventos/audit) | Fonte de sequências de trades / ordens (futuro) |
| `execution` | Origem de fill / trade events |
| `markets` | Identificadores de instrumentos e exchanges |

| Consumido por | Uso |
|---------------|-----|
| `core` | Consolidação pós-backtest e monitoramento runtime |
| `risk` | Alimentar limites adaptativos (ex: volatilidade realizada) |
| `strategy` | Feedback loop para otimização / adaptação |

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

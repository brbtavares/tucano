# Toucan Strategy

> Definição de interfaces e componentes auxiliares para estratégias algorítmicas plugáveis.

## 🎯 Papel
A crate **strategy** fornece traits e utilidades para implementação de estratégias desacopladas do motor (`core`). Define pontos de extensão para reação a eventos, desconexões, desativação de trading e ciclo de vida.

| Responsabilidade | Descrição |
|------------------|-----------|
| Trait Principal | `AlgoStrategy` (nome sujeito a revisão) – callbacks de processamento |
| Hooks de Estado | `OnDisconnectStrategy`, `OnTradingDisabledStrategy` para eventos de infraestrutura |
| Fechamento | `close_positions.rs` abstrai rotina de desmontagem de exposição |
| Defaults | `default.rs` implementa comportamento neutro |

## 🔑 Conceitos
- `AlgoStrategy` – Recebe eventos (market/account) e emite intents de ordem (via core).
- Estratégias Compostas (futuro) – Multiplexar várias lógicas em um portfolio.
- Gestão de Capital (futuro) – Alocação percentual por instrumento / cluster.

## 🔗 Interdependências
| Depende de | Motivo |
|------------|-------|
| `markets` | Identificação de instrumentos |
| `execution` | Tipos de ordens utilizadas nos intents |
| `data` | Eventos de mercado processados |
| `risk` | Feedback de aprovação / recusa para ajustes |

| Consumido por | Uso |
|---------------|-----|
| `core` | Invoca callbacks de estratégia |
| `analytics` | Consumo de sinais para métricas de eficácia |

## ✅ Concluído
- Estrutura de traits e estratégias de resposta a eventos básicos.
- Hooks para desligar trading / lidar com desconexão.

## 🧪 Parcial
- Sistema de intents de ordem ainda não formalizado (interfaces adaptadas durante rollback).
- Falta camada de portfolio / multi-strategy.

## 🚧 Pendências
- DSL de construção de estratégias (pipelines de sinais, filtros, indicadores B3).
- Estado interno persistente (cache de features) entre execuções.
- Modo de otimização (grid search / walk-forward) integrado a backtest.

## 🇧🇷 Contexto B3
Estratégias alvo: arbitragem estatística entre mini e cheio, rotinas de abertura/fechamento (leilões), mean reversion em ações líquidas, momentum intraday em contratos de índice e dólar.

## Exemplo (conceitual)
```rust
struct MinhaEstrategia; // implementa AlgoStrategy
```

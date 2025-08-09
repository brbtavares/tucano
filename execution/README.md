# Toucan Execution

> Camada de execução de ordens e sincronização de conta (B3 via ProfitDLL inicialmente).

## 🎯 Papel
A crate **execution** encapsula interação com venues para envio de ordens, recebimento de fills, sincronização de saldos e posições, oferecendo uma interface estável ao `core` e abstraindo detalhes específicos (latência, formatos proprietários).

| Responsabilidade | Descrição |
|------------------|-----------|
| Client Trait | `client/` abstrai submissão, cancelamento, fetch de estado |
| Ordem | `order/` modela ciclo de vida (request, snapshot, estado) |
| Trade | `trade/` representa execuções e fees |
| Balance | `balance/` rastreia saldos multi-ativo |
| Indexação | `indexer.rs` e map/ para lookup eficiente |
| Mock | `exchange/mock` para testes e backtests determinísticos |

## 🔑 Principais Elementos
- `ExecutionClient` (trait) – Contrato para qualquer integração de execução.
- `MockExchange` / `MockExecutionConfig` – Ambiente de simulação controlado.
- `AccountEvent` / `AccountSnapshot` – Unificação de atualizações de conta.
- `OrderRequest(Open/Cancel)` e `OrderSnapshot` – Fluxo completo da ordem.
- `map::ExecutionTxMap` – Roteamento de requisições para diferentes exchanges.

## 🔗 Interdependências
| Depende de | Motivo |
|------------|-------|
| `markets` | Identificadores de exchange / instrumento |
| `integration` | Canais assíncronos para requests/respostas |
| `data` | Coerência entre eventos de mercado e fills (timestamp) |

| Consumido por | Uso |
|---------------|-----|
| `core` | Envio de ordens e ingestão de eventos de conta |
| `risk` | Validação antes de submit / cancel |
| `analytics` | Sourcing de trades para métricas |

## ✅ Concluído
- Estrutura do mock de execução funcional.
- Pipeline de eventos de conta (snapshot, ordem aberta, cancel, trade) estruturado.
- Compat layer (String ↔ ExchangeId) estabilizada pós refactor.

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

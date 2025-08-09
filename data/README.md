# Toucan Data

> Camada de ingestão, normalização e distribuição de eventos de mercado para a B3 (e futuras fontes).

## 🎯 Papel
A crate **data** centraliza a modelagem de eventos de mercado (ticks, trades, books, snapshots), abstrai fontes heterogêneas (ProfitDLL inicialmente) e entrega um fluxo unificado ao `core`.

| Responsabilidade | Descrição |
|------------------|-----------|
| Modelos de Evento | `event.rs` define enums estruturados (Trade, Quote, Book, etc.) |
| Identificadores | `instrument.rs`, `exchange.rs`, `subscriber`/`subscription` gerenciam chaves |
| Streams | Módulo `streams/` provê construção, reconexão, transformação |
| Transformação | `transformer/` para parsing/adaptação de payloads brutos |
| Normalização B3 | Integração com tipos de ativos/mercados da crate `markets` |
| Snapshotting | `snapshot.rs` + `collection/` para estado consistente inicial |

## 🔑 Principais Módulos
- `event.rs` – Tipos de evento de mercado/conta.
- `instrument.rs` – Estruturas de identificação de instrumentos normalizados.
- `streams/` – Conectores e lógica de (re)conexão resiliente.
- `exchange/` – Organização por venue (B3 primeiro; espaço para outros).
- `subscriber/` & `subscription/` – Gestão de inscrições e lifecycle.
- `transformer/` – Pipelines de parsing e enriquecimento.
- `snapshot.rs` – Processamento de snapshots iniciais.

## 🔗 Interdependências
| Depende de | Motivo |
|------------|-------|
| `markets` | Tipagem de ativos / instrumentos B3 |
| `integration` | Canais / transporte (websocket / http wrappers futuramente) |
| `execution` | Para unir eventos de mercado e conta (consistência) |

| Consumido por | Uso |
|---------------|-----|
| `core` | Feed principal do motor de eventos |
| `analytics` | Série de preços / trades para métricas |
| `strategy` | Gatilhos de sinal |
| `risk` | Volatilidade, gaps, validações de integridade |

## ✅ Concluído
- Modelos básicos de evento e assinatura.
- Estrutura de reconexão inicial (`streams::reconnect`).
- Integração parcial com ProfitDLL (estado embrionário).

## 🧪 Parcial
- Transformer genérico (alguns parsers placeholders).
- Snapshots de book / profundidade – a detalhar.
- Book incremental (diffs) não implementado.

## 🚧 Pendências
- Suporte a diferentes frequências (agg de 1s/1m) nativamente.
- Compressão e serialização eficiente (Parquet / Arrow) para histórico.
- Backfill de gaps de conexão.
- Canal de latência (timestamp triplo: source, receive, process).

## 🇧🇷 Contexto B3
Foco inicial: ações e derivativos listados; necessidade de mapear códigos padronizados (WIN, IND, DOL, WDO, ouro, BTC). Fábricas de símbolos e *rollover* de contratos futuros serão adicionados.

## 🏁 Exemplo (conceitual)
```rust
use data::event::MarketEvent; // assinatura ilustrativa

fn handle(event: MarketEvent) {
    match event { /* ... */ }
}
```

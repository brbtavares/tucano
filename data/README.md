# Tucano Data

> Layer for ingestion, normalization, and distribution of market events for B3 (and future sources).

## 🎯 Role
The **data** crate centralizes the modeling of market events (ticks, trades, books, snapshots), abstracts heterogeneous sources (initially ProfitDLL), and delivers a unified stream to the `core`.

| Responsibility      | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| Event Models        | `event.rs` defines structured enums (Trade, Quote, Book, etc.)              |
| Identifiers         | `instrument.rs`, `exchange.rs`, `subscriber`/`subscription` manage keys     |
| Streams             | `streams/` module provides construction, reconnection, transformation       |
| Transformation      | `transformer/` for parsing/adapting raw payloads                            |
| B3 Normalization    | Integration with asset/market types from the `markets` crate                |
| Snapshotting        | `snapshot.rs` + `collection/` for consistent initial state                  |

## 🔑 Main Modules
- `event.rs` – Market/account event types.
- `instrument.rs` – Structures for normalized instrument identification.
- `streams/` – Connectors and resilient (re)connection logic.
- `exchange/` – Organization by venue (B3 first; room for others).
- `subscriber/` & `subscription/` – Subscription and lifecycle management.
- `transformer/` – Parsing and enrichment pipelines.
- `snapshot.rs` – Initial snapshot processing.

## 🔗 Interdependencies
| Depends on     | Reason                                                        |
|----------------|---------------------------------------------------------------|
| `markets`      | Typing for B3 assets/instruments                              |
| `integration`  | Channels/transport (websocket/http wrappers in the future)    |
| `execution`    | To join market and account events (consistency)               |

| Consumed by    | Usage                                                         |
|----------------|---------------------------------------------------------------|
| `core`         | Main feed for the event engine                                |
| `analytics`    | Price/trade series for metrics                                |
| `strategy`     | Signal triggers                                               |
| `risk`         | Volatility, gaps, integrity checks                            |

## ✅ Completed
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

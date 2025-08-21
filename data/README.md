# Toucan Data

> Layer for ingestion, normalization, and distribution of market events for global sources. All concrete exchange/broker integrations are now implemented as local modules here. The `markets` crate contains only abstractions (traits, enums, types).

## 🎯 Role
The **data** crate centralizes the modeling of market events (ticks, trades, books, snapshots), abstracts heterogeneous sources (initially ProfitDLL), and delivers a unified stream to the `core`.

| Responsibility      | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| Event Models        | `event.rs` defines structured enums (Trade, Quote, Book, etc.)              |
| Identifiers         | `instrument.rs`, `exchange.rs`, `subscriber`/`subscription` manage keys     |
| Streams             | `streams/` module provides construction, reconnection, transformation       |
| Transformation      | `transformer/` for parsing/adapting raw payloads                            |
| Integration Example | Implemented as a local module in `exchange/`                        |
| Normalization    | Integration with asset/market types from the `markets` crate                |
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
| `markets`      | Abstractions (traits, enums, types) for assets/instruments |
| `integration`  | Channels/transport (websocket/http wrappers in the future)    |
| `execution`    | To join market and account events (consistency)               |

| Consumed by    | Usage                                                         |
|----------------|---------------------------------------------------------------|
| `core`         | Main feed for the event engine                                |
| `analytics`    | Price/trade series for metrics                                |
| `strategy`     | Signal triggers                                               |
| `risk`         | Volatility, gaps, integrity checks                            |

## ✅ Completed
- Basic event and subscription models.
- Initial reconnection structure (`streams::reconnect`).



## 🧪 Partial
- Generic transformer (some parser placeholders).
- Book/depth snapshots – to be detailed.
- Incremental book (diffs) not implemented.

## 🚧 Pending
- Native support for different frequencies (1s/1m aggregation).
- Efficient compression and serialization (Parquet / Arrow) for history.
- Backfill of connection gaps.
- Latency channel (triple timestamp: source, receive, process).




## 🏁 Exemplo (conceitual)
```rust
use data::event::MarketEvent; // assinatura ilustrativa

fn handle(event: MarketEvent) {
    match event { /* ... */ }
}
```

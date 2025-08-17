# Tucano Integration

> Communication abstractions (channels, protocols) and transformation to connect external sources (WebSocket, HTTP, future FIX) to the ecosystem.

## 🎯 Role
The **integration** crate provides building blocks for data ingestion and request sending: typed channels, snapshots, transformers, validators, and generic metrics – all reused by `data` and `execution`.

| Responsibility | Description                                                                 |
|----------------|-----------------------------------------------------------------------------|
| Channels       | `channel/` defines unified Tx/Rx (includes controlled drops)                |
| Transformation | `protocol/` + `de.rs` for deserializing and converting payloads             |
| Metrics        | `metric.rs` abstracts runtime metric collection                             |
| Subscription   | `subscription/` manages subscription lifecycle                              |
| Snapshot       | `snapshot.rs` structures initial consistency                                |
| Collection     | Auxiliary types (`OneOrMany`, `NoneOneOrMany`) for ergonomics              |

## 🔑 Types / Traits
- `Tx`, `ChannelTxDroppable` – Decoupled sending with backpressure.
- `Snapshot<T>` – Value + metadata (timestamp/sequence).
- `Validator`, `Transformer`, `Terminal`, `Unrecoverable` – Robustness contracts.

## 🔗 Interdependencies
| Depends on | Reason                                  |
|------------|-----------------------------------------|
| (minimal)  | Kept lightweight to be a reusable base  |

| Consumed by | Usage                                  |
|-------------|----------------------------------------|
| `data`      | Builds stream pipelines                |
| `execution` | Propagates account/internal order events|
| `analytics` | Can receive runtime metrics            |
| `core`      | Uses channel abstractions generically  |

## ✅ Completed
- Operational generic channel abstractions.
- Basic Snapshot and Subscription structures.
- Transformation and validation traits defined.

## 🧪 Partial
- Metrics: collection still superficial.
- Protocolos: HTTP/WebSocket placeholders; FIX não iniciado.
- Tipos collection documentados mas com links rustdoc quebrados (ajustar).

## 🚧 Pendências
- Service layer para reconexão automática com política configurável.
- Buffering adaptativo com pressão de memória.
- Telemetria estruturada (latência por estágio de pipeline).

## 🇧🇷 Contexto B3
Servirá de ponte para integrar ProfitDLL hoje e futuramente APIs oficiais B3 (REST / WebSocket) ou provedores de dados alternativos.

## Exemplo (conceitual)
```rust
use integration::channel::mpsc_unbounded; // assinatura ilustrativa
let (tx, rx) = mpsc_unbounded();
tx.send("payload")?;
```

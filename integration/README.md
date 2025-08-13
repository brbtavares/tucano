# Tucano Integration

> Abstrações de comunicação (canais, protocolos) e transformação para conectar fontes externas (WebSocket, HTTP, FIX futuro) ao ecossistema.

## 🎯 Papel
A crate **integration** provê blocos de construção para ingestão de dados e envio de requisições: canais tipados, snapshots, transformadores, validadores e métrica genérica – tudo reutilizado por `data` e `execution`.

| Responsabilidade | Descrição |
|------------------|-----------|
| Canais | `channel/` define Tx/Rx unificados (inclui quedas controladas) |
| Transformação | `protocol/` + `de.rs` para desserializar e converter payloads |
| Métricas | `metric.rs` abstrai coleta de métricas runtime |
| Subscription | `subscription/` gerencia ciclo de vida de inscrições |
| Snapshot | `snapshot.rs` estrutura consistência inicial |
| Collection | Tipos auxiliares (`OneOrMany`, `NoneOneOrMany`) para ergonomia |

## 🔑 Tipos / Traits
- `Tx`, `ChannelTxDroppable` – Envio desacoplado de backpressure.
- `Snapshot<T>` – Valor + metadados (timestamp / sequência).
- `Validator`, `Transformer`, `Terminal`, `Unrecoverable` – Contratos de robustez.

## 🔗 Interdependências
| Depende de | Motivo |
|------------|-------|
| (mínimas) | Mantida leve para ser base reutilizável |

| Consumido por | Uso |
|---------------|-----|
| `data` | Constrói pipelines de stream |
| `execution` | Propaga eventos de conta / ordens internas |
| `analytics` | Pode receber métricas runtime |
| `core` | Usa abstrações de canal genericamente |

## ✅ Concluído
- Abstrações de canal genéricas operacionais.
- Estruturas Snapshot e Subscription básicas.
- Traits de transformação e validação definidos.

## 🧪 Parcial
- Métricas: coleta ainda superficial.
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

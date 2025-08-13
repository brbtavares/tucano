# Tucano Markets

> Modela entidades de mercado (Exchange, Asset, Instrument, Side, OrderType) com especialização para a B3.

## 🎯 Papel
A crate **markets** fornece a taxonomia e tipos semânticos que suportam execução, dados e risco. Garante que instrumentos e ativos sejam identificados de forma consistente ao longo da plataforma.

| Responsabilidade | Descrição |
|------------------|-----------|
| Exchange Model | `exchange.rs` / `b3.rs` definem enum `ExchangeId` e características |
| Asset Model | `asset.rs`, `asset_simplified.rs` e especializações B3 (Stocks, ETFs, REITs, Futuros) |
| Instrument | Construção padronizada (nome, mercado, símbolo derivado) |
| Profit DLL Bridge | `profit_dll.rs` / `profit_dll_complete.rs` suporte de interop |
| Index | `index/` para coleções chaveadas eficientes |
| Broker Abstractions | `broker/` esqueleto para unir múltiplas corretoras ProfitDLL |

## 🔑 Principais Tipos
- `ExchangeId` – Identificador canônico (ex: `B3`).
- `Asset` / `B3Asset*` – Implementações por categoria (stock, ETF, REIT, futuro). 
- `Instrument` – Combinação de asset + mercado + semântica (ex: mini‑índice).
- `Side`, `OrderType` – Direção e modalidade de ordens.
- `ProfitConnector` / `ProfitDLLBroker` (via ponte com execution futura).

## 🔗 Interdependências
| Depende de | Motivo |
|------------|-------|
| `rust_decimal`, `chrono` | Precisão monetária / timestamps |

| Consumido por | Uso |
|---------------|-----|
| `execution` | Identificação de ordens / roteamento |
| `data` | Normalização de eventos de mercado |
| `risk` | Cálculos de limites por ativo / instrumento |
| `core` | Estado global de instrumentos |
| `analytics` | Chaves de agregação por instrumento |

## ✅ Concluído
- Enum de exchanges e tipos básicos B3.
- Estruturas iniciais de Profit DLL wrapper.
- Instrumentos básicos listados como exemplos (stocks, alguns futuros administrativos).

## 🧪 Parcial
- Broker multi‑corretora (estrutura inicial; falta abstração robusta de credenciais/latência).
- Futuros: rollover e cálculo de fator de ajuste ainda não implementados.
- Opções listadas B3: não suportadas no momento (design pendente).

## 🚧 Pendências
- Normalização de símbolos de derivativos (WIN, IND, DOL, WDO) com parsing robusto.
- Tabela de multiplicadores / tick size configurável.
- Catálogo dinâmico de instrumentos (carregamento via API/Master file). 
- Mapeamento de corporate actions (dividendos / splits) para backtest fiel.

## 🇧🇷 Contexto B3
Proverá base para suportar gradualmente toda a gama de instrumentos negociados, com especial atenção a: mini-contratos, contratos cheios, ETFs setoriais e futuros de cripto listados.

## Exemplo (conceitual)
```rust
use markets::exchange::ExchangeId;
let venue = ExchangeId::B3;
```

# 🏗️ Proposta: Arquitetura Híbrida - Abstrações + B3 Específico

## 🎯 Objetivo

Manter abstrações úteis da `markets` mas simplificar e focar na B3, criando uma arquitetura que é:
- **Abstrata** o suficiente para reutilização
- **Específica** o suficiente para B3
- **Simples** sem complexidade desnecessária

## 🔄 Estratégia Híbrida

### 1. **Markets Simplificada** - Abstrações Essenciais

```rust
// markets/src/lib.rs - Manter apenas o essencial
pub mod exchange;    // Traits e conceitos de exchange
pub mod instrument;  // Traits de instrumentos
pub mod asset;       // Traits de ativos
pub mod side;        // Buy/Sell enum
pub mod index;       // Sistema de indexação (se necessário)

// ❌ Remover implementações específicas de outras exchanges
// ❌ Remover Binance, Coinbase, etc.
// ✅ Manter apenas traits e enums básicos
```

### 2. **B3 Como Implementação Concreta**

```rust
// data/src/exchange/b3/ - Implementações B3-específicas
use markets::{Exchange, Instrument, Asset, Side}; // Usar traits

pub struct B3Exchange;
impl markets::Exchange for B3Exchange { ... }

pub struct B3Instrument(String, String); // symbol, market
impl markets::Instrument for B3Instrument { ... }

pub enum B3Asset { BRL, Stock(String), Fund(String) }
impl markets::Asset for B3Asset { ... }
```

### 3. **Execution Layer Unificado**

```rust
// execution/src/ - Usar traits de markets + implementações B3
use markets::{Exchange, Instrument, Asset, Side};
use data::exchange::b3::{B3Exchange, B3Instrument, B3Asset};

pub trait ExecutionClient<E: Exchange, I: Instrument, A: Asset> {
    async fn submit_order(&self, instrument: I, side: Side, ...) -> Result<...>;
    // Genérico mas tipado
}
```

## 🏛️ Estrutura Proposta

```
markets/
├── src/
│   ├── lib.rs          # Traits básicos reutilizáveis
│   ├── exchange.rs     # trait Exchange
│   ├── instrument.rs   # trait Instrument  
│   ├── asset.rs        # trait Asset
│   ├── side.rs         # enum Side
│   └── index.rs        # Sistema de indexação (opcional)

data/src/exchange/b3/
├── types.rs           # B3-específicos que implementam traits
├── exchange.rs        # B3Exchange: markets::Exchange
├── instrument.rs      # B3Instrument: markets::Instrument
├── asset.rs          # B3Asset: markets::Asset
└── connector/
    ├── profit_dll.rs  # B3ProfitConnector
    ├── rest_api.rs    # B3RestConnector (futuro)
    └── websocket.rs   # B3WebSocketConnector (futuro)

execution/src/
├── client/
│   ├── mod.rs         # ExecutionClient<E,I,A> trait
│   └── b3/
│       ├── mod.rs     # B3ExecutionClient
│       └── adapter.rs # Conversões
└── order/
    └── mod.rs         # Order<E,I,A> genérico
```

## 🔄 Exemplo de Implementação

### 1. **Markets Traits (Simplificados)**

```rust
// markets/src/exchange.rs
pub trait Exchange {
    type ExchangeId: Copy + Eq + Hash;
    fn id(&self) -> Self::ExchangeId;
    fn name(&self) -> &'static str;
}

// markets/src/instrument.rs  
pub trait Instrument {
    type Symbol: Display + Clone;
    fn symbol(&self) -> &Self::Symbol;
    fn market(&self) -> &str;
}

// markets/src/asset.rs
pub trait Asset {
    fn symbol(&self) -> &str;
    fn asset_type(&self) -> AssetType;
}

// markets/src/side.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Buy, Sell }
```

### 2. **B3 Implementações Concretas**

```rust
// data/src/exchange/b3/exchange.rs
pub struct B3Exchange;

impl markets::Exchange for B3Exchange {
    type ExchangeId = B3ExchangeId;
    fn id(&self) -> Self::ExchangeId { B3ExchangeId }
    fn name(&self) -> &'static str { "B3" }
}

// data/src/exchange/b3/instrument.rs
pub struct B3Instrument {
    symbol: SmolStr,
    market: B3Market,
}

impl markets::Instrument for B3Instrument {
    type Symbol = SmolStr;
    fn symbol(&self) -> &Self::Symbol { &self.symbol }
    fn market(&self) -> &str { self.market.as_str() }
}

impl B3Instrument {
    pub fn bovespa(symbol: impl Into<SmolStr>) -> Self { ... }
    pub fn bmf(symbol: impl Into<SmolStr>) -> Self { ... }
}
```

### 3. **Execution Client Genérico**

```rust
// execution/src/client/mod.rs
pub trait ExecutionClient<E, I, A>
where
    E: markets::Exchange,
    I: markets::Instrument,
    A: markets::Asset,
{
    async fn submit_order(
        &self,
        instrument: I,
        side: markets::Side,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<Order<E, I, A>, ExecutionError>;
    
    async fn cancel_order(&self, order_id: OrderId) -> Result<(), ExecutionError>;
    async fn get_balances(&self) -> Result<Vec<Balance<A>>, ExecutionError>;
}

// execution/src/client/b3/mod.rs
pub struct B3ExecutionClient { ... }

impl ExecutionClient<B3Exchange, B3Instrument, B3Asset> for B3ExecutionClient {
    // Implementação específica para B3
}
```

## 🎯 Vantagens da Arquitetura Híbrida

### ✅ **Abstração Mantida**
- Traits reutilizáveis para conceitos de mercado
- Sistema de tipos genérico para orders, trades, etc.
- Possibilidade de adicionar outras exchanges futuramente

### ✅ **B3-Específico Onde Importa**
- Implementações concretas com terminologia brasileira
- Múltiplos conectores B3 (ProfitDLL, REST, WebSocket)
- Tipos nativos (BOVESPA, BMF, BRL)

### ✅ **Simplicidade**
- Markets crate muito menor (só traits essenciais)
- Remoção de exchanges desnecessárias
- Foco na funcionalidade real necessária

### ✅ **Flexibilidade Futura**
- Fácil adição de novas exchanges
- Reutilização de execution layer
- Manutenção de compatibilidade com core engine

## 🚀 Implementação

### Fase 1: Simplificar Markets
1. Remover todas as exchanges específicas (Binance, etc.)
2. Manter apenas traits essenciais
3. Simplificar sistema de indexação

### Fase 2: B3 Como Implementação
1. Criar B3Exchange, B3Instrument, B3Asset implementando traits
2. Migrar B3ExecutionClient para usar traits genéricos
3. Manter conectores múltiplos (ProfitDLL, REST, etc.)

### Fase 3: Unificar Execution Layer
1. ExecutionClient genérico com type parameters
2. Orders e Trades genéricos mas tipados
3. Engine usando abstrações

## 🤔 Alternativa: Sem Markets

Se quisermos **remover completamente** a `markets`:

```rust
// Criar nossa própria abstração mínima em execution/
pub trait Exchange { ... }
pub trait Instrument { ... }
pub trait Asset { ... }

// B3 implementa diretamente
impl Exchange for B3Exchange { ... }
impl Instrument for B3Instrument { ... }
impl Asset for B3Asset { ... }
```

## 💭 Qual Abordagem Preferir?

**Pergunta para você**: Qual das duas abordagens prefere?

1. **Híbrida**: Markets simplificada + B3 específico
2. **Própria**: Remover markets, criar traits próprios mínimos

A **híbrida** mantém compatibilidade e reutilização.
A **própria** é mais simples e focada.

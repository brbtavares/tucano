# 🏛️ Arquitetura Modular B3 - Framework Toucan

## 🎯 Visão Geral

A refatoração implementada transforma o Toucan em uma arquitetura **híbrida** que combina:
- **Abstrações reutilizáveis** da crate `markets` (simplificada)
- **Implementações B3-específicas** com terminologia e tipos brasileiros
- **Conectividade modular** para múltiplos provedores

## 🔄 Estratégia Híbrida: Abstrações + B3 Específico

### Filosofia
```
📐 Abstrações (markets)     +     🇧🇷 Implementações B3     =     🚀 Framework Flexível
     ↓                              ↓                              ↓
• Traits reutilizáveis        • Tipos brasileiros           • Fácil extensão  
• Sistema de tipos           • Terminologia nativa         • Múltiplos conectores
• Compatibilidade            • Regras específicas B3       • Testabilidade
```

## 📁 Arquitetura Híbrida

### 1. **Markets (Abstrações Essenciais)** (`markets/src/`)

```rust
// ✅ Traits e tipos fundamentais (mantidos e simplificados)
pub trait Exchange {
    type ExchangeId: Copy + Eq + Hash;
    fn id(&self) -> Self::ExchangeId;
    fn name(&self) -> &'static str;
}

pub trait Instrument {
    type Symbol: Display + Clone;
    fn symbol(&self) -> &Self::Symbol;
    fn market(&self) -> &str;
}

pub trait Asset {
    fn symbol(&self) -> &str;
    fn asset_type(&self) -> AssetType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Buy, Sell }

// ❌ Removidas: implementações específicas de Binance, Coinbase, etc.
// ✅ Mantidas: apenas abstrações reutilizáveis
```

### 2. **B3 Implementações Concretas** (`data/src/exchange/b3/`)

```rust
// 🇧🇷 Tipos B3-específicos que implementam as abstrações
pub struct B3Exchange;
impl markets::Exchange for B3Exchange {
    type ExchangeId = B3ExchangeId;
    fn id(&self) -> Self::ExchangeId { B3ExchangeId }
    fn name(&self) -> &'static str { "B3" }
}

pub struct B3Instrument {
    symbol: SmolStr,    // PETR4, VALE3, etc.
    market: B3Market,   // BOVESPA, BMF
}
impl markets::Instrument for B3Instrument { ... }

pub enum B3Asset { BRL, Stock(SmolStr), Fund(SmolStr), Future(SmolStr) }
impl markets::Asset for B3Asset { ... }

// ✅ Terminologia brasileira + compatibilidade com abstrações
```

### 3. **Execution Layer Unificado** (`execution/src/`)

```rust
// � ExecutionClient genérico que funciona com qualquer Exchange/Instrument/Asset
pub trait ExecutionClient<E, I, A>
where
    E: markets::Exchange,
    I: markets::Instrument,  
    A: markets::Asset,
{
    async fn submit_order(&self, instrument: I, side: markets::Side, ...) -> Result<Order<E,I,A>>;
    async fn cancel_order(&self, order_id: OrderId) -> Result<()>;
    async fn get_balances(&self) -> Result<Vec<Balance<A>>>;
}

// 🏛️ B3 como implementação específica
impl ExecutionClient<B3Exchange, B3Instrument, B3Asset> for B3ExecutionClient {
    // Implementação usando ProfitDLL, REST API, WebSocket, etc.
}
```

### 3. **Eventos B3** 

```rust
pub enum B3MarketEvent {
    NewTrade { trade: B3Trade },
    DailySummary { instrument: B3Instrument, open, high, low, close, volume },
    OrderBookUpdate { instrument: B3Instrument, side: B3BookSide, level: B3BookLevel },
    AccountChanged { account: B3Account },
    InvalidInstrument { instrument: B3Instrument },
    StateChanged { connection_type: String, result: i32 },
}
```

## 🚀 Benefícios da Arquitetura Híbrida

### ✅ **Melhor dos Dois Mundos**
- **Abstrações úteis**: Traits reutilizáveis para Exchange, Instrument, Asset
- **B3-específico**: Implementações com terminologia e regras brasileiras  
- **Tipagem forte**: Sistema de tipos genérico mas seguro
- **Compatibilidade**: Engine e execution layer funcionam com qualquer exchange

### ✅ **Markets Simplificada**
- **Focada em abstrações**: Apenas traits essenciais, sem implementações específicas
- **Sem Binance/outras**: Remoção de exchanges desnecessárias
- **Indexação opcional**: Sistema de indexação mantido se necessário
- **Menor footprint**: Crate muito mais leve e focada

### ✅ **B3 Como Implementação de Referência**
- **Terminologia nativa**: BOVESPA, BMF, BRL, etc.
- **Múltiplos conectores**: ProfitDLL, REST API, WebSocket
- **Traits implementados**: Compatível com todo o framework
- **Extensibilidade**: Fácil adição de novas funcionalidades B3

### 🔧 **Múltiplos Provedores de Conectividade**

1. **ProfitDLL (Nelógica)** - ✅ Implementado
   - Conector via DLL da Nelógica
   - Ideal para dados em tempo real
   - Requer credenciais da Nelógica

2. **B3 API Oficial** - 🔜 Futuro
   - REST API oficial da B3
   - WebSocket feeds
   - Dados históricos

3. **Mock Connector** - 🔜 Futuro
   - Para testes automatizados
   - Simulação de mercado
   - Desenvolvimento offline

4. **Outros Provedores** - 🔜 Futuro
   - Trademap, SysInvest, etc.
   - APIs de terceiros
   - Feeds alternativos

### 📊 **Exemplo de Uso da Arquitetura Híbrida**

```rust
use markets::{Exchange, Instrument, Asset, Side};
use toucan_data::exchange::b3::{B3Exchange, B3Instrument, B3Asset};
use toucan_execution::client::ExecutionClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 🏛️ Tipos B3-específicos que implementam traits abstratos
    let exchange = B3Exchange;
    let petr4 = B3Instrument::bovespa("PETR4");
    let vale3 = B3Instrument::bovespa("VALE3");
    let real = B3Asset::BRL;
    
    println!("🏛️ Exchange: {} ({})", exchange.name(), exchange.id());
    println!("📊 Instruments: {}, {}", petr4.symbol(), vale3.symbol());
    println!("💰 Asset: {}", real.symbol());
    
    // 🔌 ExecutionClient genérico tipado para B3
    let client: Box<dyn ExecutionClient<B3Exchange, B3Instrument, B3Asset>> = 
        Box::new(B3ExecutionClient::new(config));
    
    // 📋 Submit order usando abstrações + tipos específicos
    let order = client.submit_order(
        petr4,
        Side::Buy,
        Decimal::new(100, 0),
        Some(Decimal::new(2850, 2)), // R$ 28.50
    ).await?;
    
    println!("✅ Order submitted: {:?}", order);
    
    // 💰 Get balances - retorna B3Asset específicos
    let balances = client.get_balances().await?;
    for balance in balances {
        println!("💰 Balance: {} = {}", balance.asset.symbol(), balance.total());
    }
    
    Ok(())
}
```

### 🔄 **Conectividade Múltipla**

```rust
// 🔌 Diferentes conectores, mesma interface
let profit_client = B3ProfitConnector::new(profit_config);
let rest_client = B3RestConnector::new(rest_config);      // Futuro
let mock_client = MockB3Connector::new(mock_data);        // Testes

// 🎯 Todos implementam ExecutionClient<B3Exchange, B3Instrument, B3Asset>
let clients: Vec<Box<dyn ExecutionClient<B3Exchange, B3Instrument, B3Asset>>> = vec![
    Box::new(profit_client),
    Box::new(rest_client),
    Box::new(mock_client),
];

// 🚀 Engine pode usar qualquer um sem mudanças
for client in clients {
    let balances = client.get_balances().await?;
    // Processamento uniforme
}
```

## 🏗️ **Implementação da Arquitetura Híbrida**

### Fase 1: **Simplificar Markets** 🧹

```rust
// markets/src/lib.rs - Manter apenas abstrações essenciais
pub trait Exchange {
    type ExchangeId: Copy + Eq + Hash;
    fn id(&self) -> Self::ExchangeId;
    fn name(&self) -> &'static str;
}

pub trait Instrument {
    type Symbol: Display + Clone;
    fn symbol(&self) -> &Self::Symbol;
    fn market(&self) -> &str;
}

pub trait Asset {
    fn symbol(&self) -> &str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side { Buy, Sell }

// ❌ Remover: BinanceSpot, BinanceFutures, Coinbase, etc.
// ❌ Remover: implementações específicas de instrumentos/assets
// ✅ Manter: apenas traits e enums básicos
```

### Fase 2: **B3 Implementa Abstrações** 🇧🇷

```rust
// data/src/exchange/b3/types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct B3ExchangeId;

pub struct B3Exchange;
impl markets::Exchange for B3Exchange {
    type ExchangeId = B3ExchangeId;
    fn id(&self) -> Self::ExchangeId { B3ExchangeId }
    fn name(&self) -> &'static str { "B3" }
}

pub struct B3Instrument {
    symbol: SmolStr,
    market: B3Market,
}

impl markets::Instrument for B3Instrument {
    type Symbol = SmolStr;
    fn symbol(&self) -> &Self::Symbol { &self.symbol }
    fn market(&self) -> &str { self.market.as_str() }
}

// 🇧🇷 Métodos B3-específicos extras
impl B3Instrument {
    pub fn bovespa(symbol: impl Into<SmolStr>) -> Self { ... }
    pub fn bmf(symbol: impl Into<SmolStr>) -> Self { ... }
    pub fn is_stock(&self) -> bool { ... }
    pub fn is_future(&self) -> bool { ... }
}
```

### Fase 3: **ExecutionClient Genérico** ⚙️

```rust
// execution/src/client/mod.rs
pub trait ExecutionClient<E, I, A>
where
    E: markets::Exchange,
    I: markets::Instrument + Clone,
    A: markets::Asset + Clone,
{
    type Config;
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn submit_order(
        &self,
        instrument: I,
        side: markets::Side,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<Order<E, I, A>, Self::Error>;
    
    async fn cancel_order(&self, order_id: OrderId) -> Result<(), Self::Error>;
    async fn get_balances(&self) -> Result<Vec<Balance<A>>, Self::Error>;
}

// execution/src/client/b3/mod.rs
pub struct B3ExecutionClient {
    connector: Box<dyn B3Connector>,
}

impl ExecutionClient<B3Exchange, B3Instrument, B3Asset> for B3ExecutionClient {
    type Config = B3Config;
    type Error = B3ExecutionError;
    
    async fn submit_order(
        &self,
        instrument: B3Instrument,
        side: markets::Side,
        quantity: Decimal,
        price: Option<Decimal>,
    ) -> Result<Order<B3Exchange, B3Instrument, B3Asset>, Self::Error> {
        // Usar connector (ProfitDLL, REST, WebSocket, etc.)
        self.connector.submit_order(instrument, side, quantity, price).await
    }
}
```

## 📖 **Exemplo de Uso**

```rust
use toucan_data::exchange::b3::{
    B3ProfitConnector, B3Instrument, B3SubscriptionType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 🔌 Criar conector (ProfitDLL como exemplo)
    let mut connector = B3ProfitConnector::new();
    
    // 🏛️ Definir instrumentos B3
    let petr4 = B3Instrument::bovespa("PETR4");
    let vale3 = B3Instrument::bovespa("VALE3");
    
    // 📡 Conectar e subscrever
    connector.initialize("key", "user", "pass").await?;
    connector.subscribe_instrument(&petr4)?;
    connector.subscribe_instrument(&vale3)?;
    
    // 📊 Processar eventos
    while let Some(event) = connector.process_events().await {
        match event {
            B3MarketEvent::NewTrade { trade } => {
                println!("📈 Trade: {} {} @ R${}", 
                    trade.instrument, trade.quantity, trade.price);
            }
            B3MarketEvent::DailySummary { instrument, open, close, .. } => {
                println!("📊 {} - Abertura: R${} Fechamento: R${}", 
                    instrument, open, close);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

## 🎯 **Vantagens Competitivas**

### 🇧🇷 **Foco no Mercado Brasileiro**
- Terminologia nativa (BOVESPA, BMF, etc.)
- Regras específicas da B3
- Tipos de ativos brasileiros
- Moeda em Real (BRL)

### 🔄 **Flexibilidade de Conectividade**
- **ProfitDLL**: Dados em tempo real da Nelógica
- **APIs Oficiais**: Acesso direto à B3 (futuro)
- **Mock/Simulação**: Testes e desenvolvimento
- **Híbrido**: Combinação de múltiplas fontes

### 🚀 **Facilidade de Extensão**
- Novos conectores implementam apenas o trait `B3Connector`
- Tipos B3 reutilizáveis entre todos os conectores
- Zero dependência em abstrações genéricas
- Fácil adição de novas funcionalidades B3-específicas

### 🧪 **Testabilidade**
- Mock connectors para testes unitários
- Simulação de cenários de mercado
- Desenvolvimento offline sem credenciais
- CI/CD simplificado

## 📋 **Comparação: Antes vs Híbrida vs Sem Markets**

| Aspecto | Antes | Híbrida (Proposta) | Sem Markets |
|---------|-------|-------------------|-------------|
| **Abstrações** | ✅ Markets completa | ✅ Markets simplificada | ❌ Nenhuma reutilização |
| **B3-Específico** | ❌ Tipos genéricos | ✅ Implementações nativas | ✅ Tipos nativos |
| **Binance/Outros** | ❌ Incluídas | ✅ Removidas | ✅ Não existem |
| **Extensibilidade** | ✅ Alta | ✅ Alta | ⚠️ Média |
| **Compatibilidade** | ✅ Engine funciona | ✅ Engine funciona | ❌ Requer refatoração |
| **Complexidade** | ❌ Alta | ✅ Média | ✅ Baixa |
| **Tipagem** | ✅ Forte | ✅ Forte + específica | ✅ Específica |
| **Testabilidade** | ✅ Boa | ✅ Excelente | ✅ Boa |

### 🎯 **Recomendação: Arquitetura Híbrida**

**Por quê?**
1. **✅ Mantém compatibilidade** com engine e execution existentes
2. **✅ Remove complexidade** desnecessária (Binance, etc.)
3. **✅ Adiciona especificidade B3** sem perder abstrações
4. **✅ Permite extensão futura** para outras exchanges se necessário
5. **✅ Melhor testabilidade** com tipos específicos + abstrações

## 🎉 **Status e Próximos Passos**

### ✅ **Já Implementado**
- **B3-specific types**: B3Exchange, B3Instrument, B3Asset
- **ProfitDLL connector**: B3ProfitConnector funcional
- **Event system**: B3MarketEvent com conversões
- **Modular connectivity**: Trait B3Connector
- **Example code**: Demonstração funcional

### 🔄 **Próximos Passos para Híbrida**

1. **Simplificar Markets** (1-2 horas)
   ```bash
   # Remover exchanges desnecessárias
   rm -rf markets/src/exchange/{binance,coinbase}/*
   
   # Manter apenas traits essenciais
   # Simplificar markets/src/lib.rs
   ```

2. **B3 Implementa Traits** (2-3 horas)
   ```rust
   // Fazer B3Exchange implementar markets::Exchange
   // Fazer B3Instrument implementar markets::Instrument  
   // Fazer B3Asset implementar markets::Asset
   ```

3. **ExecutionClient Genérico** (3-4 horas)
   ```rust
   // Refatorar ExecutionClient para ser genérico
   // Atualizar B3ExecutionClient para usar types
   // Manter compatibilidade com engine
   ```

4. **Testes e Validação** (1-2 horas)
   ```rust
   // Garantir que todos os testes passam
   // Validar compatibilidade com core engine
   // Criar testes específicos para híbrida
   ```

### 💭 **Decisão Necessária**

**Qual caminho seguir?**

1. **🔄 Implementar Híbrida** (recomendado)
   - Manter abstrações úteis
   - B3-específico onde importa
   - Compatibilidade mantida

2. **🗑️ Remover Markets** (alternativa)
   - Mais simples
   - Menos compatibilidade
   - Foco total em B3

**Vote na abordagem preferida!** �️

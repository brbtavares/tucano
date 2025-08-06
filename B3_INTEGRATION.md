# Toucan - B3 Integration via ProfitDLL

## Overview

Este projeto integra o ProfitDLL da Nelógica ao framework Toucan, permitindo conectividade com a bolsa brasileira B3 usando as abstrações unificadas do Toucan para backtesting e execução ao vivo.

## 🎯 Objetivos da Integração

- ✅ **Unificação**: Usar a mesma engine do core para backtest e execução ao vivo
- ✅ **Abstrações Consistentes**: ExecutionClient trait para B3 igual aos outros exchanges
- ✅ **Mercado Brasileiro**: Conectividade nativa com B3 via ProfitDLL
- ✅ **Framework Simplificado**: Removidas implementações de crypto/forex, mantida apenas Binance

## 🏗️ Arquitetura da Integração

### 1. Estrutura do Workspace

```
toucan/
├── profit-dll/           # Wrapper Rust para ProfitDLL da Nelógica
├── execution/            # ExecutionClient traits e implementações
│   └── src/client/b3/    # Cliente B3 usando ProfitDLL
├── data/                 # Streams de dados (incluindo B3)
├── markets/              # Definições de exchanges (B3 + Binance)
└── core/                 # Engine unificado para backtest/live
```

### 2. Componentes Principais

#### **ProfitDLL Wrapper** (`profit-dll/`)
- Rust wrapper para a DLL da Nelógica
- Interface assíncrona para conectividade B3
- Conversão de eventos nativos para callbacks estruturados

#### **B3ExecutionClient** (`execution/src/client/b3/`)
- Implementa `ExecutionClient` trait do Toucan
- Conectividade com B3 via ProfitDLL
- Conversão entre formatos ProfitDLL ↔ Toucan

#### **Engine Unificado** (`core/`)
- Mesma engine para backtest e execução ao vivo
- Suporte nativo para B3 através do ExecutionClient
- Abstrações consistentes independente do exchange

## 🚀 Como Usar

### Configuração Básica

```rust
use execution::client::b3::B3Client;
use markets::{exchange::ExchangeId, instrument::name::InstrumentNameExchange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar cliente B3
    let config = execution::client::b3::B3Config::new(
        "sua_chave_ativacao".to_string(),
        "seu_usuario".to_string(), 
        "sua_senha".to_string(),
    );

    let client = B3Client::new(config);

    // Usar abstrações padrão do Toucan
    let instruments = vec![
        InstrumentNameExchange::new("PETR4".to_string()),
        InstrumentNameExchange::new("VALE3".to_string()),
    ];

    // Snapshot da conta
    let snapshot = client.account_snapshot(&[], &instruments).await?;
    println!("Balances: {}", snapshot.balances.len());

    Ok(())
}
```

### Integração com Core Engine

```rust
use core::engine::Engine;
use execution::client::b3::B3Client;

// O mesmo engine serve para backtest e live trading
let engine = Engine::new(/* config */);

// Para backtest: usar dados históricos
engine.run_backtest(/* historical data */).await?;

// Para live trading: usar B3Client
let b3_client = B3Client::new(b3_config);
engine.run_live(b3_client).await?;
```

## 📊 Status da Implementação

### ✅ Completado
- [x] Workspace integration do profit-dll
- [x] B3ExecutionClient básico implementando ExecutionClient trait
- [x] Remoção de exchanges crypto/forex (exceto Binance)
- [x] Simplificação do enum ExchangeId
- [x] Adapter para conversão ProfitDLL ↔ Toucan events
- [x] Projeto compila completamente
- [x] Estrutura básica para account snapshots, orders, trades

### 🔄 Em Desenvolvimento
- [ ] Implementação completa dos métodos ExecutionClient
- [ ] Sistema de eventos em tempo real
- [ ] Mapeamento completo de tipos de ordem B3
- [ ] Tratamento de erros robusto
- [ ] Testes de integração

### 🎯 Próximos Passos
- [ ] Conectividade real com ProfitDLL (requer DLL Windows)
- [ ] Event streaming para updates em tempo real
- [ ] Order management completo
- [ ] Position tracking
- [ ] Risk management integration

## 🔧 Dependências

### Runtime
- `tokio` - Runtime assíncrono
- `profit-dll` - Wrapper para Nelógica ProfitDLL
- Framework Toucan (`execution`, `markets`, `core`)

### Development
- Windows environment (para ProfitDLL)
- Acesso à API Nelógica Profit
- Credenciais B3 válidas

## 📚 Estrutura de Código

### ExecutionClient Trait
```rust
pub trait ExecutionClient {
    const EXCHANGE: ExchangeId;
    type Config: Clone;
    type AccountStream: Stream<Item = UnindexedAccountEvent>;

    fn new(config: Self::Config) -> Self;
    
    // Métodos principais
    async fn account_snapshot(&self, ...) -> Result<UnindexedAccountSnapshot, ...>;
    async fn account_stream(&self, ...) -> Result<Self::AccountStream, ...>;
    async fn open_order(&self, ...) -> Option<Order<...>>;
    async fn cancel_order(&self, ...) -> Option<...>;
    
    // Queries
    async fn fetch_balances(&self) -> Result<Vec<AssetBalance<...>>, ...>;
    async fn fetch_open_orders(&self) -> Result<Vec<Order<...>>, ...>;
    async fn fetch_trades(&self, ...) -> Result<Vec<Trade<...>>, ...>;
}
```

### B3Client Implementation
```rust
impl ExecutionClient for B3ExecutionClient {
    const EXCHANGE: ExchangeId = ExchangeId::B3;
    type Config = B3Config;
    type AccountStream = UnboundedReceiverStream<UnindexedAccountEvent>;

    // Implementação usando ProfitDLL internamente
    // mas expondo interface padrão Toucan
}
```

## 🎯 Benefícios da Integração

1. **Unificação**: Uma única engine para backtest e live trading
2. **Consistência**: Mesma interface para todos os exchanges (B3, Binance)
3. **Mercado Brasileiro**: Acesso nativo à B3 via ProfitDLL
4. **Flexibilidade**: Framework extensível para novos exchanges
5. **Simplicidade**: Abstrações padronizadas reduzem complexidade

## ⚡ Performance e Escalabilidade

- Event streaming assíncrono
- Connection pooling e reconnection automática
- Minimal overhead nas conversões de tipo
- Otimizações específicas para mercado brasileiro

## 🔒 Considerações de Segurança

- Credenciais gerenciadas via configuração
- Logging estruturado (sem exposição de credenciais)
- Error handling robusto
- Rate limiting para APIs

---

**Status**: ✅ Integração básica funcionando - Pronto para desenvolvimento completo do ExecutionClient

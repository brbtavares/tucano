# ProfitDLL Rust Wrapper

Um wrapper Rust completo para a biblioteca ProfitDLL, sistema brasileiro de integração com trading.

## 🚀 Funcionalidades

- ✅ **Interface Rust-friendly** para ProfitDLL
- ✅ **Gerenciamento de Conexão** - Login, market data, routing
- ✅ **Callbacks Assíncronos** usando Tokio channels
- ✅ **Tipos Seguros** com validação automática
- ✅ **Tratamento de Erros** robusto com `thiserror`
- ✅ **Multiplataforma** - Windows (funcional) + Linux/Mac (mock para desenvolvimento)
- ✅ **Documentação** completa com exemplos

## 📦 Instalação

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
profit-dll = { path = "../profit-dll", features = ["async"] }
tokio = { version = "1.0", features = ["full"] }
```

## 🔧 Uso Básico

### Inicialização e Login

```rust
use profit_dll::{ProfitConnector, ProfitError};

#[tokio::main]
async fn main() -> Result<(), ProfitError> {
    // Criar connector (None = usar DLL padrão do sistema)
    let connector = ProfitConnector::new(None)?;
    
    // Login com eventos assíncronos
    let mut events = connector.initialize_login("user", "password", "activation_key").await?;
    
    // Processar eventos de callback
    while let Some(event) = events.recv().await {
        match event {
            CallbackEvent::StateChanged { connection_type, result } => {
                println!("Estado mudou: {:?} -> {}", connection_type, result);
            }
            CallbackEvent::NewTrade { ticker, price, volume, .. } => {
                println!("Nova negociação: {} - R${} vol:{}", ticker, price, volume);
            }
            // ... outros eventos
            _ => {}
        }
    }
    
    Ok(())
}
```

### Market Data

```rust
// Subscrever cotações
connector.subscribe_ticker("PETR4", "BOVESPA")?;
connector.subscribe_price_book("VALE3", "BOVESPA")?;

// Subscrever múltiplos ativos
let tickers = [("ITUB4", "BOVESPA"), ("BBDC4", "BOVESPA")];
connector.subscribe_multiple_tickers(&tickers)?;
```

### Envio de Ordens

```rust
use profit_dll::{SendOrder, OrderType, OrderSide};

// Ordem de compra
let order = SendOrder {
    account: "12345".into(),
    ticker: "PETR4".into(),
    exchange: "BOVESPA".into(),
    order_type: OrderType::Market,
    side: OrderSide::Buy,
    quantity: 100,
    price: None, // Market order
    validity: None,
    // ... outros campos
};

let order_id = connector.send_order(&order)?;
println!("Ordem enviada: ID {}", order_id);
```

### Consulta de Posições

```rust
let position = connector.get_position(
    "12345",    // account_id
    "PETR4",    // ticker  
    "BOVESPA",  // exchange
    "VISTA"     // category
)?;

println!("Posição: {} unidades", position.quantity);
```

## 🏗️ Arquitetura

```
profit-dll/
├── src/
│   ├── lib.rs          # API pública
│   ├── connector.rs    # Connector principal
│   ├── types.rs        # Tipos de dados
│   ├── callbacks.rs    # Sistema de eventos
│   ├── utils.rs        # Interface DLL
│   └── error.rs        # Tratamento de erros
├── examples/           # Exemplos de uso
└── README.md
```

### Componentes Principais

1. **ProfitConnector** - Interface principal de alto nível
2. **CallbackEvent** - Eventos assíncronos do sistema
3. **DllLoader** - Interface de baixo nível com Windows DLL
4. **Tipos Rust** - Structs e enums type-safe

## 🔄 Sistema de Callbacks

O wrapper converte callbacks síncronos da DLL em streams assíncronos:

```rust
// A DLL chama funções C extern "stdcall"
extern "stdcall" fn state_callback(conn_type: i32, result: i32);

// Convertido em eventos Rust assíncronos
enum CallbackEvent {
    StateChanged { connection_type: ConnectionState, result: i32 },
    NewTrade { ticker: String, price: Decimal, ... },
    // ...
}
```

## 🛠️ Features

- **`async`** (padrão) - Habilita funcionalidades Tokio
- **`serde`** - Serialização JSON automática dos tipos

## 📋 Requisitos

### Windows (Produção)
- Windows 10/11
- ProfitDLL instalada no sistema
- Rust 1.70+

### Linux/Mac (Desenvolvimento)
- Funciona como mock para desenvolvimento
- Todos os métodos retornam `ProfitError::NotInitialized`

## 🧪 Testes e Exemplos

```bash
# Compilar o wrapper
cargo build --release

# Executar exemplo básico
cargo run --example minimal_test

# Executar com features específicas
cargo run --example basic_usage --features async

# Executar testes
cargo test
```

## 📊 Tipos de Dados Suportados

### Ordens
- `SendOrder` - Estrutura completa de ordem
- `OrderType` - Market, Limit, Stop, etc.
- `OrderSide` - Buy/Sell
- `OrderStatus` - Estados da ordem

### Market Data
- `Trade` - Negociação individual
- `DailySummary` - Resumo diário do ativo
- `Position` - Posição atual
- `BookLevel` - Nível do book de ofertas

### Identificadores
- `AccountIdentifier` - Conta de trading
- `AssetIdentifier` - Ativo financeiro

## ⚠️ Notas Importantes

1. **Thread Safety**: O wrapper é thread-safe usando `Arc` e `RwLock`
2. **Memory Management**: Gerenciamento automático de memória
3. **Error Handling**: Todos os códigos de erro da DLL são mapeados
4. **Windows Only**: A DLL real só funciona no Windows

## 🔗 Integração com Toucan

Este wrapper foi projetado para integrar-se ao ecossistema Toucan:

```rust
// No seu sistema de trading Toucan
use profit_dll::ProfitConnector;
use core::execution::ExecutionEngine;

let profit = ProfitConnector::new(None)?;
let execution_engine = ExecutionEngine::new(Box::new(profit));
```

## 📈 Status do Projeto

- [x] ✅ Estrutura básica
- [x] ✅ Tipos de dados completos  
- [x] ✅ Sistema de callbacks
- [x] ✅ Interface de alto nível
- [x] ✅ Tratamento de erros
- [x] ✅ Exemplos e documentação
- [x] ✅ Compilação em múltiplas plataformas
- [ ] 🔄 Testes com DLL real (requer Windows)
- [ ] 🔄 Otimizações de performance
- [ ] 🔄 Mais exemplos avançados

---

## 📝 Licença

MIT License - veja o arquivo LICENSE para detalhes.

## 🤝 Contribuições

Contribuições são bem-vindas! Abra issues para bugs ou PRs para melhorias.

---

**Nota**: Este wrapper foi criado para facilitar a integração da ProfitDLL com aplicações Rust modernas, mantendo toda a funcionalidade original mas com ergonomia e segurança Rust.

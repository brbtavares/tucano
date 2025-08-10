# 🇧🇷 Toucan - Framework de Trading Algorítmico para B3

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![B3](https://img.shields.io/badge/exchange-B3-green.svg)](http://www.b3.com.br)

Framework moderno de trading algorítmico desenvolvido em Rust, especializado no mercado brasileiro (B3) com integração nativa via ProfitDLL da Nelógica.

## 🎯 Características Principais

- **🧠 Engine Unificado**: Mesmo código para backtesting e execução ao vivo
- **🇧🇷 Mercado Brasileiro**: Integração nativa com B3 via ProfitDLL
- **⚡ Alta Performance**: Desenvolvido em Rust para máxima eficiência
- **🛡️ Type Safety**: Sistema de tipos que previne erros em tempo de compilação
- **📊 Analytics**: Métricas financeiras abrangentes (Sharpe, Sortino, Drawdown)
- **🔄 Modular**: Arquitetura extensível e componentes reutilizáveis

## 🏗️ Arquitetura do Sistema

```
toucan/
├── 🧠 core/              # Engine principal - backtesting e execução
├── 📊 analytics/         # Métricas de performance e análise quantitativa
├── 📈 data/              # Streaming de dados de mercado em tempo real
├── 🏛️ markets/           # Abstrações de exchanges e instrumentos
├── ⚡ execution/         # Execução de ordens em exchanges
├── 🔄 integration/       # Protocolos de comunicação (HTTP, WebSocket)
├── 🛡️ risk/              # Gestão de risco e validações
├── 🧩 strategy/          # Framework de estratégias algorítmicas
├── 🔧 macros/            # Macros Rust para geração de código
└── 📝 examples/          # Exemplos práticos de uso
```

### Filosofia de Design

O Toucan implementa uma **arquitetura híbrida** que combina:
- **Abstrações Reutilizáveis**: Traits genéricos para máxima flexibilidade
- **Implementações B3**: Tipos brasileiros com terminologia nativa
- **Conectividade Modular**: Fácil extensão para novos exchanges

## 🚀 Início Rápido

### Pré-requisitos

```bash
# Instalar Rust (versão 1.75 ou superior)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar o repositório
git clone https://github.com/brbtavares/toucan.git
cd toucan
```

### Compilação

```bash
# Compilar todo o workspace
cargo build --release

# Executar testes
cargo test

# Formatação de código (usa rustfmt.toml)
cargo fmt

# Verificar formatação (CI/CD)
cargo fmt --check

# Lint de código (usa .config/clippy.toml)
cargo clippy -- -D warnings

# Script personalizado de formatação
./scripts/format.sh
./scripts/format.sh --check

# Executar exemplo básico
cargo run --example basic_b3_usage

# Gerar documentação
cargo doc --open
```

### Configuração para B3

```bash
# Variáveis de ambiente
export PROFIT_DLL_PATH="/path/to/ProfitDLL.dll"  # Windows
export B3_USERNAME="seu_usuario"
export B3_PASSWORD="sua_senha"
export RUST_LOG=info
```

## 💡 Exemplo de Uso

### Estratégia Básica de Médias Móveis

```rust
use toucan::{
    core::engine::Engine,
    strategy::algo::AlgoStrategy,
    execution::client::b3::B3Client,
    analytics::metric::sharpe::SharpeRatio,
};

// Definir estratégia
struct MovingAverageStrategy {
    short_period: usize,
    long_period: usize,
}

impl AlgoStrategy for MovingAverageStrategy {
    type State = EngineState<GlobalData, InstrumentData>;
    
    fn generate_algo_orders(&self, state: &Self::State) -> (Vec<CancelOrder>, Vec<OpenOrder>) {
        let mut orders = Vec::new();
        
        for (instrument, data) in state.market_data.iter() {
            let short_ma = calculate_ma(&data.prices, self.short_period);
            let long_ma = calculate_ma(&data.prices, self.long_period);
            
            if short_ma > long_ma {
                orders.push(Order::market_buy(instrument, 100.0));
            }
        }
        
        (vec![], orders)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar cliente B3
    let b3_client = B3Client::new()
        .with_credentials(&env::var("B3_USERNAME")?, &env::var("B3_PASSWORD")?)
        .connect().await?;
    
    // Configurar estratégia
    let strategy = MovingAverageStrategy {
        short_period: 10,
        long_period: 30,
    };
    
    // Criar engine
    let mut engine = Engine::new(
        UTCClock::new(),
        EngineState::new(),
        vec![b3_client.execution_tx()],
        strategy,
        RiskManager::default(),
    );
    
    // Loop principal de trading
    loop {
        // Processar eventos de mercado
        if let Some(market_event) = market_stream.next().await {
            let audit = engine.process(EngineEvent::Market(market_event));
            println!("Processado: {:?}", audit);
        }
    }
}
```

### Backtesting

```rust
use toucan::{
    core::backtest::BacktestEngine,
    analytics::summary::TradingSummary,
    data::historical::HistoricalDataProvider,
};

async fn run_backtest() -> Result<(), Box<dyn std::error::Error>> {
    // Dados históricos
    let data_provider = HistoricalDataProvider::new()
        .with_instruments(vec!["PETR4", "VALE3", "ITUB4"])
        .with_period("2023-01-01", "2023-12-31");
    
    // Configurar backtest
    let config = BacktestConfig::new()
        .with_initial_capital(100_000.0)
        .with_commission_rate(0.001);
    
    // Executar
    let mut engine = BacktestEngine::new(config, strategy);
    let results = engine.run(data_provider).await?;
    
    // Analisar resultados
    println!("Retorno Total: {:.2}%", results.total_return * 100.0);
    println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", results.max_drawdown * 100.0);
    println!("Win Rate: {:.2}%", results.win_rate * 100.0);
    
    Ok(())
}
```

## 🇧🇷 Integração B3 via ProfitDLL

### Configuração Windows

```rust
use toucan::markets::profit_dll::ProfitDLLClient;

// Configurar ProfitDLL
let client = ProfitDLLClient::new()
    .with_dll_path("C:/Program Files/Nelogica/ProfitDLL.dll")
    .with_credentials(username, password)
    .initialize().await?;

// Subscrever dados
client.subscribe_ticker("PETR4").await?;
client.subscribe_price_book("PETR4").await?;

// Enviar ordem
let order = Order::limit_buy("PETR4", 25.50, 100);
client.send_order(order).await?;
```

### Instrumentos Suportados

```rust
use toucan::markets::b3::{B3Stock, B3Option, B3Future};

// Ações
let petr4 = B3Stock::new("PETR4");
let vale3 = B3Stock::new("VALE3");

// Opções
let petr_call = B3Option::call("PETRJ45", "PETR4", 45.0, "2024-01-15");

// Futuros
let dol_future = B3Future::new("DOLM24", "USD", "2024-12-31");
```

## 📊 Métricas e Analytics

### Métricas Disponíveis

```rust
use toucan::analytics::metric::*;

// Sharpe Ratio
let sharpe = SharpeRatio::calculate(&returns, risk_free_rate)?;

// Sortino Ratio (downside risk)
let sortino = SortinoRatio::calculate(&returns, target_return)?;

// Maximum Drawdown
let max_dd = MaxDrawdown::calculate(&portfolio_values)?;

// Win Rate
let win_rate = WinRate::calculate(&trades)?;

// Profit Factor
let pf = ProfitFactor::calculate(&trades)?;
```

### Relatórios Automatizados

```rust
use toucan::analytics::summary::TradingSummary;

let summary = TradingSummary::generate(&trades, &positions)?;
println!("{}", summary.display_table());

// Output:
// ┌─────────────────┬──────────────┐
// │ Métrica         │ Valor        │
// ├─────────────────┼──────────────┤
// │ Retorno Total   │ 15.3%        │
// │ Sharpe Ratio    │ 1.45         │
// │ Max Drawdown    │ -8.2%        │
// │ Win Rate        │ 62.5%        │
// └─────────────────┴──────────────┘
```

## 🛡️ Gestão de Risco

### Implementação Básica

```rust
use toucan::risk::{RiskManager, RiskCheck};

struct MyRiskManager {
    max_position_size: f64,
    max_daily_loss: f64,
}

impl RiskManager for MyRiskManager {
    fn check_order(&self, order: &Order) -> RiskResult<Order> {
        // Verificar tamanho da posição
        if order.quantity > self.max_position_size {
            return Err(RiskRefused::new(order.clone(), "Excede limite de posição"));
        }
        
        // Verificar perda diária
        if current_daily_loss() > self.max_daily_loss {
            return Err(RiskRefused::new(order.clone(), "Excede perda diária máxima"));
        }
        
        Ok(RiskApproved::new(order.clone()))
    }
}
```

## 🔧 Desenvolvimento

### Comandos Úteis

```bash
# Formatação de código
cargo fmt

# Lint
cargo clippy -- -D warnings

# Gerar documentação
cargo doc --open

# Benchmarks
cargo bench

# Testes específicos
cargo test -p core --test engine_tests
```

### Formatação Automática

O projeto usa [`rustfmt.toml`](rustfmt.toml) para garantir código consistente:

- **VS Code**: Formatação automática ao salvar (configurado em `.vscode/settings.json`)
- **CI/CD**: Verificação automática no GitHub Actions
- **Manual**: Execute `cargo fmt` para formatar todo o código

```bash
# Verificar se código está formatado (usado no CI)
cargo fmt --check

# Formatar automaticamente
cargo fmt
```

### Estrutura de Testes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_strategy_execution() {
        let mut engine = create_test_engine();
        let market_data = mock_market_data();
        
        let result = engine.process_market_data(market_data).await;
        assert!(result.is_ok());
    }
}
```

### Debugging

```rust
use tracing::{info, warn, error, debug};

// Setup de logging
tracing_subscriber::fmt()
    .with_env_filter("toucan=debug")
    .init();

// Logs em código
debug!("Processando ordem: {:?}", order);
info!("Posição atualizada: {}", position);
warn!("Limite de risco próximo: {}", exposure);
```

## 📈 Análise de Implementação ProfitDLL

### Status Atual (~30% de Cobertura)

**✅ Implementadas (15 funções)**
- Inicialização e autenticação
- Subscrições básicas de market data
- Ordens básicas (buy/sell/cancel)
- Callbacks principais

**🔴 Ausentes (35 funções)**
- Ordens avançadas (market, stop, modificação)
- Gestão avançada de posições
- Análise técnica integrada
- Configurações de sessão

### Prioridades de Desenvolvimento

1. **🔥 Alta Prioridade**
   - `SendMarketBuyOrder/SellOrder`
   - `SendStopBuyOrder/SellOrder`
   - `SendChangeOrder`

2. **🔶 Média Prioridade**
   - `GetCurrentPosition`
   - `GetDayTrades`
   - `SubscribeIndicator`

3. **🔵 Baixa Prioridade**
   - Análise técnica avançada
   - Configurações específicas

## 🚀 Deployment

### Configuração de Produção

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### Docker

```dockerfile
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/toucan /usr/local/bin/
CMD ["toucan"]
```

### Variáveis de Ambiente

```bash
# Produção
export RUST_ENV=production
export RUST_LOG=info
export B3_USERNAME=usuario_producao
export B3_PASSWORD=senha_producao
export DATABASE_URL=postgresql://user:pass@localhost/toucan
export REDIS_URL=redis://localhost:6379
```

## 📋 Roadmap

### Versão 1.0 (Atual)
- ✅ Core engine funcional
- ✅ Integração B3 básica
- ✅ Estratégias algorítmicas
- ✅ Métricas financeiras
- ✅ Backtesting

### Versão 1.1
- 🔄 Ordens avançadas ProfitDLL
- 🔄 WebSocket para dados real-time
- 🔄 Dashboard web
- 🔄 Alertas automáticos

### Versão 1.2
- 📋 Machine Learning integrado
- 📋 Multi-threading otimizado
- 📋 Métricas avançadas de risco
- 📋 API REST para controle externo

---

**Toucan** - Trading algorítmico moderno para o mercado brasileiro 🇧🇷  
*Desenvolvido com ❤️ em Rust*

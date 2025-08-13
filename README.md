# 🇧🇷 Tucano - Framework de Trading Algorítmico para B3 (anteriormente Toucan)

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![B3](https://img.shields.io/badge/exchange-B3-green.svg)](http://www.b3.com.br)

Framework moderno de trading algorítmico desenvolvido em Rust, especializado no mercado brasileiro (B3). Rebranding: Toucan → Tucano.

## 🎯 Características Principais

- **🧠 Engine Unificado**: Mesmo código para backtest e execução (real ou simulada) ao vivo
- **🇧🇷 Mercado Brasileiro**: Integração nativa com B3 já implementada via ProfitDLL
- **⚡ Alta Performance**: Desenvolvido em Rust para máxima eficiência
- **🛡️ Type Safety**: Sistema de tipos que previne erros em tempo de compilação
- **🔄 Modular**: Arquitetura extensível e componentes reutilizáveis

## 🏗️ Arquitetura do Sistema

```
tucano/  # diretório original "toucan/" permanece até renomear repo
├── 🧠 core/              # Engine principal (processamento de eventos, backtest & live)
├── 📊 analytics/         # Métricas financeiras e resumos
├── 📈 data/              # Eventos & streaming de dados (livros, trades, assinaturas)
├── 🏛️ markets/           # Modelos de instrumentos, exchange catalog & tipos B3
├── 🤝 brokers/           # Registro/carregamento de brokers & modelos de conta
├── ⚡ execution/         # Camada de execução (ordens, clientes, transporte, mapping)
├── 🔌 integration/       # Protocolos externos (canal, stream, serialização)
├── 🛡️ risk/              # Gestão de risco (checks/validações)
├── 🧩 trader/            # Abstrações (traits + tipos) para estratégias
├── 📦 strategies/        # Implementações concretas de estratégias (features)
├── 🔧 macros/            # Macros Rust para geração de código
├── 📝 examples/          # Exemplos práticos de uso
└── 🛠️ scripts/           # Scripts utilitários (format, automações)
```

### Filosofia de Design

O Tucano (ex-Toucan) implementa uma **arquitetura híbrida** que combina:
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

### Estratégia Reutilizável: Order Book Imbalance

Uma estratégia simples que observa o desequilíbrio entre volumes BID e ASK no melhor nível do livro. A mesma implementação pode ser plugada tanto em um motor live quanto em um motor de backtest sem alterar a lógica.

```rust
use trader::AlgoStrategy;
use strategies::{
    order_book_imbalance::OrderBookImbalanceStrategy,
    shared::NoOpState,
};
use execution::{ExchangeIndex, InstrumentIndex};
use execution::order::request::{OrderRequestCancel, OrderRequestOpen};

// Wrapper leve para demonstrar o trait (delegaria internamente para a estratégia real).
struct MyImbalance(OrderBookImbalanceStrategy);

impl AlgoStrategy for MyImbalance {
    type State = NoOpState; // estado do engine (placeholder)

    fn generate_algo_orders(
        &self,
        _state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        // Aqui chamaríamos self.0.generate_algo_orders(...) quando integrado ao estado real
        (Vec::<OrderRequestCancel<_, _>>::new(), Vec::<OrderRequestOpen<_, _>>::new())
    }
}
```

### Uso em Live vs Backtest (mesma estratégia)

```rust
// Live
let strategy = MyImbalance(OrderBookImbalanceStrategy::new(Default::default()));
let engine_live = Engine::new(clock, live_state, live_exec_txs, strategy, risk_manager);

// Backtest
let strategy_bt = MyImbalance(OrderBookImbalanceStrategy::new(Default::default()));
let engine_bt = BacktestEngine::new(bt_config, bt_state, bt_exec_txs, strategy_bt, risk_manager_bt);
```

Somente os componentes de dados (streaming vs histórico) e de execução (cliente real vs simulado) mudam; a estratégia permanece idêntica.

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
    .with_env_filter("tucano=debug")
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
# (Após renomear binário no futuro)
COPY --from=builder /app/target/release/tucano /usr/local/bin/
CMD ["tucano"]
```

### Variáveis de Ambiente

```bash
# Produção
export RUST_ENV=production
export RUST_LOG=info
export B3_USERNAME=usuario_producao
export B3_PASSWORD=senha_producao
export DATABASE_URL=postgresql://user:pass@localhost/tucano
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

**Tucano** (anteriormente Toucan) - Trading algorítmico moderno para o mercado brasileiro 🇧🇷  
*Desenvolvido com ❤️ em Rust*

## 🧭 Roadmap de Arquitetura (Exchange vs Broker vs Transporte)

Objetivo: separar claramente três camadas hoje parcialmente acopladas.

1. Exchange (mercado)
    - Representado por `ExchangeId` (enum).
    - Responsável por catálogo de instrumentos, normalização de símbolos, calendários.
2. Broker / Account (corretora)
    - Nova identificação: `BrokerId`, `AccountId`.
    - Responsável por saldos, posições, envio de ordens (semântica de conta), limites e permissões.
3. Transporte / Adapter
    - Abstrai meio físico/protocolo (DLL Profit, WebSocket, FIX, REST).
    - Exposto via trait (futuro) `TransportAdapter` (connect, subscribe, send, shutdown).

### Estado Atual (antes da refatoração)
`ExchangeId` é usado como chave para tudo. Código da Profit DLL mistura: lógica de broker (account events), lógica de exchange (símbolos) e transporte (chamadas FFI) no mesmo módulo.

### Fases Planejadas
Fase 1 (iniciada): Introduzir aliases `BrokerId` e `AccountId` para permitir evolução sem quebra.
Fase 2: Extrair módulo `transport::profit_dll` contendo somente IO/FFI; deixar conversões em adapter.
Fase 3: Criar trait `BrokerAccount` para operações de conta/ordem (usa internamente um `TransportAdapter`).
Fase 4: Criar trait `ExchangeCatalogue` em `markets` para resolução de instrumentos e metadados.
Fase 5: Atualizar `ExecutionClient` para compor `BrokerAccount + ExchangeCatalogue` em vez de implementar tudo.
Fase 6: Revisar mapas de instrumentos para escopo `(ExchangeId, BrokerId)` evitando colisões multi-conta.
Fase 7: Estratificar erros: `TransportError`, `BrokerError`, `ExchangeRuleError`, mantendo `ClientError` como envelope.
Fase 8: Otimizações (índices numéricos, caching, normalização consistente B3).

### Benefícios
- Multi-conta e multi-broker sem refactor profundo futuro.
- Testes mais isolados (mock de transporte sem simular exchange inteira).
- Evolução de protocolos (ex: adicionar FIX) sem tocar em lógica de ordens.
- Claridade semântica → menos risco de confusões entre camadas.

### Métrica de Conclusão da Fase 1
- `BrokerId` e `AccountId` disponíveis em `execution::compat`.
- Documentação deste roadmap publicada (este bloco).
- Nenhuma quebra de build.

Próximos passos imediatos: propagar `BrokerId` (Option) em eventos de conta e depois extrair transporte ProfitDLL.

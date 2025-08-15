<div align="center">

# Tucano - Framework de Trading Algorítmico para B3


<table>
    <tr>
        <td align="center" valign="middle" style="border: none;">
            <img src="assets/logo.png" alt="Logo Tucano" />
        </td>
    </tr>
    <tr>
        <td align="left" valign="middle" style="border: none; padding-left: 16px;">
            <li><strong>Visão aguçada & alcance estratégico</strong>: monitora múltiplos mercados / books em tempo real para antecipar movimento.</li>
            <li><strong>Precisão & eficiência</strong>: envia ordens enxutas, evita latência e reduz fricção operacional.</li>
            <li><strong>Navegação em ambientes complexos</strong>: abstrai protocolos, streams e formatos heterogêneos.</li>
            <li><strong>Inteligência adaptativa</strong>: ajusta parâmetros & estratégias conforme regime de mercado.</li>
            <li><strong>Visão panorâmica</strong>: consolida dados multi‐fonte para decisão holística (preço, volume, risco, PnL).</li>
            <li><strong>Velocidade de resposta</strong>: loop de eventos otimizado para reagir a micro variações antes da competição.</li>
        </td>
    </tr>
</table>



[![Rust Version](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Crate](https://img.shields.io/crates/v/tucano.svg)](https://crates.io/crates/tucano)
[![Docs](https://img.shields.io/docsrs/tucano)](https://docs.rs/tucano)
[![CI](https://github.com/brbtavares/tucano/actions/workflows/ci.yml/badge.svg)](https://github.com/brbtavares/tucano/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![B3](https://img.shields.io/badge/exchange-B3-green.svg)](http://www.b3.com.br)

</div>

## 🎯 Características Principais

- **🧠 Engine Unificado**: Mesmo código para backtest e execução (real ou simulada) ao vivo
- **🇧🇷 Mercado Brasileiro**: Integração nativa com B3 já implementada via ProfitDLL
- **⚡ Alta Performance**: Desenvolvido em Rust para máxima eficiência
- **🛡️ Type Safety**: Sistema de tipos que previne erros em tempo de compilação
 - **🔄 Modular**: Arquitetura extensível e componentes reutilizáveis

## 📦 Crates Publicadas

| Crate | Versão | Descrição | Docs |
|-------|--------|-----------|------|
| `tucano` | 0.1.x | Façade unificada (re-export) | [docs.rs](https://docs.rs/tucano) |
| `tucano-core` | 0.12.x | Engine, execução/backtest unificado | [docs.rs](https://docs.rs/tucano-core) |
| `tucano-markets` | 0.3.x | Instrumentos & exchanges (B3, etc) | [docs.rs](https://docs.rs/tucano-markets) |
| `tucano-data` | 0.10.x | Eventos & pipeline de dados (streams, books, trades) | [docs.rs](https://docs.rs/tucano-data) |
| `tucano-execution` | 0.5.x | Ordens, fills, roteamento & clientes | [docs.rs](https://docs.rs/tucano-execution) |
| `tucano-trader` | 0.1.x | Traits centrais de estratégia & tipos | [docs.rs](https://docs.rs/tucano-trader) |
| `tucano-risk` | 0.1.x | Gestão de risco (limites, validações) | [docs.rs](https://docs.rs/tucano-risk) |
| `tucano-strategies` | 0.1.x | Estratégias de exemplo / referência | [docs.rs](https://docs.rs/tucano-strategies) |
| `tucano-analytics` | 0.1.x | Métricas, summaries, performance & PnL | [docs.rs](https://docs.rs/tucano-analytics) |
| `tucano-integration` | 0.9.x | Protocolos externos, canais & snapshots | [docs.rs](https://docs.rs/tucano-integration) |
| `tucano-macros` | 0.2.x | Procedural macros internas | [docs.rs](https://docs.rs/tucano-macros) |
| `tucano-profitdll` | 0.1.x | Integração ProfitDLL (mock + FFI opcional) | [docs.rs](https://docs.rs/tucano-profitdll) |

Convenção: usar intervalo `major.minor.x` nas docs; indique patch específico se precisar de reprodutibilidade.

## 🏗️ Arquitetura do Sistema (Visão Rápida)
`core/` (engine), `execution/` (ordens), `data/` (streams), `markets/` (instrumentos B3), `analytics/` (métricas), `risk/`, `trader/` (traits), `strategies/`, `integration/` (protocolos) e `examples/`.

## 🚀 Início Rápido

### Pré-requisitos

```bash
# Instalar Rust (versão 1.75 ou superior)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar o repositório (novo nome)
git clone https://github.com/brbtavares/tucano.git
cd tucano
```

### Adicionar dependência (façade)

No seu `Cargo.toml` adicione a crate unificada (re-export) — recomendado para começar:

```toml
[dependencies]
tucano = { version = "0.1", features = ["full" ] }
```

Ou, se quiser granularidade / compilar menos coisas, use crates individuais:

```toml
[dependencies]
tucano-core = "0.12"
tucano-markets = "0.3"
tucano-data = "0.10"
tucano-execution = "0.5"
tucano-trader = "0.1"
tucano-risk = "0.1"
tucano-strategies = "0.1"
tucano-analytics = "0.1"
```

Depois importe via façade:

```rust
use tucano::prelude::*; // Engine, ExchangeId, Side, etc.
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
use tucano_trader::AlgoStrategy;
use tucano_strategies::{
    order_book_imbalance::OrderBookImbalanceStrategy,
    shared::NoOpState,
};
use tucano_execution::{ExchangeIndex, InstrumentIndex};
use tucano_execution::order::request::{OrderRequestCancel, OrderRequestOpen};

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

## 🇧🇷 Integração B3 via ProfitDLL (conceitual)

A integração com a ProfitDLL foi extraída para a crate dedicada `profitdll` (anteriormente `tucano-profitdll`).
O exemplo abaixo mostra uso direto do conector mock atualmente disponível:

```rust
use profitdll::{ProfitConnector, CallbackEvent};

let connector = ProfitConnector::new(None)?;
let mut events = connector
    .initialize_login("ACTIVATION_KEY", "user", "pass")
    .await?;

// Exemplo de subscription (mock)
connector.subscribe_ticker("PETR4", "B")?;

while let Ok(event) = events.try_recv() {
    println!("Evento: {:?}", event);
}
```

### Instrumentos Suportados

```rust
use tucano::tucano_markets::b3::{B3Stock, B3Option, B3Future};

// Ações
let petr4 = B3Stock::new("PETR4");
let vale3 = B3Stock::new("VALE3");

// Opções
let petr_call = B3Option::call("PETRJ45", "PETR4", 45.0, "2024-01-15");

// Futuros
let dol_future = B3Future::new("DOLM24", "USD", "2024-12-31");
```

## 📊 Métricas & Analytics

### Métricas Disponíveis

```rust
use tucano::tucano_analytics::metric::*;

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
use tucano::tucano_analytics::summary::TradingSummary;

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

## 🛡️ Gestão de Risco (exemplo simplificado)

### Implementação Básica

```rust
use tucano::tucano_risk::{RiskManager, RiskApproved, RiskRefused};

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
# Verificação de mini-disclaimers (CI falha se ausentes)
./scripts/verify_disclaimers.sh
./scripts/verify_disclaimers.sh --fix  # injeta onde faltar

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

### Estrutura de Testes (exemplo)

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

## 🙏 Inspiração & Agradecimentos

Este projeto foi fortemente **inspirado no desenho arquitetural do [barter-rs](https://github.com/barter-rs/barter-rs)**, cuja estrutura inicial serviu como ponto de partida para organizar módulos, traits centrais e a abordagem de streaming/normalização. Nosso sincero agradecimento ao seu criador e a todos os demais desenvolvedores e contribuidores do ecossistema barter-rs – o trabalho de vocês facilitou acelerar a fase inicial deste framework.

---

## ⚠️ Disclaimer (Resumo) & Escopo Legal

Uso educacional/experimental. **Não é recomendação de investimento** nem consultoria financeira, jurídica, contábil ou tributária. Risco elevado: valide tudo em ambiente controlado (backtest / simulação) antes de qualquer operação real. Você é integralmente responsável por configurações, limites de risco, conformidade regulatória e monitoramento contínuo.

### Ausência de Afiliação
Autores e contribuidores **não são afiliados** nem possuem vínculo formal, societário, empregatício, contratual, de representação, patrocínio ou parceria com corretoras, bancos, fintechs/investechs, gestoras, consultorias ou agentes regulados.

### Nenhuma Remuneração de Terceiros
Não há recebimento de comissão, rebate, patrocínio ou qualquer vantagem econômica de terceiros em função deste projeto.

### Profit / ProfitDLL
"Profit", "ProfitChart", "Profit DLL" (ProfitDLL) e marcas correlatas são **propriedade da Nelógica**. A integração aqui exibida é meramente técnica (FFI dinâmico) e não implica endosso, suporte ou parceria. O repositório **não distribui** a DLL – apenas mostra como interoperar quando o usuário já possui direito legítimo de uso. Leia e respeite os termos de licença Nelógica.

Para o texto completo, consulte `DISCLAIMER.md`.

---

**Tucano** – Trading algorítmico moderno para o mercado brasileiro 🇧🇷  \
*MIT License* – ver [LICENSE](LICENSE) | [DISCLAIMER](DISCLAIMER.md)

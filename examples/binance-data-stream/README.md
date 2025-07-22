# Binance Data Stream TUI

Uma aplicação TUI (Terminal User Interface) para visualização em tempo real de dados de mercado da Binance usando o framework Toucan.

## 🎯 Funcionalidades

- **Order Book em Tempo Real**: Visualização de livro de ofertas (bids/asks) para futuros perpétuos de BTC
- **Stream de Trades**: Histórico de negociações recentes com estatísticas
- **Interface Intuitiva**: TUI responsiva usando ratatui-rs
- **Integração Toucan**: Utiliza as crates do framework Toucan para dados de mercado

## 🚀 Como Executar


### Pré-requisitos


- Rust 1.70+ instalado
- Terminal com suporte a cores


### Execução


```bash
# Na raiz do projeto Toucan
cargo run --bin binance-data-stream

# Ou diretamente na subcrate
cd examples/binance-data-stream
cargo run
```


### Controles


- **`q`**: Sair da aplicação
- **`r`**: Resetar dados
- **`Ctrl+C`**: Forçar saída

## 📊 Interface

A interface é dividida em quatro seções principais:


### 1. Header

- Nome do símbolo (BTCUSDT)
- Timestamp da última atualização


### 2. Estatísticas de Mercado

- Best Bid/Ask
- Spread atual
- Preço médio


### 3. Order Book

- **Lado Esquerdo**: Asks (ordens de venda) em vermelho
- **Lado Direito**: Bids (ordens de compra) em verde
- Colunas: Preço, Quantidade, Total


### 4. Trades Recentes

- Histórico das últimas negociações
- Volume e preço médio em 1 minuto
- Cores: Verde (compra), Vermelho (venda)

## 🏗️ Arquitetura

```
src/
├── main.rs              # Aplicação principal e setup do terminal
├── config.rs            # Configurações da aplicação
├── data/                # Estruturas de dados
│   ├── mod.rs
│   ├── orderbook.rs     # Estruturas do order book
│   └── trades.rs        # Estruturas de trades e histórico
└── ui/                  # Interface do usuário
    ├── mod.rs
    ├── app.rs           # Estado da aplicação
    ├── orderbook.rs     # Widget do order book
    └── trades.rs        # Widget de trades
```

## 🔧 Próximos Passos


### Fase 1: Dados Reais (TODO)

- [ ] Integrar com WebSocket da Binance usando toucan-data
- [ ] Implementar reconexão automática
- [ ] Processar mensagens reais do order book


### Fase 2: Funcionalidades Avançadas

- [ ] Suporte a múltiplos símbolos
- [ ] Gráficos de preço em tempo real
- [ ] Alertas de preço/volume
- [ ] Exportação de dados


### Fase 3: Configuração

- [ ] Arquivo de configuração JSON
- [ ] Personalização de cores/layout
- [ ] Configuração de símbolos via CLI

## 📦 Dependências


### Toucan Framework

- `data`: Streams de dados de mercado
- `integration`: Tipos e traits comuns
- `markets`: Abstrações de exchange


### TUI & Async

- `ratatui`: Framework para interface terminal
- `crossterm`: Controle de terminal multiplataforma
- `tokio`: Runtime assíncrono


### Utilitários

- `serde`: Serialização de dados
- `chrono`: Manipulação de tempo
- `anyhow`: Tratamento de erros

## 🎨 Exemplo Visual

```
┌─ Binance Data Stream - BTCUSDT | Last Update: 14:30:25.123 ───────┐
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
┌─ Market Data ──────────────────────────────────────────────────────┐
│        Best Bid: $45000.00 | Best Ask: $45010.00 | Spread: $10.00 │
└────────────────────────────────────────────────────────────────────┘
┌─ Asks (SELL) ──────────────┐┌─ Bids (BUY) ───────────────┐
│ Price    │ Quantity │ Total ││ Price    │ Quantity │ Total │
│ 45020.00 │ 0.150    │ 6753  ││ 45000.00 │ 0.150    │ 6750  │
│ 45015.00 │ 0.125    │ 5627  ││ 44995.00 │ 0.125    │ 5624  │
└────────────────────────────┘└────────────────────────────┘
┌─ Recent Trades ────────────────────────────────────────────────────┐
│ Time     │ Side │ Price    │ Quantity │ Total │
│ 14:30:25 │ BUY  │ 45005.00 │ 0.0250   │ 1125  │
│ 14:30:24 │ SELL │ 45003.00 │ 0.0100   │ 450   │
└────────────────────────────────────────────────────────────────────┘
┌─ Quit: q | Reset: r ───────────────────────────────────────────────┐
└────────────────────────────────────────────────────────────────────┘
```

## 🤝 Contribuição

Este é um exemplo educacional que demonstra:

1. **Integração com Toucan**: Como usar as crates do framework
2. **TUI com Ratatui**: Desenvolvimento de interfaces terminais
3. **Async Rust**: Programação assíncrona para streams de dados
4. **Arquitetura Modular**: Separação de responsabilidades

Melhorias e extensões são bem-vindas!

## 📝 Notas

- **Dados Mock**: Atualmente usa dados simulados para demonstração
- **Educational Purpose**: Focado em aprendizado e demonstração
- **Performance**: Otimizado para responsividade da interface
- **Extensibilidade**: Preparado para integração com dados reais

---

*Desenvolvido com ❤️ usando Toucan Framework e Ratatui*

# Binance Data Stream TUI

Uma aplicação de terminal (TUI) para visualizar dados de mercado da Binance em tempo real usando o framework Toucan.

## Recursos

- 📊 Visualização do order book BTCUSDT em tempo real
- 📈 Stream de trades com estatísticas
- 🖥️ Interface de terminal responsiva com ratatui
- 🔄 Integração com o framework Toucan (em progresso)
- ⚡ Dados simulados para demonstração

## Como usar

### Executar a aplicação

```bash
cargo run --bin binance-data-stream
```

### Controles

- `q` - Sair da aplicação
- `r` - Reset das estatísticas

## Status da implementação

### ✅ Completo

- Interface TUI básica com ratatui
- Widgets para order book e trades
- Estruturas de dados mock
- Sistema de canais para comunicação entre threads
- Layout responsivo

### 🔄 Em progresso - Integração Real com Toucan

- Streams de dados reais usando Toucan framework
- WebSocket conexões com Binance

### Abordagem de integração (Planejada)

A integração real com o framework Toucan será implementada da seguinte forma:

```rust
// Exemplo de integração real (planejada)
use data::{
    streams::Streams,
    subscription::{trade::PublicTrades, book::OrderBooksL1},
    exchange::binance::futures::BinanceFuturesUsd,
};

// Streams de trades reais
let mut trades_stream = Streams::<PublicTrades>::builder()
    .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades)])
    .init()
    .await?;

// Streams de order book reais
let mut book_stream = Streams::<OrderBooksL1>::builder()
    .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL1)])
    .init()
    .await?;
```

## Arquitetura

```text
src/
├── main.rs          # Aplicação principal e setup do terminal
├── ui/              # Widgets e interface
│   ├── app.rs       # Estado da aplicação
│   ├── orderbook.rs # Widget do order book
│   └── trades.rs    # Widget de trades
├── data/            # Estruturas de dados
│   ├── orderbook.rs # Dados do order book
│   └── trades.rs    # Dados de trades
└── config.rs        # Configuração
```

## Dependências

- **ratatui**: Framework TUI para interfaces de terminal
- **crossterm**: Controle de terminal multiplataforma
- **tokio**: Runtime assíncrono
- **data**: Crate do framework Toucan para market data
- **integration**: Crate do framework Toucan para protocolos
- **markets**: Crate do framework Toucan para instrumentos

## Desenvolvimento

### Adicionando novos widgets

1. Crie um novo arquivo em `src/ui/`
2. Implemente o widget usando ratatui
3. Adicione ao `src/ui/app.rs`
4. Atualize o layout principal

### Integrando dados reais

Para ativar a integração real com Toucan:

1. Descomente os imports do Toucan framework
2. Substitua as funções mock por streams reais
3. Configure credenciais se necessário
4. Teste a conexão WebSocket

## Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature
3. Commit suas mudanças
4. Push para a branch
5. Abra um Pull Request

## Licença

Este projeto faz parte do framework Toucan.

# Toucan Examples

Esta pasta contém exemplos práticos de como usar o ecossistema Toucan para diferentes cenários de trading.

## Estrutura

### 📁 **core/**
Exemplos do motor de trading principal:
- `backtests_concurrent.rs` - Backtests concorrentes
- `engine_async_with_historic_market_data_and_mock_execution.rs` - Motor assíncrono com dados históricos
- `engine_sync_with_audit_replica_engine_state.rs` - Motor síncrono com auditoria
- `engine_sync_with_live_market_data_and_mock_execution_and_audit.rs` - Motor síncrono com dados ao vivo
- `engine_sync_with_multiple_strategies.rs` - Motor com múltiplas estratégias
- `engine_sync_with_risk_manager_open_order_checks.rs` - Motor com gerenciamento de risco
- `statistical_trading_summary.rs` - Resumo estatístico de trading

### 📁 **data/**
Exemplos de streaming de dados de mercado:
- `dynamic_multi_stream_multi_exchange.rs` - Streams dinâmicos multi-exchange
- `indexed_market_stream.rs` - Streams indexados
- `multi_stream_multi_exchange.rs` - Múltiplos streams multi-exchange
- `order_books_l1_streams.rs` - Streams de order book L1
- `order_books_l1_streams_multi_exchange.rs` - Streams L1 multi-exchange
- `order_books_l2_manager.rs` - Gerenciamento de order book L2
- `order_books_l2_streams.rs` - Streams de order book L2
- `public_trades_streams.rs` - Streams de trades públicos
- `public_trades_streams_multi_exchange.rs` - Streams de trades multi-exchange

### 📁 **execution/**
Exemplos de execução de ordens:
- `binance_client_example.rs` - Cliente Binance para execução

### 📁 **integration/**
Exemplos de integração com exchanges:
- `signed_get_request.rs` - Requisições autenticadas
- `simple_websocket_integration.rs` - Integração WebSocket simples

### 📁 **complete/**
Exemplos completos e abrangentes:
- *Em desenvolvimento - exemplos end-to-end serão adicionados aqui*

### 📁 **assets/**
Arquivos de configuração e dados para os exemplos:
- `config/` - Arquivos de configuração JSON
- `data/` - Dados de mercado para teste

## Como Executar

### Executar um exemplo específico:
```bash
# Executar um exemplo do core
cargo run --bin backtests_concurrent

# Executar um exemplo de data
cargo run --bin public_trades_streams

# Executar um exemplo de execution
cargo run --bin binance_client_example
```

### Listar todos os exemplos disponíveis:
```bash
cargo run --bin
```

### Executar com logs detalhados:
```bash
RUST_LOG=debug cargo run --bin nome_do_exemplo
```

## Dependências

Todos os exemplos têm acesso às seguintes dependências:

### Toucan Ecosystem:
- `analytics` - Análise e métricas de trading
- `core` - Motor de trading principal
- `data` - Streaming de dados de mercado
- `execution` - Execução de ordens
- `integration` - Integração com exchanges
- `macros` - Macros utilitárias
- `markets` - Definições de mercado e instrumentos
- `risk` - Gerenciamento de risco
- `strategy` - Estratégias de trading

### Dependências Externas:
- `tokio` - Runtime assíncrono
- `tracing` - Logging estruturado
- `serde` - Serialização/deserialização
- `anyhow` - Tratamento de erros
- `chrono` - Manipulação de datas
- `uuid` - Geração de UUIDs
- `futures` - Programação assíncrona

## Contribuindo

Ao adicionar novos exemplos:

1. **Coloque o arquivo na pasta apropriada** (`core/`, `data/`, `execution/`, etc.)
2. **Adicione uma entrada `[[bin]]`** no `Cargo.toml`
3. **Atualize este README** com uma descrição do exemplo
4. **Use logs estruturados** com `tracing` para facilitar debug
5. **Adicione comentários explicativos** no código

## Próximos Passos

- [ ] Adicionar exemplos completos em `complete/`
- [ ] Exemplos de paper trading
- [ ] Exemplos de backtesting avançado
- [ ] Exemplos de estratégias específicas (RSI, Moving Average, etc.)
- [ ] Exemplos de trading multi-asset
- [ ] Exemplos de arbitragem

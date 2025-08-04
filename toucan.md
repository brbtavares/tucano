# Apresentação do Projeto

## Estrutura de Pastas

``` text
/workspaces/toucan
├── Cargo.toml
├── analytics
│   ├── Cargo.toml
│   └── src
│       ├── algorithm.rs
│       ├── lib.rs
│       ├── metric
│       │   ├── calmar.rs
│       │   ├── drawdown
│       │   │   ├── max.rs
│       │   │   ├── mean.rs
│       │   │   └── mod.rs
│       │   ├── mod.rs
│       │   ├── profit_factor.rs
│       │   ├── rate_of_return.rs
│       │   ├── sharpe.rs
│       │   ├── sortino.rs
│       │   └── win_rate.rs
│       ├── summary
│       │   ├── asset.rs
│       │   ├── dataset
│       │   │   ├── dispersion.rs
│       │   │   └── mod.rs
│       │   ├── display.rs
│       │   ├── instrument.rs
│       │   ├── instrument_backup.rs
│       │   ├── instrument_new.rs
│       │   ├── mod.rs
│       │   └── pnl.rs
│       └── time.rs
├── core
│   ├── Cargo.toml
│   ├── benches
│   │   └── backtest
│   │       └── mod.rs
│   ├── src
│   │   ├── backtest
│   │   │   ├── market_data.rs
│   │   │   ├── mod.rs
│   │   │   └── summary.rs
│   │   ├── engine
│   │   │   ├── action
│   │   │   │   ├── cancel_orders.rs
│   │   │   │   ├── close_positions.rs
│   │   │   │   ├── generate_algo_orders.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── send_requests.rs
│   │   │   ├── audit
│   │   │   │   ├── context.rs
│   │   │   │   ├── mod.rs
│   │   │   │   └── state_replica.rs
│   │   │   ├── clock.rs
│   │   │   ├── command.rs
│   │   │   ├── error.rs
│   │   │   ├── execution_tx.rs
│   │   │   ├── mod.rs
│   │   │   ├── run.rs
│   │   │   └── state
│   │   │       ├── asset
│   │   │       │   ├── filter.rs
│   │   │       │   └── mod.rs
│   │   │       ├── builder.rs
│   │   │       ├── connectivity
│   │   │       │   └── mod.rs
│   │   │       ├── global.rs
│   │   │       ├── instrument
│   │   │       │   ├── data.rs
│   │   │       │   ├── filter.rs
│   │   │       │   └── mod.rs
│   │   │       ├── mod.rs
│   │   │       ├── order
│   │   │       │   ├── in_flight_recorder.rs
│   │   │       │   ├── manager.rs
│   │   │       │   └── mod.rs
│   │   │       ├── position.rs
│   │   │       └── trading
│   │   │           └── mod.rs
│   │   ├── error.rs
│   │   ├── execution
│   │   │   ├── builder.rs
│   │   │   ├── error.rs
│   │   │   ├── manager.rs
│   │   │   ├── mod.rs
│   │   │   └── request.rs
│   │   ├── lib.rs
│   │   ├── logging.rs
│   │   ├── shutdown.rs
│   │   └── system
│   │       ├── builder.rs
│   │       ├── config.rs
│   │       └── mod.rs
│   └── tests
│       └── test_engine_process_engine_event_with_audit.rs
├── data
│   ├── Cargo.toml
│   └── src
│       ├── books
│       │   ├── manager.rs
│       │   ├── map.rs
│       │   └── mod.rs
│       ├── error.rs
│       ├── event.rs
│       ├── exchange
│       │   ├── binance
│       │   │   ├── book
│       │   │   │   ├── l1.rs
│       │   │   │   ├── l2.rs
│       │   │   │   └── mod.rs
│       │   │   ├── channel.rs
│       │   │   ├── futures
│       │   │   │   ├── l2.rs
│       │   │   │   ├── liquidation.rs
│       │   │   │   └── mod.rs
│       │   │   ├── market.rs
│       │   │   ├── mod.rs
│       │   │   ├── spot
│       │   │   │   ├── l2.rs
│       │   │   │   └── mod.rs
│       │   │   ├── subscription.rs
│       │   │   └── trade.rs
│       │   ├── bitfinex
│       │   │   ├── channel.rs
│       │   │   ├── market.rs
│       │   │   ├── message.rs
│       │   │   ├── mod.rs
│       │   │   ├── subscription.rs
│       │   │   ├── trade.rs
│       │   │   └── validator.rs
│       │   ├── bitmex
│       │   │   ├── channel.rs
│       │   │   ├── market.rs
│       │   │   ├── message.rs
│       │   │   ├── mod.rs
│       │   │   ├── subscription.rs
│       │   │   └── trade.rs
│       │   ├── bybit
│       │   │   ├── book
│       │   │   │   ├── l1.rs
│       │   │   │   ├── l2.rs
│       │   │   │   └── mod.rs
│       │   │   ├── channel.rs
│       │   │   ├── futures
│       │   │   │   └── mod.rs
│       │   │   ├── market.rs
│       │   │   ├── message.rs
│       │   │   ├── mod.rs
│       │   │   ├── spot
│       │   │   │   └── mod.rs
│       │   │   ├── subscription.rs
│       │   │   └── trade.rs
│       │   ├── coinbase
│       │   │   ├── channel.rs
│       │   │   ├── market.rs
│       │   │   ├── mod.rs
│       │   │   ├── subscription.rs
│       │   │   └── trade.rs
│       │   ├── gateio
│       │   │   ├── channel.rs
│       │   │   ├── future
│       │   │   │   └── mod.rs
│       │   │   ├── market.rs
│       │   │   ├── message.rs
│       │   │   ├── mod.rs
│       │   │   ├── option
│       │   │   │   └── mod.rs
│       │   │   ├── perpetual
│       │   │   │   ├── mod.rs
│       │   │   │   └── trade.rs
│       │   │   ├── spot
│       │   │   │   ├── mod.rs
│       │   │   │   └── trade.rs
│       │   │   └── subscription.rs
│       │   ├── kraken
│       │   │   ├── book
│       │   │   │   ├── l1.rs
│       │   │   │   └── mod.rs
│       │   │   ├── channel.rs
│       │   │   ├── market.rs
│       │   │   ├── message.rs
│       │   │   ├── mod.rs
│       │   │   ├── subscription.rs
│       │   │   └── trade.rs
│       │   ├── mod.rs
│       │   ├── okx
│       │   │   ├── channel.rs
│       │   │   ├── market.rs
│       │   │   ├── mod.rs
│       │   │   ├── subscription.rs
│       │   │   └── trade.rs
│       │   └── subscription.rs
│       ├── instrument.rs
│       ├── lib.rs
│       ├── streams
│       │   ├── builder
│       │   │   ├── dynamic
│       │   │   │   ├── indexed.rs
│       │   │   │   └── mod.rs
│       │   │   ├── mod.rs
│       │   │   └── multi.rs
│       │   ├── consumer.rs
│       │   ├── mod.rs
│       │   └── reconnect
│       │       ├── mod.rs
│       │       └── stream.rs
│       ├── subscriber
│       │   ├── mapper.rs
│       │   ├── mod.rs
│       │   └── validator.rs
│       ├── subscription
│       │   ├── book.rs
│       │   ├── candle.rs
│       │   ├── liquidation.rs
│       │   ├── mod.rs
│       │   └── trade.rs
│       └── transformer
│           ├── mod.rs
│           └── stateless.rs
├── examples
│   ├── Cargo.toml
│   ├── config
│   │   ├── backtest_config.json
│   │   └── system_config.json
│   ├── data
│   │   ├── binance_spot_market_data_with_disconnect_events.json
│   │   └── binance_spot_trades_l1_btcusdt_ethusdt_solusdt.json
│   └── src
│       ├── bin
│       │   ├── binance_api_authenticated_request.rs
│       │   ├── binance_btc_realtime_statistics.rs
│       │   ├── binance_client_example.rs
│       │   ├── binance_orderbook_level1_streaming.rs
│       │   ├── binance_orderbook_level2_streaming.rs
│       │   ├── binance_public_trades_streaming.rs
│       │   ├── binance_websocket_basic_integration.rs
│       │   ├── dynamic_multi_stream_multi_exchange.rs
│       │   ├── engine_sync_with_audit_replica_engine_state.rs
│       │   ├── indexed_market_stream.rs
│       │   ├── multi_exchange_synchronized_streaming.rs
│       │   ├── order_books_l1_streams_multi_exchange.rs
│       │   ├── order_books_l2_manager.rs
│       │   ├── public_trades_streams_multi_exchange.rs
│       │   ├── trading_backtesting_concurrent_strategies.rs
│       │   ├── trading_engine_historical_data_simulation.rs
│       │   ├── trading_engine_live_data_with_audit.rs
│       │   ├── trading_engine_multiple_strategies.rs
│       │   ├── trading_engine_paper_trading_simulation.rs
│       │   └── trading_engine_risk_management.rs
│       ├── credentials.rs
│       └── lib.rs
├── execution
│   ├── Cargo.toml
│   └── src
│       ├── balance.rs
│       ├── client
│       │   ├── binance
│       │   │   ├── mod.rs
│       │   │   ├── model.rs
│       │   │   ├── request.rs
│       │   │   ├── response.rs
│       │   │   ├── tests.rs
│       │   │   └── websocket.rs
│       │   ├── mock
│       │   │   └── mod.rs
│       │   └── mod.rs
│       ├── error.rs
│       ├── exchange
│       │   ├── mock
│       │   │   ├── account.rs
│       │   │   ├── mod.rs
│       │   │   └── request.rs
│       │   └── mod.rs
│       ├── indexer.rs
│       ├── lib.rs
│       ├── map.rs
│       ├── order
│       │   ├── id.rs
│       │   ├── mod.rs
│       │   ├── request.rs
│       │   └── state.rs
│       └── trade.rs
├── integration
│   ├── Cargo.toml
│   └── src
│       ├── channel.rs
│       ├── collection
│       │   ├── mod.rs
│       │   ├── none_one_or_many.rs
│       │   └── one_or_many.rs
│       ├── de.rs
│       ├── error.rs
│       ├── lib.rs
│       ├── metric.rs
│       ├── protocol
│       │   ├── http
│       │   │   ├── mod.rs
│       │   │   ├── private
│       │   │   │   ├── encoder.rs
│       │   │   │   └── mod.rs
│       │   │   ├── public
│       │   │   │   └── mod.rs
│       │   │   └── rest
│       │   │       ├── client.rs
│       │   │       └── mod.rs
│       │   ├── mod.rs
│       │   └── websocket.rs
│       ├── snapshot.rs
│       ├── stream
│       │   ├── indexed.rs
│       │   ├── merge.rs
│       │   └── mod.rs
│       └── subscription.rs
├── macros
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── markets
│   ├── Cargo.toml
│   └── src
│       ├── asset
│       │   ├── mod.rs
│       │   └── name.rs
│       ├── exchange.rs
│       ├── index
│       │   ├── builder.rs
│       │   ├── error.rs
│       │   └── mod.rs
│       ├── instrument
│       │   ├── kind
│       │   │   ├── future.rs
│       │   │   ├── mod.rs
│       │   │   ├── option.rs
│       │   │   └── perpetual.rs
│       │   ├── market_data
│       │   │   ├── kind.rs
│       │   │   └── mod.rs
│       │   ├── mod.rs
│       │   ├── name.rs
│       │   ├── quote.rs
│       │   └── spec.rs
│       └── lib.rs
├── risk
│   ├── Cargo.toml
│   └── src
│       ├── check
│       │   ├── mod.rs
│       │   └── util.rs
│       └── lib.rs
├── rustfmt.toml
├── strategy
│   ├── Cargo.toml
│   └── src
│       ├── algo.rs
│       ├── close_positions.rs
│       ├── default.rs
│       ├── lib.rs
│       ├── mod.rs
│       ├── on_disconnect.rs
│       └── on_trading_disabled.rs
└── toucan.md
```



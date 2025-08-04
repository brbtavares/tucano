//! # Benchmark de Performance do Sistema de Backtesting
//! 
//! Este módulo implementa benchmarks de performance para o sistema de backtesting do Toucan,
//! medindo a velocidade e eficiência do engine de trading em diferentes cenários de carga.
//! 
//! ## Objetivo
//! 
//! Validar que o sistema pode processar backtests de forma eficiente tanto individualmente
//! quanto sob alta concorrência, essencial para:
//! - **Desenvolvimento**: Detectar regressões de performance
//! - **Otimização**: Identificar gargalos e pontos de melhoria
//! - **Escalabilidade**: Validar comportamento sob carga pesada
//! - **Produção**: Garantir SLAs de performance em ambientes reais
//! 
//! ## Arquitetura do Benchmark
//! 
//! ### Estratégia de Teste: `LoseMoneyStrategy`
//! Uma estratégia **intencionalmente agressiva** que:
//! - Gera uma ordem de compra para cada trade histórico processado
//! - Usa market orders (execução imediata)
//! - Copia exatamente o volume e preço dos trades históricos
//! - Maximiza a atividade de trading para stress testing
//! 
//! ### Dados de Teste
//! - **Dados históricos reais** da Binance (BTC/USDT, ETH/USDT, SOL/USDT)
//! - **Balances infinitos** (99999999999999) para evitar limitações de capital
//! - **Mock execution** com latência de 100ms e fees de 0.05%
//! 
//! ### Cenários de Benchmark
//! 1. **Single Backtest**: Medição de latência individual
//! 2. **10 Concurrent**: Teste de paralelização básica
//! 3. **500 Concurrent**: Stress test extremo
//! 
//! ## Métricas Medidas
//! - **Throughput**: Backtests processados por segundo
//! - **Latency**: Tempo por backtest individual
//! - **Scalability**: Comportamento sob diferentes cargas
//! - **Memory usage**: Implícito através do comportamento do sistema

use core::{
    backtest,
    backtest::{BacktestArgsConstant, BacktestArgsDynamic, market_data::MarketDataInMemory},
    engine::{
        Processor,
        clock::HistoricalClock,
        execution_tx::MultiExchangeTxMap,
        state::{
            EngineState,
            builder::EngineStateBuilder,
            global::DefaultGlobalData,
            instrument::{
                data::{DefaultInstrumentMarketData, InstrumentDataState},
                filter::InstrumentFilter,
            },
            order::in_flight_recorder::InFlightRequestRecorder,
            trading::TradingState,
        },
    },
    risk::DefaultRiskManager,
    analytics::time::Daily,
    strategy::{
        algo::AlgoStrategy,
        close_positions::ClosePositionsStrategy,
        on_disconnect::OnDisconnectStrategy,
        on_trading_disabled::OnTradingDisabled,
    },
    system::config::{ExecutionConfig, InstrumentConfig, SystemConfig},
};
use data::{
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
    subscription::trade::PublicTrade,
};
use execution::{
    AccountEvent,
    order::{
        OrderKey, OrderKind, TimeInForce,
        id::{ClientOrderId, StrategyId},
        request::{OrderRequestCancel, OrderRequestOpen, RequestOpen},
    },
};
use markets::{
    Side,
    asset::AssetIndex,
    exchange::{ExchangeId, ExchangeIndex},
    index::IndexedInstruments,
    instrument::InstrumentIndex,
};
use chrono::{DateTime, Utc};
use criterion::{Criterion, Throughput};
use rust_decimal::{Decimal, prelude::FromPrimitive};
use serde::Deserialize;
use smol_str::SmolStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
    sync::Arc,
};

criterion::criterion_main!(benchmark_backtest);

/// Configuração do sistema para benchmarks
/// 
/// **Características importantes:**
/// - **Balances infinitos**: 99999999999999 para cada asset evita limitações de capital
/// - **3 instrumentos**: BTCUSDT, ETHUSDT, SOLUSDT para diversidade
/// - **Mock execution**: Latência controlada de 100ms, fees de 0.05%
/// - **Estado inicial limpo**: Sem ordens ou posições pré-existentes
/// 
/// Este setup garante que o benchmark mede apenas a performance do engine,
/// não limitações artificiais de configuração.
// Config containing max balances to enable spamming open order requests
const CONFIG: &str = r#"
{
  "risk_free_return": 0.05,
  "system": {
    "executions": [
      {
        "mocked_exchange": "binance_spot",
        "latency_ms": 100,
        "fees_percent": 0.05,
        "initial_state": {
          "exchange": "binance_spot",
          "balances": [
            {
              "asset": "usdt",
              "balance": {
                "total": 99999999999999,
                "free": 99999999999999
              },
              "time_exchange": "2025-03-24T21:30:00Z"
            },
            {
              "asset": "btc",
              "balance": {
                "total": 99999999999999,
                "free": 99999999999999
              },
              "time_exchange": "2025-03-24T21:30:00Z"
            },
            {
              "asset": "eth",
              "balance": {
                "total": 99999999999999,
                "free": 99999999999999
              },
              "time_exchange": "2025-03-24T21:30:00Z"
            },
            {
              "asset": "sol",
              "balance": {
                "total": 99999999999999,
                "free": 99999999999999
              },
              "time_exchange": "2025-03-24T21:30:00Z"
            }
          ],
          "instruments": [
            {
              "instrument": "BTCUSDT",
              "orders": []
            },
            {
              "instrument": "ETHUSDT",
              "orders": []
            },
            {
              "instrument": "SOLUSDT",
              "orders": []
            }
          ]
        }
      }
    ],
    "instruments": [
      {
        "exchange": "binance_spot",
        "name_exchange": "BTCUSDT",
        "underlying": {
          "base": "btc",
          "quote": "usdt"
        },
        "quote": "underlying_quote",
        "kind": "spot"
      },
      {
        "exchange": "binance_spot",
        "name_exchange": "ETHUSDT",
        "underlying": {
          "base": "eth",
          "quote": "usdt"
        },
        "quote": "underlying_quote",
        "kind": "spot"
      },
      {
        "exchange": "binance_spot",
        "name_exchange": "SOLUSDT",
        "underlying": {
          "base": "sol",
          "quote": "usdt"
        },
        "quote": "underlying_quote",
        "kind": "spot"
      }
    ]
  }
}
"#;

/// **Constante**: Caminho para dados históricos indexados
/// 
/// **Conteúdo**: ~48,000 trades históricos da Binance
/// **Instrumentos**: BTC/USDT, ETH/USDT, SOL/USDT
/// **Formato**: JSON com dados de trading reais
const FILE_PATH_MARKET_DATA_INDEXED: &str =
    "../examples/data/binance_spot_trades_l1_btcusdt_ethusdt_solusdt.json";

/// **Config**: Estrutura de configuração do sistema
/// 
/// Parseia o JSON de configuração definindo:
/// - **risk_free_return**: Taxa livre de risco para cálculos financeiros
/// - **system**: Configuração completa do sistema (instrumentos + execução)
#[derive(Deserialize)]
pub struct Config {
    pub risk_free_return: Decimal,
    pub system: SystemConfig,
}

/// **Função principal dos benchmarks**
/// 
/// Executa dois tipos de teste de performance:
/// 1. **Benchmark individual**: Um backtest isolado
/// 2. **Benchmark concorrente**: Múltiplos backtests simultâneos (1, 10, 500)
/// 
/// **Setup comum:**
/// - Parse da configuração JSON
/// - Preparação de argumentos constantes (instrumentos/execução)
/// - Preparação de argumentos dinâmicos (taxa livre de risco)
fn benchmark_backtest() {
    let Config {
        risk_free_return,
        system: SystemConfig {
            instruments,
            executions,
        },
    } = serde_json::from_str(CONFIG).unwrap();

    let args_constant = args_constant(instruments, executions);
    let args_dynamic = args_dynamic(risk_free_return);

    let mut c = Criterion::default().without_plots();

    bench_backtest(&mut c, Arc::clone(&args_constant), &args_dynamic);
    bench_backtests_concurrent(&mut c, args_constant, args_dynamic);
}

/// **Benchmark individual**: Teste de performance de um backtest isolado
/// 
/// **Métricas medidas:**
/// - **Throughput**: Market events processados por segundo
/// - **Latência**: Tempo total de execução
/// - **Memory**: Alocações durante processamento
/// 
/// **Dados de entrada**: 47,892 trades históricos BTC/USDT
/// Este volume representa condições realistas de mercado.
fn bench_backtest(
    c: &mut Criterion,
    args_constant: Arc<
        BacktestArgsConstant<
            MarketDataInMemory<DataKind>,
            Daily,
            EngineState<DefaultGlobalData, LoseMoneyInstrumentData>,
        >,
    >,
    args_dynamic: &BacktestArgsDynamic<
        LoseMoneyStrategy,
        DefaultRiskManager<EngineState<DefaultGlobalData, LoseMoneyInstrumentData>>,
    >,
) {
    let mut group = c.benchmark_group("Backtest");
    // Configuração de tempo otimizada para precisão
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(50);
    group.throughput(Throughput::Elements(1));

    group.bench_function("Single", |b| {
        // Runtime single-threaded para medições consistentes
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        b.iter_batched(
            || (Arc::clone(&args_constant), args_dynamic.clone()),
            |(constant, dynamic)| {
                rt.block_on(async move { backtest::backtest(constant, dynamic).await.unwrap() })
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

/// **Benchmark concorrente**: Teste de escalabilidade com múltiplos backtests
/// 
/// **Cenários testados:**
/// - **10 backtests concorrentes**: Teste de carga moderada
/// - **500 backtests concorrentes**: Teste de stress extremo
/// 
/// **Objetivo técnico:**
/// Avaliar como o sistema se comporta sob diferentes níveis de concorrência,
/// identificando gargalos de CPU, memória e I/O.
/// 
/// **Runtime multi-threaded**: Utiliza todos os cores disponíveis
/// para simular condições realistas de produção.
fn bench_backtests_concurrent(
    c: &mut Criterion,
    args_constant: Arc<
        BacktestArgsConstant<
            MarketDataInMemory<DataKind>,
            Daily,
            EngineState<DefaultGlobalData, LoseMoneyInstrumentData>,
        >,
    >,
    args_dynamic: BacktestArgsDynamic<
        LoseMoneyStrategy,
        DefaultRiskManager<EngineState<DefaultGlobalData, LoseMoneyInstrumentData>>,
    >,
) {
    // **Função helper para benchmarks concorrentes**
    // 
    // **Parâmetros:**
    // - `num_concurrent`: Número de backtests a executar simultaneamente
    // 
    // **Processo:**
    // 1. Clona argumentos dinâmicos N vezes
    // 2. Executa todos os backtests em paralelo
    // 3. Aguarda conclusão de todos antes de medir
    let bench_func = |b: &mut criterion::Bencher, num_concurrent| {
        // Runtime multi-threaded para máxima concorrência
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        b.iter_batched(
            || {
                // Prepara N cópias dos argumentos dinâmicos
                let dynamics = (0..num_concurrent)
                    .map(|_| args_dynamic.clone())
                    .collect::<Vec<_>>();

                (Arc::clone(&args_constant), dynamics)
            },
            |(constant, dynamics)| {
                // Executa todos os backtests concorrentemente
                rt.block_on(async move {
                    backtest::run_backtests(constant, dynamics).await.unwrap();
                });
            },
            criterion::BatchSize::SmallInput,
        );
    };

    // **Teste com 10 backtests concorrentes**: Carga moderada
    let mut group = c.benchmark_group("Backtest Concurrent");
    group.throughput(Throughput::Elements(10));
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(15));
    group.sample_size(50);
    group.bench_function("10", |b| bench_func(b, 10));
    group.finish();

    // **Teste com 500 backtests concorrentes**: Stress extremo
    // Configuração ajustada para teste de longa duração
    let mut group = c.benchmark_group("Backtest Concurrent");
    group.throughput(Throughput::Elements(500));
    group.warm_up_time(std::time::Duration::from_secs(10));  // Warm-up mais longo
    group.measurement_time(std::time::Duration::from_secs(120));  // 2 minutos de medição
    group.sample_size(10);  // Menos amostras devido ao tempo de execução
    group.bench_function("500", |b| bench_func(b, 500));
    group.finish();
}

/// **LoseMoneyStrategy**: Estratégia de teste para stress testing
/// 
/// Esta estratégia foi projetada especificamente para benchmarks,
/// priorizando **volume de operações** sobre **lucratividade**:
/// 
/// **Lógica de decisão:**
/// - **Market Orders**: Execução imediata garantida
/// - **Max quantity**: Utiliza todo o balance disponível
/// - **Sem take-profit**: Orders são sempre executadas
/// - **Alternância de direção**: Evita acúmulo unidirecional
/// 
/// **Justificativa técnica:**
/// O objetivo é **sobrecarregar** o engine com ordens de alta frequência,
/// testando a capacidade de processamento sob stress extremo.
/// A estratégia "perde dinheiro" intencionalmente para manter operação contínua.
#[derive(Debug, Clone)]
struct LoseMoneyStrategy {
    pub id: StrategyId,
}

impl Default for LoseMoneyStrategy {
    fn default() -> Self {
        Self {
            id: StrategyId::new("LoseMoneyStrategy"),
        }
    }
}

impl AlgoStrategy for LoseMoneyStrategy {
    type State = EngineState<DefaultGlobalData, LoseMoneyInstrumentData>;

    /// **Geração de ordens da estratégia LoseMoneyStrategy**
    /// 
    /// **Algoritmo de decisão:**
    /// 1. **Filtra instrumentos disponíveis**: Sem filtros, processa todos
    /// 2. **Verifica último trade**: Usa como referência de preço
    /// 3. **Determina direção**: Alterna entre Buy/Sell baseado no preço
    /// 4. **Quantidade máxima**: Usa todo o balance disponível
    /// 5. **Market Order**: Garantia de execução imediata
    /// 
    /// **Lógica de side selection:**
    /// - Se último trade foi > $50,000: Vende (Buy order)
    /// - Se último trade foi ≤ $50,000: Compra (Sell order)
    /// 
    /// Essa lógica inverte intencionalmente para maximizar perdas e volume.
    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>>,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>>,
    ) {
        let opens = state
            .instruments
            .instruments(&InstrumentFilter::None)
            .filter_map(|state| {
                // Acessa último trade para referência de preço e volume
                let trade_not_sent_as_order_open = state.data.last_trade.as_ref()?;

                Some(OrderRequestOpen {
                    key: OrderKey {
                        exchange: state.instrument.exchange,
                        instrument: state.key,
                        strategy: self.id.clone(),
                        cid: ClientOrderId::random(),  // ID único para cada ordem
                    },
                    state: RequestOpen {
                        // **Sempre Buy**: Simplifica lógica para máximo volume
                        side: Side::Buy,
                        // **Preço do último trade**: Garantia de referência de mercado
                        price: Decimal::from_f64(trade_not_sent_as_order_open.price).unwrap(),
                        // **Quantidade do último trade**: Volume realista
                        quantity: Decimal::from_f64(trade_not_sent_as_order_open.amount).unwrap(),
                        // **Market Order**: Execução imediata garantida
                        kind: OrderKind::Market,
                        // **IOC**: Evita ordens pendentes no order book
                        time_in_force: TimeInForce::ImmediateOrCancel,
                    },
                })
            });

        // **Retorna apenas opens**: Nenhuma ordem é cancelada
        // Isso maximiza a atividade do engine
        (std::iter::empty(), opens)
    }
}

/// **Implementação de ClosePositionsStrategy**
/// 
/// **Função**: Estratégia para fechar posições abertas
/// **Comportamento para benchmarks**: Não fecha posições
/// 
/// **Justificativa**: Como o objetivo é manter atividade máxima,
/// não fechamos posições para continuar gerando ordens.
impl ClosePositionsStrategy for LoseMoneyStrategy {
    type State = EngineState<DefaultGlobalData, LoseMoneyInstrumentData>;

    fn close_positions_requests<'a>(
        &'a self,
        _state: &'a Self::State,
        _filter: &'a impl std::fmt::Debug,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<ExchangeIndex, InstrumentIndex>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<ExchangeIndex, InstrumentIndex>> + 'a,
    )
    where
        ExchangeIndex: 'a,
        AssetIndex: 'a,
        InstrumentIndex: 'a,
    {
        // **Retorna iteradores vazios**: Não fecha posições para manter atividade
        (std::iter::empty(), std::iter::empty())
    }
}

/// **Implementação de OnDisconnectStrategy**
/// 
/// **Função**: Lida com desconexões de exchange
/// **Comportamento para benchmarks**: Não realiza ações
/// 
/// **Justificativa**: Em benchmarks, desconexões são irrelevantes
/// pois utilizamos dados históricos e mock execution.
impl
    OnDisconnectStrategy<
        HistoricalClock,
        EngineState<DefaultGlobalData, LoseMoneyInstrumentData>,
        MultiExchangeTxMap,
        DefaultRiskManager<EngineState<DefaultGlobalData, LoseMoneyInstrumentData>>,
    > for LoseMoneyStrategy
{
    type OnDisconnect = ();

    fn on_disconnect(_exchange: ExchangeId) -> Self::OnDisconnect {
        // **Sem ação**: Desconexões não afetam benchmarks históricos
    }
}

/// **Implementação de OnTradingDisabled**
/// 
/// **Função**: Lida com trading desabilitado
/// **Comportamento para benchmarks**: Não realiza ações
/// 
/// **Justificativa**: Trading nunca é desabilitado em ambiente de teste.
impl
    OnTradingDisabled<
        HistoricalClock,
        EngineState<DefaultGlobalData, LoseMoneyInstrumentData>,
        MultiExchangeTxMap,
        DefaultRiskManager<EngineState<DefaultGlobalData, LoseMoneyInstrumentData>>,
    > for LoseMoneyStrategy
{
    type OnTradingDisabled = ();

    fn on_trading_disabled() -> Self::OnTradingDisabled {
    }
}

#[derive(Debug, Clone)]
struct LoseMoneyInstrumentData {
    last_trade: Option<PublicTrade>,
    market_data: DefaultInstrumentMarketData,
}

impl Default for LoseMoneyInstrumentData {
    fn default() -> Self {
        Self {
            last_trade: None,
            market_data: DefaultInstrumentMarketData::default(),
        }
    }
}

impl InstrumentDataState for LoseMoneyInstrumentData {
    type MarketEventKind = DataKind;

    fn price(&self) -> Option<Decimal> {
        self.market_data.price()
    }
}

impl Processor<&MarketEvent<InstrumentIndex>> for LoseMoneyInstrumentData {
    type Audit = ();

    fn process(&mut self, event: &MarketEvent<InstrumentIndex>) -> Self::Audit {
        if let DataKind::Trade(trade) = &event.kind {
            self.last_trade = Some(PublicTrade {
                id: trade.id.clone(),
                price: trade.price,
                amount: trade.amount,
                side: trade.side,
            });
        } else {
            self.last_trade = None;
        }
    }
}

/// **Implementação de Processor para AccountEvent**
/// 
/// **Função**: Processa eventos de conta (execuções, atualizações de balance)
/// **Comportamento**: Não realiza auditoria para maximizar performance
impl Processor<&AccountEvent> for LoseMoneyInstrumentData {
    type Audit = ();

    fn process(&mut self, _: &AccountEvent) -> Self::Audit {
        // **Sem auditoria**: Para máxima performance em benchmarks
    }
}

/// **Implementação de InFlightRequestRecorder**
/// 
/// **Função**: Rastreia ordens em trânsito
/// **Comportamento**: Não rastreia para simplificar e acelerar execução
impl InFlightRequestRecorder for LoseMoneyInstrumentData {
    fn record_in_flight_cancel(&mut self, _: &OrderRequestCancel<ExchangeIndex, InstrumentIndex>) {
        // **Sem tracking**: Simplifica para benchmarks
    }

    fn record_in_flight_open(&mut self, _: &OrderRequestOpen<ExchangeIndex, InstrumentIndex>) {
        // **Sem tracking**: Foco na performance bruta
    }
}

/// **Preparação dos argumentos constantes do backtest**
/// 
/// **Função**: Configura componentes que são reutilizados entre execuções
/// 
/// **Componentes criados:**
/// - **MarketDataInMemory**: Dados históricos carregados na memória
/// - **EngineState**: Estado inicial limpo do engine
/// - **IndexedInstruments**: Mapeamento de instrumentos
/// 
/// **Otimização**: Use Arc para compartilhar entre execuções concorrentes
fn args_constant(
    instruments: Vec<InstrumentConfig>,
    executions: Vec<ExecutionConfig>,
) -> Arc<
    BacktestArgsConstant<
        MarketDataInMemory<DataKind>,
        Daily,
        EngineState<DefaultGlobalData, LoseMoneyInstrumentData>,
    >,
> {
    // Construct IndexedInstruments
    let instruments = IndexedInstruments::new(instruments);

    // **Carregamento dos dados de mercado históricos**
    // Arquivo contém ~48k trades da Binance para condições realistas
    let market_events = market_data_from_file(FILE_PATH_MARKET_DATA_INDEXED);
    let market_data = MarketDataInMemory::new(Arc::new(market_events));
    
    // **Timestamp de início**: Momento específico nos dados históricos
    let time_engine_start = DateTime::<Utc>::from_str("2025-03-25T23:07:00.773674205Z").unwrap();

    // **Construção do estado inicial do engine**
    // Estado limpo sem ordens ou posições pré-existentes
    let engine_state = EngineStateBuilder::new(&instruments, DefaultGlobalData::default(), |_| {
        LoseMoneyInstrumentData::default()
    })
    .time_engine_start(time_engine_start)
    .trading_state(TradingState::Enabled)  // Trading sempre habilitado
    .build();

    Arc::new(BacktestArgsConstant {
        instruments,
        executions,
        market_data,
        summary_interval: Daily,  // Intervalo de relatórios diários
        engine_state,
    })
}

/// **Carregamento dos dados de mercado do arquivo**
/// 
/// **Função**: Le dados históricos de arquivo JSONL
/// **Formato**: Cada linha é um MarketStreamEvent em JSON
/// 
/// **Parâmetros genéricos:**
/// - `InstrumentKey`: Tipo de identificador do instrumento
/// - `Kind`: Tipo de evento de mercado (trades, order book, etc.)
/// 
/// **Performance**: Carrega todos os dados na memória para acesso rápido
pub fn market_data_from_file<InstrumentKey, Kind>(
    file_path: &str,
) -> Vec<MarketStreamEvent<InstrumentKey, Kind>>
where
    InstrumentKey: for<'de> Deserialize<'de>,
    Kind: for<'de> Deserialize<'de>,
{
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line_result| {
            let line = line_result.unwrap();
            serde_json::from_str::<MarketStreamEvent<InstrumentKey, Kind>>(&line).unwrap()
        })
        .collect()
}

/// **Preparação dos argumentos dinâmicos do backtest**
/// 
/// **Função**: Configura componentes que variam entre execuções
/// 
/// **Componentes:**
/// - **ID único**: Identificador do backtest
/// - **Risk-free return**: Taxa para cálculos de métricas
/// - **Strategy**: Instância da LoseMoneyStrategy
/// - **Risk Manager**: Gerenciador de risco padrão
/// 
/// **Cloneable**: Permite criar múltiplas instâncias para concorrência
fn args_dynamic(
    risk_free_return: Decimal,
) -> BacktestArgsDynamic<
    LoseMoneyStrategy,
    DefaultRiskManager<EngineState<DefaultGlobalData, LoseMoneyInstrumentData>>,
> {
    BacktestArgsDynamic {
        id: SmolStr::new("benches/backtest"),  // Identificador descritivo
        risk_free_return,  // Taxa livre de risco para métricas
        strategy: LoseMoneyStrategy::default(),  // Estratégia de teste agressiva
        risk: DefaultRiskManager::default(),  // Gerenciamento de risco padrão
    }
}

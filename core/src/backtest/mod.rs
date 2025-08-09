//! # Backtesting Framework
//!
//! This module provides a comprehensive backtesting framework for algorithmic trading strategies.
//! It enables historical simulation of trading strategies using market data and provides detailed
//! performance analysis and reporting capabilities.
//!
//! ## Key Features
//!
//! ### Historical Simulation
//! - **Market Data Replay**: Process historical market data chronologically
//! - **Strategy Execution**: Run trading strategies against historical data
//! - **Order Simulation**: Simulate order execution without real market impact
//! - **Multi-timeframe Support**: Backtest across different time intervals
//!
//! ### Performance Analysis
//! - **Comprehensive Metrics**: Calculate Sharpe ratio, Sortino ratio, max drawdown, etc.
//! - **Risk Analytics**: Analyze risk-adjusted returns and portfolio volatility
//! - **Trade Analysis**: Detailed breakdown of individual trade performance
//! - **Timeline Analysis**: Performance evolution over time
//!
//! ### Multi-Strategy Testing
//! - **Parallel Execution**: Run multiple strategy variations simultaneously
//! - **Comparative Analysis**: Compare performance across different strategies
//! - **Parameter Optimization**: Test strategy parameter combinations
//! - **Portfolio Simulation**: Multi-asset and multi-strategy portfolios
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                 BACKTEST FRAMEWORK                      │
//! ├─────────────────┬─────────────────┬───────────────────────┤
//! │ Historical Data │ Strategy Engine │ Performance Analytics│
//! │                 │                 │                       │
//! │ • Market Replay │ • Order Sim     │ • Metrics Calculation │
//! │ • Time Control  │ • Risk Mgmt     │ • Reporting           │
//! │ • Data Loading  │ • State Track   │ • Visualization       │
//! └─────────────────┴─────────────────┴───────────────────────┘
//!                              │
//! ┌─────────────────────────────┼─────────────────────────────┐
//! │                   DATA SOURCES                          │
//! ├─────────────────────────────┼─────────────────────────────┤
//! │ Historical Candles          │    Trade & Quote Data       │
//! │ Order Book Snapshots        │    Corporate Actions        │
//! └─────────────────────────────┴─────────────────────────────┘
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! use core::backtest::{BacktestSummary, market_data::BacktestMarketData};
//! use analytics::time::TimeInterval;
//!
//! // Setup backtest configuration
//! let market_data = BacktestMarketData::load_from_csv("historical_data.csv").await?;
//! let strategy = MyTradingStrategy::new();
//!
//! // Run backtest
//! let summary = run_backtest(
//!     strategy,
//!     market_data,
//!     TimeInterval::days(365), // 1 year backtest
//! ).await?;
//!
//! // Analyze results
//! println!("Sharpe Ratio: {:.2}", summary.sharpe_ratio());
//! println!("Max Drawdown: {:.2}%", summary.max_drawdown() * 100.0);
//! println!("Total Return: {:.2}%", summary.total_return() * 100.0);
//! ```
/// Backtesting utilities for algorithmic trading strategies.
///
/// This module provides tools for running historical simulations of trading strategies
/// using market data, and analyzing the performance of these simulations.
use crate::{
    backtest::{
        market_data::BacktestMarketData,
        summary::{BacktestSummary, MultiBacktestSummary},
    },
    engine::{
        clock::HistoricalClock,
        execution_tx::MultiExchangeTxMap,
        state::{instrument::data::InstrumentDataState, EngineState},
        Processor,
    },
    error::ToucanError,
    risk::RiskManager,
    system::{builder::EngineFeedMode, config::ExecutionConfig},
};
use crate::{
    engine::Engine,
    execution::builder::{ExecutionBuild, ExecutionBuilder},
    system::builder::{AuditMode, SystemBuild},
};
use analytics::time::TimeInterval;
use data::event::MarketEvent;
use execution::{AccountEvent, InstrumentIndex};
use futures::future::try_join_all;
use rust_decimal::Decimal;
use smol_str::SmolStr;
use std::{fmt::Debug, sync::Arc};
use strategy::{AlgoStrategy, ClosePositionsStrategy, OnDisconnectStrategy, OnTradingDisabled};

/// Placeholder for IndexedInstruments
pub type IndexedInstruments = Vec<()>;

/// Defines the interface and implementations for different types of market data sources
/// that can be used in backtests.
pub mod market_data;

/// Contains data structures for representing backtest results and metrics.
pub mod summary;

/// Configuration for constants used across all backtests in a batch.
///
/// Contains shared inputs like instruments, execution configurations,
/// market data, and summary time intervals.
#[derive(Debug, Clone)]
pub struct BacktestArgsConstant<MarketData, SummaryInterval, State> {
    /// Set of trading instruments indexed by unique identifiers.
    pub instruments: IndexedInstruments,
    /// Exchange execution configurations.
    pub executions: Vec<ExecutionConfig>,
    /// Historical market data to use for simulation.
    pub market_data: MarketData,
    /// Time interval for aggregating and reporting summary statistics.
    pub summary_interval: SummaryInterval,
    /// EngineState.
    pub engine_state: State,
}

/// Configuration for variables that can change between individual backtests.
///
/// Contains parameters that define a specific strategy variant to test.
#[derive(Debug, Clone)]
pub struct BacktestArgsDynamic<Strategy, Risk> {
    /// Unique identifier for this backtest.
    pub id: SmolStr,
    /// Risk-free return rate used for performance metrics.
    pub risk_free_return: Decimal,
    /// Trading strategy to backtest.
    pub strategy: Strategy,
    /// Risk management rules.
    pub risk: Risk,
}
/// Run multiple backtests concurrently, each with different strategy parameters.
///
/// Takes the shared constants and an iterator of different strategy configurations,
/// then executes all backtests in parallel, collecting the results.
pub async fn run_backtests<
    MarketData,
    SummaryInterval,
    Strategy,
    Risk,
    GlobalData,
    InstrumentData,
>(
    args_constant: Arc<
        BacktestArgsConstant<MarketData, SummaryInterval, EngineState<GlobalData, InstrumentData>>,
    >,
    args_dynamic_iter: impl IntoIterator<Item = BacktestArgsDynamic<Strategy, Risk>>,
) -> Result<MultiBacktestSummary<SummaryInterval>, ToucanError>
where
    MarketData: BacktestMarketData<Kind = InstrumentData::MarketEventKind>,
    SummaryInterval: TimeInterval,
    Strategy: AlgoStrategy<State = EngineState<GlobalData, InstrumentData>>
        + ClosePositionsStrategy<State = EngineState<GlobalData, InstrumentData>>
        + OnTradingDisabled<
            HistoricalClock,
            EngineState<GlobalData, InstrumentData>,
            MultiExchangeTxMap,
            Risk,
        > + OnDisconnectStrategy<
            HistoricalClock,
            EngineState<GlobalData, InstrumentData>,
            MultiExchangeTxMap,
            Risk,
        > + Send
        + 'static,
    <Strategy as OnTradingDisabled<
        HistoricalClock,
        EngineState<GlobalData, InstrumentData>,
        MultiExchangeTxMap,
        Risk,
    >>::OnTradingDisabled: Debug + Clone + Send,
    <Strategy as OnDisconnectStrategy<
        HistoricalClock,
        EngineState<GlobalData, InstrumentData>,
        MultiExchangeTxMap,
        Risk,
    >>::OnDisconnect: Debug + Clone + Send,
    Risk: RiskManager<State = EngineState<GlobalData, InstrumentData>> + Send + 'static,
    GlobalData: for<'a> Processor<&'a MarketEvent<InstrumentIndex, InstrumentData::MarketEventKind>>
        + for<'a> Processor<&'a AccountEvent>
        + Debug
        + Clone
        + Default
        + Send
        + 'static,
    InstrumentData: InstrumentDataState + Default + Send + 'static,
{
    let time_start = std::time::Instant::now();

    let backtest_futures = args_dynamic_iter
        .into_iter()
        .map(|args_dynamic| backtest(Arc::clone(&args_constant), args_dynamic));

    // Run all backtests concurrently
    let summaries = try_join_all(backtest_futures).await?;

    Ok(MultiBacktestSummary::new(
        std::time::Instant::now().duration_since(time_start),
        summaries,
    ))
}

/// Run a single backtest with the given parameters.
///
/// Simulates a trading strategy using historical market data and generates performance metrics.
pub async fn backtest<MarketData, SummaryInterval, Strategy, Risk, GlobalData, InstrumentData>(
    args_constant: Arc<
        BacktestArgsConstant<MarketData, SummaryInterval, EngineState<GlobalData, InstrumentData>>,
    >,
    args_dynamic: BacktestArgsDynamic<Strategy, Risk>,
) -> Result<BacktestSummary<SummaryInterval>, ToucanError>
where
    MarketData: BacktestMarketData<Kind = InstrumentData::MarketEventKind>,
    SummaryInterval: TimeInterval,
    Strategy: AlgoStrategy<State = EngineState<GlobalData, InstrumentData>>
        + ClosePositionsStrategy<State = EngineState<GlobalData, InstrumentData>>
        + OnTradingDisabled<
            HistoricalClock,
            EngineState<GlobalData, InstrumentData>,
            MultiExchangeTxMap,
            Risk,
        > + OnDisconnectStrategy<
            HistoricalClock,
            EngineState<GlobalData, InstrumentData>,
            MultiExchangeTxMap,
            Risk,
        > + Send
        + 'static,
    <Strategy as OnTradingDisabled<
        HistoricalClock,
        EngineState<GlobalData, InstrumentData>,
        MultiExchangeTxMap,
        Risk,
    >>::OnTradingDisabled: Debug + Clone + Send,
    <Strategy as OnDisconnectStrategy<
        HistoricalClock,
        EngineState<GlobalData, InstrumentData>,
        MultiExchangeTxMap,
        Risk,
    >>::OnDisconnect: Debug + Clone + Send,
    Risk: RiskManager<State = EngineState<GlobalData, InstrumentData>> + Send + 'static,
    GlobalData: for<'a> Processor<&'a MarketEvent<InstrumentIndex, InstrumentData::MarketEventKind>>
        + for<'a> Processor<&'a AccountEvent>
        + Debug
        + Clone
        + Default
        + Send
        + 'static,
    InstrumentData: InstrumentDataState + Send + 'static,
{
    let clock = args_constant
        .market_data
        .time_first_event()
        .await
        .map(HistoricalClock::new)?;
    let market_stream = args_constant.market_data.stream().await?;

    // Build Execution infrastructure
    let ExecutionBuild {
        execution_tx_map,
        account_channel,
        futures,
    } = args_constant
        .executions
        .clone()
        .into_iter()
        .try_fold(
            ExecutionBuilder::new(&args_constant.instruments),
            |builder, config| match config {
                ExecutionConfig::Mock(mock_config) => builder.add_mock(mock_config, clock.clone()),
            },
        )?
        .build();

    let engine = Engine::new(
        clock,
        args_constant.engine_state.clone(),
        execution_tx_map,
        args_dynamic.strategy,
        args_dynamic.risk,
    );

    let system = SystemBuild::new(
        engine,
        EngineFeedMode::Stream,
        AuditMode::Disabled,
        market_stream,
        account_channel,
        futures,
    )
    .init()
    .await?;

    let (engine, _shutdown_audit) = system.shutdown_after_backtest().await?;

    let trading_summary = engine
        .trading_summary_generator(args_dynamic.risk_free_return)
        .generate(args_constant.summary_interval);

    Ok(BacktestSummary {
        id: args_dynamic.id,
        risk_free_return: args_dynamic.risk_free_return,
        trading_summary,
    })
}

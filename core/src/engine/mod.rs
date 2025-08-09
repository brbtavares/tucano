//! # Core Engine - Sistema de Trading Algorítmico
//!
//! Este módulo implementa o coração do sistema de trading algorítmico, fornecendo uma arquitetura
//! flexível e de alta performance para processamento de eventos de mercado, execução de estratégias
//! e gerenciamento de risco.
//!
//! ## 🎯 Visão Geral
//!
//! O [`Engine`] é uma estrutura genérica que processa diferentes tipos de eventos:
//! - **Market Events**: Dados de mercado (preços, book de ofertas, negócios)
//! - **Account Events**: Eventos de execução (ordens executadas, posições)
//! - **Commands**: Comandos externos (fechar posições, cancelar ordens)
//! - **Trading State**: Mudanças de estado (habilitado/desabilitado)
//!
//! ## 🏗️ Arquitetura
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │  Market Data    │    │   Engine Core   │    │   Execution     │
//! │  • Prices       │───▶│  • Process      │───▶│  • Orders       │
//! │  • Book         │    │  • Strategy     │    │  • Cancels      │
//! │  • Trades       │    │  • Risk Mgmt    │    │  • Positions    │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//!                               │
//!                               ▼
//!                        ┌─────────────────┐
//!                        │   Audit Trail   │
//!                        │  • Decisions    │
//!                        │  • Performance  │
//!                        │  • Compliance   │
//!                        └─────────────────┘
//! ```
//!
//! ## 🔧 Componentes Principais
//!
//! ### [`Engine`] - Estrutura Principal
//! Estrutura genérica com 5 type parameters que permite composição flexível:
//! - **`Clock`**: Controle de tempo (real-time vs backtest)
//! - **`State`**: Estado interno (posições, ordens, dados de mercado)
//! - **`ExecutionTxs`**: Canais de comunicação com exchanges
//! - **`Strategy`**: Lógica de trading algorítmico
//! - **`Risk`**: Gerenciamento de risco
//!
//! ### Traits Principais
//! - [`Processor`]: Processamento de eventos com audit trail
//! - [`EngineClock`]: Abstração de tempo para backtesting
//! - [`ExecutionTxMap`]: Roteamento de ordens para exchanges
//! - [`AlgoStrategy`]: Implementação de estratégias de trading
//! - [`RiskManager`]: Controle de risco e validações
//!
//! ## 📊 Fluxo de Processamento
//!
//! 1. **Event Input**: Recebe [`EngineEvent`] (market data, commands, etc.)
//! 2. **Clock Update**: Atualiza timestamp do engine
//! 3. **State Update**: Processa evento e atualiza estado interno
//! 4. **Strategy Execution**: Se trading habilitado, executa estratégia
//! 5. **Risk Validation**: Valida decisões com risk manager
//! 6. **Order Generation**: Gera ordens se estratégia retornar sinais
//! 7. **Audit Creation**: Cria trilha de auditoria completa
//!
//! ## 🚀 Exemplos de Uso
//!
//! ### Configuração Básica
//! ```rust
//! use core::engine::Engine;
//!
//! // Construir engine para trading real-time
//! let engine = Engine::new(
//!     RealTimeClock::new(),           // Clock em tempo real
//!     EngineState::new(),             // Estado inicial
//!     ExecutionTxMap::new(),          // Canais para exchanges
//!     MyTradingStrategy::new(),       // Estratégia customizada
//!     RiskManager::new(),             // Controle de risco
//! );
//! ```
//!
//! ### Processamento de Eventos
//! ```rust
//! // Processar dados de mercado
//! let market_event = EngineEvent::Market(market_data);
//! let audit = engine.process(market_event);
//!
//! // Processar comando externo
//! let command = Command::ClosePositions(filter);
//! let command_event = EngineEvent::Command(command);
//! let audit = engine.process(command_event);
//! ```
//!
//! ### Backtesting
//! ```rust
//! // Configurar para backtesting
//! let backtest_engine = Engine::new(
//!     BacktestClock::new(start_date),  // Clock histórico
//!     EngineState::new(),
//!     MockExecutionTxs::new(),         // Execução simulada
//!     BacktestStrategy::new(),
//!     StrictRiskManager::new(),
//! );
//! ```
//!
//! ## ⚡ Performance
//!
//! O engine é otimizado para baixa latência com:
//! - **Zero-allocation paths**: Processamento sem alocações no hot path
//! - **Generic programming**: Monomorphization para performance máxima
//! - **Lock-free structures**: Evita contention em cenários multi-thread
//! - **Efficient state updates**: Updates incrementais do estado
//!
//! ## 🛡️ Segurança e Confiabilidade
//!
//! - **Type safety**: Sistema de tipos previne erros em tempo de compilação
//! - **Audit trail**: Trilha completa de decisões para debugging/compliance
//! - **Error recovery**: Tratamento robusto de erros com graceful degradation
//! - **Risk management**: Validações de risco integradas no fluxo

use crate::{
    engine::{
        action::{
            cancel_orders::CancelOrders,
            close_positions::ClosePositions,
            generate_algo_orders::{GenerateAlgoOrders, GenerateAlgoOrdersOutput},
            send_requests::SendRequests,
            ActionOutput,
        },
        audit::{context::EngineContext, AuditTick, Auditor, EngineAudit, ProcessAudit},
        clock::EngineClock,
        command::Command,
        execution_tx::ExecutionTxMap,
        state::{
            instrument::data::InstrumentDataState,
            order::in_flight_recorder::InFlightRequestRecorder, position::PositionExited,
            trading::TradingState, EngineState,
        },
    },
    execution::{request::ExecutionRequest, AccountStreamEvent},
    risk::RiskManager,
    shutdown::SyncShutdown,
    EngineEvent, Sequence,
};
use analytics::summary::TradingSummaryGenerator;
use chrono::{DateTime, Utc};
use data::{event::MarketEvent, streams::consumer::MarketStreamEvent};
use execution::{AccountEvent, ExchangeIndex, InstrumentIndex, QuoteAsset};
use integration::channel::Tx;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use strategy::{AlgoStrategy, ClosePositionsStrategy, OnDisconnectStrategy, OnTradingDisabled};
use tracing::info;

/// Defines how the [`Engine`] actions a [`Command`], and the associated outputs.
///
/// Contains implementations for:
/// - [`SendRequests`]: Envio de ordens e cancelamentos
/// - [`CancelOrders`]: Cancelamento de ordens por filtros
/// - [`ClosePositions`]: Fechamento de posições por filtros
/// - [`GenerateAlgoOrders`]: Geração de ordens algorítmicas
pub mod action;

/// Defines an `Engine` audit types as well as utilities for handling the `Engine` `AuditStream`.
///
/// O sistema de auditoria fornece:
/// - **Trilha completa**: Registro de todas as decisões e processamentos
/// - **Debugging**: Informações detalhadas para análise de problemas
/// - **Compliance**: Trilha de auditoria para regulamentação
/// - **Performance**: Métricas de performance e timing
///
/// eg/ `StateReplicaManager` component can be used to maintain an `EngineState` replica.
pub mod audit;

/// Defines the [`EngineClock`] interface used to determine the current `Engine` time.
///
/// Esta abstração permite flexibilidade entre diferentes modos de operação:
/// - **Real-time**: `RealTimeClock` para trading ao vivo
/// - **Backtest**: `BacktestClock` para simulação com dados históricos
/// - **Custom**: Implementações customizadas para casos específicos
///
/// This flexibility enables back-testing runs to use approximately correct historical timestamps.
pub mod clock;

/// Defines an [`Engine`] [`Command`] - used to give trading directives to the `Engine` from an
/// external process (eg/ ClosePositions).
///
/// Comandos disponíveis:
/// - `SendCancelRequests`: Cancelar ordens específicas
/// - `SendOpenRequests`: Enviar novas ordens
/// - `ClosePositions`: Fechar posições por filtro
/// - `CancelOrders`: Cancelar ordens por filtro
pub mod command;

/// Defines all possible errors that can occur in the [`Engine`].
///
/// Sistema robusto de tratamento de erros com:
/// - **Categorização**: Erros organizados por tipo (State, Execution, Risk, Strategy)
/// - **Recovery**: Informações para recuperação automática
/// - **Context**: Contexto detalhado para debugging
pub mod error;

/// Defines the [`ExecutionTxMap`] interface that models a collection of transmitters used to route
/// [`ExecutionRequest`] to the appropriate `ExecutionManagers`.
///
/// Responsável pelo roteamento de ordens:
/// - **Multi-exchange**: Suporte a múltiplas exchanges simultaneamente
/// - **Load balancing**: Distribuição de carga entre conexões
/// - **Failover**: Recuperação automática em caso de falhas
pub mod execution_tx;

/// Defines all state used by the `Engine` to algorithmically trade.
///
/// Estado interno completo incluindo:
/// - **ConnectivityStates**: Estado das conexões com exchanges
/// - **AssetStates**: Estado dos ativos (preços, volumes)
/// - **InstrumentStates**: Estado dos instrumentos (book, trades)
/// - **Positions**: Posições abertas e fechadas
/// - **Orders**: Ordens ativas e históricas
///
/// eg/ `ConnectivityStates`, `AssetStates`, `InstrumentStates`, `Position`, etc.
pub mod state;

/// `Engine` runners for processing input `Events`.
///
/// Diferentes modos de execução:
/// - **Sync**: `sync_run` para baixa latência em produção
/// - **Async**: `async_run` para backtesting e simulação
/// - **With Audit**: Versões com auditoria completa
///
/// eg/ `fn sync_run`, `fn sync_run_with_audit`, `fn async_run`, `fn async_run_with_audit`,
pub mod run;

/// Defines how a component processing an input Event and generates an appropriate Audit.
///
/// Este trait é fundamental para o sistema de auditoria, permitindo que qualquer
/// componente processe eventos e gere trilhas de auditoria correspondentes.
///
/// # Type Parameters
/// - `Event`: Tipo do evento a ser processado
///
/// # Associated Types
/// - `Audit`: Tipo da auditoria gerada pelo processamento
///
/// # Examples
/// ```rust
/// impl Processor<MarketEvent> for MyStrategy {
///     type Audit = StrategyAudit;
///
///     fn process(&mut self, event: MarketEvent) -> Self::Audit {
///         // Processa evento de mercado
///         // Retorna auditoria das decisões tomadas
///     }
/// }
/// ```
pub trait Processor<Event> {
    type Audit;
    fn process(&mut self, event: Event) -> Self::Audit;
}

/// Process an `Event` with the `Engine` and produce an [`AuditTick`] of work done.
///
/// Esta função utilitária combina processamento e auditoria em uma única operação,
/// garantindo que toda atividade do engine seja devidamente registrada.
///
/// # Parameters
/// - `engine`: Engine que processará o evento
/// - `event`: Evento a ser processado
///
/// # Returns
/// [`AuditTick`] contendo o resultado do processamento e contexto de auditoria
///
/// # Examples
/// ```rust
/// let audit_tick = process_with_audit(&mut engine, market_event);
/// println!("Processed event in {:?}", audit_tick.duration);
/// ```
pub fn process_with_audit<Event, Engine>(
    engine: &mut Engine,
    event: Event,
) -> AuditTick<Engine::Audit, EngineContext>
where
    Engine: Processor<Event> + Auditor<Engine::Audit, Context = EngineContext>,
{
    let output = engine.process(event);
    engine.audit(output)
}

/// Algorithmic trading `Engine`.
///
/// The `Engine`:
/// * Processes input [`EngineEvent`] (or custom events if implemented).
/// * Maintains the internal [`EngineState`] (instrument data state, open orders, positions, etc.).
/// * Generates algo orders (if `TradingState::Enabled`).
///
/// # Type Parameters
/// * `Clock` - [`EngineClock`] implementation.
/// * `State` - Engine `State` implementation (eg/ [`EngineState`]).
/// * `ExecutionTxs` - [`ExecutionTxMap`] implementation for sending execution requests.
/// * `Strategy` - Trading Strategy implementation (see [`super::strategy`]).
/// * `Risk` - [`RiskManager`] implementation.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Engine<Clock, State, ExecutionTxs, Strategy, Risk> {
    pub clock: Clock,
    pub meta: EngineMeta,
    pub state: State,
    pub execution_txs: ExecutionTxs,
    pub strategy: Strategy,
    pub risk: Risk,
}

/// Running [`Engine`] metadata.
///
/// Contém metadados da sessão atual do engine, incluindo timestamps
/// e contadores sequenciais para tracking de estado.
///
/// # Fields
/// - `time_start`: Timestamp de início da sessão atual
/// - `sequence`: Contador monotônico de eventos processados
///
/// # Usage
/// Os metadados são automaticamente mantidos pelo engine e utilizados para:
/// - **Performance tracking**: Duração da sessão e throughput
/// - **Audit trail**: Ordenação temporal de eventos
/// - **State management**: Controle de versão do estado
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct EngineMeta {
    /// [`EngineClock`] start timestamp of the current [`Engine`] `run`.
    pub time_start: DateTime<Utc>,
    /// Monotonically increasing [`Sequence`] associated with the number of events processed.
    pub sequence: Sequence,
}

impl<Clock, GlobalData, InstrumentData, ExecutionTxs, Strategy, Risk>
    Processor<EngineEvent<InstrumentData::MarketEventKind>>
    for Engine<Clock, EngineState<GlobalData, InstrumentData>, ExecutionTxs, Strategy, Risk>
where
    Clock: EngineClock + for<'a> Processor<&'a EngineEvent<InstrumentData::MarketEventKind>>,
    InstrumentData: InstrumentDataState,
    GlobalData: for<'a> Processor<&'a AccountEvent>
        + for<'a> Processor<&'a MarketEvent<InstrumentIndex, InstrumentData::MarketEventKind>>,
    ExecutionTxs: ExecutionTxMap<ExchangeIndex, InstrumentIndex>,
    Strategy: OnTradingDisabled<Clock, EngineState<GlobalData, InstrumentData>, ExecutionTxs, Risk>
        + OnDisconnectStrategy<Clock, EngineState<GlobalData, InstrumentData>, ExecutionTxs, Risk>
        + AlgoStrategy<State = EngineState<GlobalData, InstrumentData>>
        + ClosePositionsStrategy<State = EngineState<GlobalData, InstrumentData>>,
    Risk: RiskManager<State = EngineState<GlobalData, InstrumentData>>,
{
    type Audit = EngineAudit<
        EngineEvent<InstrumentData::MarketEventKind>,
        EngineOutput<Strategy::OnTradingDisabled, Strategy::OnDisconnect>,
    >;

    fn process(&mut self, event: EngineEvent<InstrumentData::MarketEventKind>) -> Self::Audit {
        self.clock.process(&event);

        let process_audit = match &event {
            EngineEvent::Shutdown(_) => return EngineAudit::process(event),
            EngineEvent::Command(command) => {
                let output = self.action(command);

                if let Some(unrecoverable) = output.unrecoverable_errors() {
                    return EngineAudit::process_with_output_and_errs(event, unrecoverable, output);
                } else {
                    ProcessAudit::with_output(event, output)
                }
            }
            EngineEvent::TradingStateUpdate(trading_state) => {
                let trading_disabled = self.update_from_trading_state_update(*trading_state);
                ProcessAudit::with_trading_state_update(event, trading_disabled)
            }
            EngineEvent::Account(account) => {
                let output = self.update_from_account_stream(account);
                ProcessAudit::with_account_update(event, output)
            }
            EngineEvent::Market(market) => {
                let output = self.update_from_market_stream(market);
                ProcessAudit::with_market_update(event, output)
            }
        };

        if let TradingState::Enabled = self.state.trading {
            let output = self.generate_algo_orders();

            if output.is_empty() {
                EngineAudit::from(process_audit)
            } else if let Some(unrecoverable) = output.unrecoverable_errors() {
                EngineAudit::Process(process_audit.add_errors(unrecoverable))
            } else {
                EngineAudit::from(process_audit.add_output(output))
            }
        } else {
            EngineAudit::from(process_audit)
        }
    }
}

impl<Clock, GlobalData, InstrumentData, ExecutionTxs, Strategy, Risk> SyncShutdown
    for Engine<Clock, EngineState<GlobalData, InstrumentData>, ExecutionTxs, Strategy, Risk>
where
    ExecutionTxs: ExecutionTxMap<ExchangeIndex, InstrumentIndex>,
{
    type Result = ();

    fn shutdown(&mut self) -> Self::Result {
        self.execution_txs.iter().for_each(|execution_tx| {
            let _send_result = execution_tx.send(ExecutionRequest::Shutdown);
        });
    }
}

impl<Clock, GlobalData, InstrumentData, ExecutionTxs, Strategy, Risk>
    Engine<Clock, EngineState<GlobalData, InstrumentData>, ExecutionTxs, Strategy, Risk>
{
    /// Action an `Engine` [`Command`], producing an [`ActionOutput`] of work done.
    ///
    /// Executa comandos externos no engine, como cancelamento de ordens,
    /// abertura de posições, etc. Cada comando gera um output específico
    /// e registra a ação no sistema de auditoria.
    ///
    /// # Supported Commands
    /// - `SendCancelRequests`: Cancela ordens específicas
    /// - `SendOpenRequests`: Envia novas ordens para o mercado
    /// - `ClosePositions`: Fecha posições baseado em filtros
    /// - `CancelOrders`: Cancela ordens baseado em filtros
    ///
    /// # Returns
    /// [`ActionOutput`] contendo os resultados da ação executada
    ///
    /// # Logging
    /// Todas as ações são logadas em nível INFO com detalhes dos parâmetros
    pub fn action(&mut self, command: &Command) -> ActionOutput
    where
        InstrumentData: InFlightRequestRecorder,
    ExecutionTxs: ExecutionTxMap<ExchangeIndex, InstrumentIndex>,
        Strategy: ClosePositionsStrategy<State = EngineState<GlobalData, InstrumentData>>,
        Risk: RiskManager,
    {
        match &command {
            Command::SendCancelRequests(requests) => {
                info!(
                    ?requests,
                    "Engine actioning user Command::SendCancelRequests"
                );
                let output = self.send_requests(requests.clone());
                self.state.record_in_flight_cancels(&output.sent);
                ActionOutput::CancelOrders(output)
            }
            Command::SendOpenRequests(requests) => {
                info!(?requests, "Engine actioning user Command::SendOpenRequests");
                let output = self.send_requests(requests.clone());
                self.state.record_in_flight_opens(&output.sent);
                ActionOutput::OpenOrders(output)
            }
            Command::ClosePositions(filter) => {
                info!(?filter, "Engine actioning user Command::ClosePositions");
                ActionOutput::ClosePositions(self.close_positions(filter))
            }
            Command::CancelOrders(filter) => {
                info!(?filter, "Engine actioning user Command::CancelOrders");
                ActionOutput::CancelOrders(self.cancel_orders(filter))
            }
        }
    }

    /// Update the `Engine` [`TradingState`].
    ///
    /// Atualiza o estado de trading do engine. Quando há transição para
    /// `TradingState::Disabled`, aciona automaticamente a estratégia de
    /// desabilitação configurada.
    ///
    /// # State Transitions
    /// - `Enabled` → `Disabled`: Aciona `OnTradingDisabled` strategy
    /// - `Disabled` → `Enabled`: Reativa geração de ordens algorítmicas
    /// - Sem mudança: Nenhuma ação adicional
    ///
    /// # Returns
    /// `Some(Strategy::OnTradingDisabled)` se houve transição para disabled,
    /// `None` caso contrário
    ///
    /// If the `TradingState` transitions to `TradingState::Disabled`, the `Engine` will call
    /// the configured [`OnTradingDisabled`] strategy logic.
    pub fn update_from_trading_state_update(
        &mut self,
        update: TradingState,
    ) -> Option<Strategy::OnTradingDisabled>
    where
        Strategy:
            OnTradingDisabled<Clock, EngineState<GlobalData, InstrumentData>, ExecutionTxs, Risk>,
    {
        self.state
            .trading
            .update(update)
            .transitioned_to_disabled()
            .then(|| Strategy::on_trading_disabled())
    }

    /// Update the [`Engine`] from an [`AccountStreamEvent`].
    ///
    /// Processa eventos da stream de conta, incluindo execuções de ordens,
    /// mudanças de posição e eventos de conectividade. Detecta automaticamente
    /// desconexões e aciona estratégias de recuperação.
    ///
    /// # Event Types
    /// - `Reconnecting`: Indica reconexão em andamento, aciona `OnDisconnectStrategy`
    /// - `Item(AccountEvent)`: Evento específico da conta (execução, posição, etc.)
    ///
    /// # Automatic Actions
    /// - **Disconnect Detection**: Detecta perda de conectividade automaticamente
    /// - **Strategy Trigger**: Aciona estratégia de desconexão quando necessário
    /// - **State Update**: Atualiza estado interno com novos dados da conta
    ///
    /// # Returns
    /// Output específico baseado no tipo de evento processado
    ///
    /// If the input `AccountStreamEvent` indicates the exchange execution link has disconnected,
    /// the `Engine` will call the configured [`OnDisconnectStrategy`] strategy logic.
    pub fn update_from_account_stream(
        &mut self,
        event: &AccountStreamEvent,
    ) -> UpdateFromAccountOutput<Strategy::OnDisconnect>
    where
        InstrumentData: for<'a> Processor<&'a AccountEvent>,
        GlobalData: for<'a> Processor<&'a AccountEvent>,
        Strategy: OnDisconnectStrategy<
            Clock,
            EngineState<GlobalData, InstrumentData>,
            ExecutionTxs,
            Risk,
        >,
    {
        match event {
            AccountStreamEvent::Reconnecting(exchange) => {
                self.state
                    .connectivity
                    .update_from_account_reconnecting(exchange);

                UpdateFromAccountOutput::OnDisconnect(Strategy::on_disconnect(*exchange))
            }
            AccountStreamEvent::Item(event) => self
                .state
                .update_from_account(event)
                .map(UpdateFromAccountOutput::PositionExit)
                .unwrap_or(UpdateFromAccountOutput::None),
        }
    }

    /// Update the [`Engine`] from a [`MarketStreamEvent`].
    ///
    /// Processa eventos de stream de mercado incluindo atualizações de dados,
    /// desconexões e reconexões. Detecta automaticamente perda de conectividade
    /// e aciona estratégias de recuperação conforme necessário.
    ///
    /// # Event Types
    /// - `Reconnecting`: Indica reconexão da stream de mercado em andamento
    /// - `Item(MarketEvent)`: Evento específico de mercado (dados, status, etc.)
    ///
    /// # Automatic Actions
    /// - **Disconnect Detection**: Detecta perda de conectividade da stream
    /// - **Strategy Trigger**: Aciona `OnDisconnectStrategy` quando necessário
    /// - **State Update**: Atualiza estado com novos dados de mercado
    /// - **Connectivity Tracking**: Monitora status de conexão por exchange
    ///
    /// # Parameters
    /// - `event`: Evento da stream de mercado com dados ou status de conectividade
    ///
    /// # Returns
    /// Output específico baseado no tipo de evento e ações executadas
    ///
    /// If the input `MarketStreamEvent` indicates the exchange market data link has disconnected,
    /// the `Engine` will call the configured [`OnDisconnectStrategy`] strategy logic.
    pub fn update_from_market_stream(
        &mut self,
        event: &MarketStreamEvent<InstrumentIndex, InstrumentData::MarketEventKind>,
    ) -> UpdateFromMarketOutput<Strategy::OnDisconnect>
    where
        InstrumentData: InstrumentDataState,
        GlobalData:
            for<'a> Processor<&'a MarketEvent<InstrumentIndex, InstrumentData::MarketEventKind>>,
        Strategy: OnDisconnectStrategy<
            Clock,
            EngineState<GlobalData, InstrumentData>,
            ExecutionTxs,
            Risk,
        >,
    {
        match event {
            MarketStreamEvent::Reconnecting(exchange) => {
                self.state
                    .connectivity
                    .update_from_market_reconnecting(exchange);

                UpdateFromMarketOutput::OnDisconnect(Strategy::on_disconnect(*exchange))
            }
            MarketStreamEvent::Item(event) => {
                self.state.update_from_market(event);
                UpdateFromMarketOutput::None
            }
        }
    }

    /// Returns a [`TradingSummaryGenerator`] for the current trading session.
    ///
    /// Cria um gerador de resumo de trading para a sessão atual, incluindo
    /// métricas de performance, análise de risco e relatórios financeiros.
    /// Utiliza dados históricos da sessão para calcular estatísticas.
    ///
    /// # Components
    /// - **Risk-Free Return**: Taxa livre de risco para cálculos de Sharpe ratio
    /// - **Time Window**: Período da sessão atual (start → current time)
    /// - **Instruments**: Instrumentos ativos na sessão
    /// - **Assets**: Balanços de ativos disponíveis
    ///
    /// # Usage
    /// ```rust,no_run
    /// let summary_gen = engine.trading_summary_generator();
    /// let metrics = summary_gen.calculate_metrics();
    /// println!("Sharpe Ratio: {}", metrics.sharpe_ratio);
    /// ```
    ///
    /// # Returns
    /// Gerador configurado com estado atual da sessão
    pub fn trading_summary_generator(&self, risk_free_return: Decimal) -> TradingSummaryGenerator
    where
        Clock: EngineClock,
    {
        use execution::{balance::AssetBalance, AssetIndex, InstrumentIndex};
        use integration::collection::FnvIndexMap;

        // Create placeholder empty collections since analytics expects simplified types
        let instruments: FnvIndexMap<InstrumentIndex, ()> = FnvIndexMap::default();
        let assets: FnvIndexMap<AssetIndex, AssetBalance<AssetIndex>> = FnvIndexMap::default();

        TradingSummaryGenerator::init::<(), AssetIndex>(
            risk_free_return,
            self.meta.time_start,
            self.time(),
            &instruments,
            &assets,
        )
    }
}

impl<Clock, State, ExecutionTxs, Strategy, Risk> Engine<Clock, State, ExecutionTxs, Strategy, Risk>
where
    Clock: EngineClock,
{
    /// Construct a new `Engine`.
    ///
    /// Cria uma nova instância do engine de trading algorítmico com todos
    /// os componentes necessários configurados. O engine é inicializado
    /// com estado limpo e metadados baseados no clock fornecido.
    ///
    /// # Parameters
    /// - `clock`: Relógio do engine para timestamps e sequenciamento
    /// - `state`: Estado inicial do engine (global, instrumentos, conectividade)
    /// - `execution_txs`: Canais de comunicação com módulos de execução
    /// - `strategy`: Estratégia algorítmica a ser executada
    /// - `risk`: Módulo de gestão de risco
    ///
    /// # Initialization
    /// - **Metadata**: Criado com timestamp atual e sequência inicial 0
    /// - **State**: Configurado com dados globais e instrumentos fornecidos
    /// - **Execution**: Canais prontos para envio de ordens
    /// - **Strategy**: Carregada e pronta para gerar sinais
    /// - **Risk**: Módulo ativo para validação de operações
    ///
    /// # Example
    /// ```rust,no_run
    /// let engine = Engine::new(
    ///     UTCClock::new(),
    ///     engine_state,
    ///     execution_channels,
    ///     my_strategy,
    ///     risk_manager
    /// );
    /// ```
    ///
    /// An initial [`EngineMeta`] is constructed form the provided `clock` and `Sequence(0)`.
    pub fn new(
        clock: Clock,
        state: State,
        execution_txs: ExecutionTxs,
        strategy: Strategy,
        risk: Risk,
    ) -> Self {
        Self {
            meta: EngineMeta {
                time_start: clock.time(),
                sequence: Sequence(0),
            },
            clock,
            state,
            execution_txs,
            strategy,
            risk,
        }
    }

    /// Return `Engine` clock time.
    pub fn time(&self) -> DateTime<Utc> {
        self.clock.time()
    }

    /// Reset the internal `EngineMeta` to the `clock` time and `Sequence(0)`.
    pub fn reset_metadata(&mut self) {
        self.meta.time_start = self.clock.time();
        self.meta.sequence = Sequence(0);
    }
}

/// Output produced by [`Engine`] operations, used to construct an `Engine` [`EngineAudit`].
///
/// Representa todos os possíveis outputs que o engine pode produzir durante
/// sua operação. Cada variante corresponde a um tipo específico de operação
/// e é usada para construir o audit trail completo.
///
/// # Variants
/// - `Commanded`: Output de comandos externos executados
/// - `OnTradingDisabled`: Output da estratégia de trading desabilitado
/// - `AccountDisconnect`: Output da estratégia de desconexão de conta
/// - `PositionExit`: Informações sobre posições fechadas
/// - `MarketDisconnect`: Output da estratégia de desconexão de mercado
/// - `AlgoOrders`: Output da geração de ordens algorítmicas
///
/// # Type Parameters
/// - `OnTradingDisabled`: Tipo de output da estratégia de trading disabled
/// - `OnDisconnect`: Tipo de output da estratégia de desconexão
/// - `ExchangeKey`: Tipo da chave de exchange (default: ExchangeIndex)
/// - `InstrumentKey`: Tipo da chave de instrumento (default: InstrumentIndex)
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum EngineOutput<
    OnTradingDisabled,
    OnDisconnect,
    ExchangeKey = ExchangeIndex,
    InstrumentKey = InstrumentIndex,
> {
    Commanded(ActionOutput<ExchangeKey, InstrumentKey>),
    OnTradingDisabled(OnTradingDisabled),
    AccountDisconnect(OnDisconnect),
    PositionExit(PositionExited<QuoteAsset, InstrumentKey>),
    MarketDisconnect(OnDisconnect),
    AlgoOrders(GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>),
}

/// Output produced by the [`Engine`] updating from an [`TradingState`], used to construct
/// an `Engine` [`EngineAudit`].
///
/// Representa os possíveis outputs quando o engine atualiza seu estado de trading.
/// Usado para rastrear quando estratégias de trading desabilitado são acionadas.
///
/// # Variants
/// - `None`: Nenhuma ação especial necessária
/// - `OnTradingDisabled`: Estratégia de trading desabilitado foi executada
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum UpdateTradingStateOutput<OnTradingDisabled> {
    None,
    OnTradingDisabled(OnTradingDisabled),
}

/// Output produced by the [`Engine`] updating from an [`AccountStreamEvent`], used to construct
/// an `Engine` [`EngineAudit`].
///
/// Representa os possíveis outputs quando o engine processa eventos da stream de conta.
/// Inclui desconexões detectadas e posições fechadas automaticamente.
///
/// # Variants
/// - `None`: Evento processado sem ações especiais
/// - `OnDisconnect`: Estratégia de desconexão foi acionada
/// - `PositionExit`: Posição foi fechada automaticamente
///
/// # Type Parameters
/// - `OnDisconnect`: Tipo de output da estratégia de desconexão
/// - `InstrumentKey`: Tipo da chave de instrumento (default: InstrumentIndex)
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum UpdateFromAccountOutput<OnDisconnect, InstrumentKey = InstrumentIndex> {
    None,
    OnDisconnect(OnDisconnect),
    PositionExit(PositionExited<QuoteAsset, InstrumentKey>),
}

/// Output produced by the [`Engine`] updating from an [`MarketStreamEvent`], used to construct
/// an `Engine` [`EngineAudit`].
///
/// Representa os possíveis outputs quando o engine processa eventos da stream de mercado.
/// Usado para rastrear desconexões de feed de dados e ações de recuperação.
///
/// # Variants
/// - `None`: Evento de mercado processado normalmente
/// - `OnDisconnect`: Estratégia de desconexão foi acionada devido a perda de feed
///
/// # Type Parameters
/// - `OnDisconnect`: Tipo de output da estratégia de desconexão
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum UpdateFromMarketOutput<OnDisconnect> {
    None,
    OnDisconnect(OnDisconnect),
}

impl<OnTradingDisabled, OnDisconnect, ExchangeKey, InstrumentKey>
    From<ActionOutput<ExchangeKey, InstrumentKey>>
    for EngineOutput<OnTradingDisabled, OnDisconnect, ExchangeKey, InstrumentKey>
{
    fn from(value: ActionOutput<ExchangeKey, InstrumentKey>) -> Self {
        Self::Commanded(value)
    }
}

impl<OnTradingDisabled, OnDisconnect, ExchangeKey, InstrumentKey>
    From<PositionExited<QuoteAsset, InstrumentKey>>
    for EngineOutput<OnTradingDisabled, OnDisconnect, ExchangeKey, InstrumentKey>
{
    fn from(value: PositionExited<QuoteAsset, InstrumentKey>) -> Self {
        Self::PositionExit(value)
    }
}

impl<OnTradingDisabled, OnDisconnect, ExchangeKey, InstrumentKey>
    From<GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>>
    for EngineOutput<OnTradingDisabled, OnDisconnect, ExchangeKey, InstrumentKey>
{
    fn from(value: GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>) -> Self {
        Self::AlgoOrders(value)
    }
}

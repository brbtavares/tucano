use crate::engine::state::IndexedInstruments;
use crate::{
    engine::{
        audit::{context::EngineContext, Auditor},
        clock::EngineClock,
        execution_tx::MultiExchangeTxMap,
        run::{async_run, async_run_with_audit, sync_run, sync_run_with_audit},
        state::{builder::EngineStateBuilder, trading::TradingState, EngineState},
        Engine, Processor,
    },
    error::TucanoError,
    execution::{
        builder::{ExecutionBuildFutures, ExecutionBuilder},
        AccountStreamEvent,
    },
    shutdown::SyncShutdown,
    system::{config::ExecutionConfig, System, SystemAuxillaryHandles},
};
use data::streams::reconnect::stream::ReconnectingStream;
use execution::{balance::Balance, InstrumentIndex};
use integration::{
    channel::{mpsc_unbounded, Channel, ChannelTxDroppable},
    snapshot::SnapUpdates,
    FeedEnded, Terminal,
};
use markets::{ConcreteInstrument, Keyed};

/// Placeholder types
pub type AssetNameInternal = String;
use derive_more::Constructor;
use fnv::FnvHashMap;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

/// Define como o `Engine` processa eventos de entrada.
///
/// Controla se o `Engine` roda de forma síncrona (thread bloqueante com `Iterator`)
/// ou assíncrona (via `Stream` em tasks tokio).
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Default)]
pub enum EngineFeedMode {
    /// Processa eventos de forma síncrona com `Iterator` em thread bloqueante (padrão).
    #[default]
    Iterator,

    /// Processa eventos de forma assíncrona com `Stream` em tasks tokio.
    ///
    /// Útil para múltiplos backtests concorrentes em escala.
    Stream,
}

/// Define se o `Engine` envia eventos de auditoria produzidos no canal de auditoria.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize, Default)]
pub enum AuditMode {
    /// Habilita envio de eventos de auditoria.
    Enabled,

    /// Desabilita envio de auditoria (padrão).
    #[default]
    Disabled,
}

/// Argumentos necessários para construir um sistema de trading completo Tucano.
///
/// Contém todos os componentes para montar e inicializar o sistema (Engine + infraestrutura).
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct SystemArgs<'a, Clock, Strategy, Risk, MarketStream, GlobalData, FnInstrumentData> {
    /// Coleção indexada de instrumentos que o sistema rastreará.
    pub instruments: &'a IndexedInstruments,

    /// Configurações de execução para conexões de exchanges.
    pub executions: Vec<ExecutionConfig>,

    /// Implementação de `EngineClock` para marcação de tempo.
    ///
    /// Ex: `HistoricalClock` (backtest) ou `LiveClock` (live/paper).
    pub clock: Clock,

    /// Implementação de `Strategy` do engine.
    pub strategy: Strategy,

    /// Implementação de `RiskManager` do engine.
    pub risk: Risk,

    /// `Stream` de `MarketStreamEvent`s.
    pub market_stream: MarketStream,

    /// `GlobalData` do `EngineState`.
    pub global_data: GlobalData,

    /// Closure usada na construção do `EngineState` para inicializar cada `InstrumentDataState`.
    pub instrument_data_init: FnInstrumentData,
}

/// Builder para construir um sistema de trading completo Tucano.
#[derive(Debug)]
pub struct SystemBuilder<'a, Clock, Strategy, Risk, MarketStream, GlobalData, FnInstrumentData> {
    args: SystemArgs<'a, Clock, Strategy, Risk, MarketStream, GlobalData, FnInstrumentData>,
    engine_feed_mode: Option<EngineFeedMode>,
    audit_mode: Option<AuditMode>,
    trading_state: Option<TradingState>,
    balances: FnvHashMap<AssetNameInternal, Balance>,
}

impl<'a, Clock, Strategy, Risk, MarketStream, GlobalData, FnInstrumentData>
    SystemBuilder<'a, Clock, Strategy, Risk, MarketStream, GlobalData, FnInstrumentData>
{
    /// Cria novo `SystemBuilder` com os `SystemArguments` fornecidos.
    ///
    /// Inicializa com valores padrão para configurações opcionais.
    pub fn new(
        config: SystemArgs<'a, Clock, Strategy, Risk, MarketStream, GlobalData, FnInstrumentData>,
    ) -> Self {
        Self {
            args: config,
            engine_feed_mode: None,
            audit_mode: None,
            trading_state: None,
            balances: FnvHashMap::default(),
        }
    }

    /// Configura opcionalmente o [`EngineFeedMode`] (`Iterator` ou `Stream`).
    ///
    /// Controla se o engine processa eventos de forma síncrona ou assíncrona.
    pub fn engine_feed_mode(self, value: EngineFeedMode) -> Self {
        Self {
            engine_feed_mode: Some(value),
            ..self
        }
    }

    /// Configura opcionalmente o [`AuditMode`] (habilitado ou desabilitado).
    ///
    /// Controla se o engine envia os eventos de auditoria que produz.
    pub fn audit_mode(self, value: AuditMode) -> Self {
        Self {
            audit_mode: Some(value),
            ..self
        }
    }

    /// Configura opcionalmente o [`TradingState`] inicial (habilitado ou desabilitado).
    ///
    /// Define se o trading algorítmico inicia habilitado quando o sistema sobe.
    pub fn trading_state(self, value: TradingState) -> Self {
        Self {
            trading_state: Some(value),
            ..self
        }
    }

    /// Fornece opcionalmente `Balance`s iniciais de ativos da exchange.
    ///
    /// Útil em cenários de backtest onde é necessário semear o `EngineState` com saldos iniciais.
    ///
    /// Observação: internamente usa um `HashMap`, então chaves duplicadas de
    /// `ExchangeAsset<AssetNameInternal>` sobrescrevem valores anteriores.
    pub fn balances<BalanceIter>(mut self, balances: BalanceIter) -> Self
    where
        BalanceIter: IntoIterator<Item = (AssetNameInternal, Balance)>,
    {
        self.balances.extend(balances);
        self
    }

    /// Constrói o [`SystemBuild`] com as configurações aplicadas ao builder.
    ///
    /// Constrói todos os componentes do sistema mas não inicia tasks ou streams.
    ///
    /// Inicialize a instância de `SystemBuild` para iniciar o sistema.
    pub fn build<Event, InstrumentData>(
        self,
    ) -> Result<
        SystemBuild<
            Engine<
                Clock,
                EngineState<GlobalData, InstrumentData>,
                MultiExchangeTxMap,
                Strategy,
                Risk,
            >,
            Event,
            MarketStream,
        >,
        TucanoError,
    >
    where
        Clock: EngineClock + Clone + Send + Sync + 'static,
        FnInstrumentData: Fn(&'a Keyed<InstrumentIndex, ConcreteInstrument>) -> InstrumentData,
    {
        let Self {
            args:
                SystemArgs {
                    instruments,
                    executions,
                    clock,
                    strategy,
                    risk,
                    market_stream,
                    global_data,
                    instrument_data_init,
                },
            engine_feed_mode,
            audit_mode,
            trading_state,
            balances,
        } = self;

        // Default if not provided
        let engine_feed_mode = engine_feed_mode.unwrap_or_default();
        let audit_mode = audit_mode.unwrap_or_default();
        let trading_state = trading_state.unwrap_or_default();

        // Build Execution infrastructure
        let execution = executions
            .into_iter()
            .try_fold(
                ExecutionBuilder::new(instruments),
                |builder, config| match config {
                    ExecutionConfig::Mock(mock_config) => {
                        builder.add_mock(mock_config, clock.clone())
                    }
                },
            )?
            .build();

        // Build EngineState
        let state = EngineStateBuilder::new(instruments, global_data, instrument_data_init)
            .time_engine_start(clock.time())
            .trading_state(trading_state)
            .balances(balances.into_iter())
            .build();

        // Construct Engine
        let engine = Engine::new(clock, state, execution.execution_tx_map, strategy, risk);

        Ok(SystemBuild {
            engine,
            engine_feed_mode,
            audit_mode,
            market_stream,
            account_channel: execution.account_channel,
            execution_build_futures: execution.futures,
            phantom_event: PhantomData,
        })
    }
}

/// `SystemBuild` totalmente construído e pronto para ser inicializado.
///
/// Passo intermediário antes de spawnar tasks e rodar o sistema.
#[allow(missing_debug_implementations)]
pub struct SystemBuild<Engine, Event, MarketStream> {
    /// Constructed `Engine` instance.
    pub engine: Engine,

    /// Selected [`EngineFeedMode`].
    pub engine_feed_mode: EngineFeedMode,

    /// Selected [`AuditMode`].
    pub audit_mode: AuditMode,

    /// `Stream` of `MarketStreamEvent`s.
    pub market_stream: MarketStream,

    /// Channel for `AccountStreamEvent`.
    pub account_channel: Channel<AccountStreamEvent>,

    /// Futures for initialising `ExecutionBuild` components.
    pub execution_build_futures: ExecutionBuildFutures,

    phantom_event: PhantomData<Event>,
}

impl<Engine, Event, MarketStream> SystemBuild<Engine, Event, MarketStream>
where
    Engine: Processor<Event>
        + Auditor<Engine::Audit, Context = EngineContext>
        + SyncShutdown
        + Send
        + 'static,
    Engine::Audit: From<FeedEnded> + Terminal + Debug + Clone + Send + 'static,
    Event: From<MarketStream::Item> + From<AccountStreamEvent> + Debug + Clone + Send + 'static,
    MarketStream: Stream + Send + 'static,
{
    /// Cria um novo `SystemBuild` a partir dos componentes fornecidos.
    pub fn new(
        engine: Engine,
        engine_feed_mode: EngineFeedMode,
        audit_mode: AuditMode,
        market_stream: MarketStream,
        account_channel: Channel<AccountStreamEvent>,
        execution_build_futures: ExecutionBuildFutures,
    ) -> Self {
        Self {
            engine,
            engine_feed_mode,
            audit_mode,
            market_stream,
            account_channel,
            execution_build_futures,
            phantom_event: Default::default(),
        }
    }

    /// Inicializa o sistema usando o runtime tokio atual.
    ///
    /// Spawn de todas as tasks necessárias e retorna a instância `System` em execução.
    pub async fn init(self) -> Result<System<Engine, Event>, TucanoError> {
        self.init_internal(tokio::runtime::Handle::current()).await
    }

    /// Inicializa o sistema usando um runtime tokio específico.
    ///
    /// Permite especificar um runtime customizado para spawn das tasks.
    pub async fn init_with_runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<System<Engine, Event>, TucanoError> {
        self.init_internal(runtime).await
    }

    async fn init_internal(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<System<Engine, Event>, TucanoError> {
        let Self {
            mut engine,
            engine_feed_mode,
            audit_mode,
            market_stream,
            account_channel,
            execution_build_futures,
            phantom_event: _,
        } = self;

        // Initialise all execution components
        let execution = execution_build_futures
            .init_with_runtime(runtime.clone())
            .await?;

        // Initialise central Engine channel
        let (feed_tx, mut feed_rx) = mpsc_unbounded();

        // Forward MarketStreamEvents to Engine feed
        let market_to_engine = runtime
            .clone()
            .spawn(market_stream.forward_to(feed_tx.clone()));

        // Forward AccountStreamEvents to Engine feed
        let account_stream = account_channel.rx.into_stream();
        let account_to_engine = runtime.spawn(account_stream.forward_to(feed_tx.clone()));

        // Run Engine in configured mode
        let (engine, audit) = match (engine_feed_mode, audit_mode) {
            (EngineFeedMode::Iterator, AuditMode::Enabled) => {
                // Initialise Audit channel
                let (audit_tx, audit_rx) = mpsc_unbounded();
                let mut audit_tx = ChannelTxDroppable::new(audit_tx);

                let audit = SnapUpdates {
                    snapshot: engine.audit_snapshot(),
                    updates: audit_rx,
                };

                let handle = runtime.spawn_blocking(move || {
                    let shutdown_audit =
                        sync_run_with_audit(&mut feed_rx, &mut engine, &mut audit_tx);

                    (engine, shutdown_audit)
                });

                (handle, Some(audit))
            }
            (EngineFeedMode::Iterator, AuditMode::Disabled) => {
                let handle = runtime.spawn_blocking(move || {
                    let shutdown_audit = sync_run(&mut feed_rx, &mut engine);
                    (engine, shutdown_audit)
                });

                (handle, None)
            }
            (EngineFeedMode::Stream, AuditMode::Enabled) => {
                // Initialise Audit channel
                let (audit_tx, audit_rx) = mpsc_unbounded();
                let mut audit_tx = ChannelTxDroppable::new(audit_tx);

                let audit = SnapUpdates {
                    snapshot: engine.audit_snapshot(),
                    updates: audit_rx,
                };

                let handle = runtime.spawn(async move {
                    let shutdown_audit =
                        async_run_with_audit(&mut feed_rx, &mut engine, &mut audit_tx).await;
                    (engine, shutdown_audit)
                });

                (handle, Some(audit))
            }
            (EngineFeedMode::Stream, AuditMode::Disabled) => {
                let handle = runtime.spawn(async move {
                    let shutdown_audit = async_run(&mut feed_rx, &mut engine).await;
                    (engine, shutdown_audit)
                });

                (handle, None)
            }
        };

        Ok(System {
            engine,
            handles: SystemAuxillaryHandles {
                execution,
                market_to_engine,
                account_to_engine,
            },
            feed_tx,
            audit,
        })
    }
}

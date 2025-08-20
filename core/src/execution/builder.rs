// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
use crate::engine::state::{IndexedInstruments, IndexedInstrumentsExt};
use crate::{
    engine::{clock::EngineClock, execution_tx::MultiExchangeTxMap},
    error::TucanoError,
    execution::{
        error::ExecutionError, manager::ExecutionManager, request::ExecutionRequest,
        AccountStreamEvent, Execution,
    },
    shutdown::AsyncShutdown,
};
use fnv::FnvHashMap;
use futures::{future::try_join_all, FutureExt};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, mpsc},
    task::{JoinError, JoinHandle},
};
use tucano_data::streams::{
    consumer::STREAM_RECONNECTION_POLICY, reconnect::stream::ReconnectingStream,
};
use tucano_execution::{
    client::{
        mock::{MockExecution, MockExecutionClientConfig, MockExecutionConfig},
        ExecutionClient,
    },
    exchange::mock::{request::MockExchangeRequest, MockExchange},
    indexer::AccountEventIndexer,
    map::generate_execution_instrument_map,
    UnindexedAccountEvent,
};
use tucano_execution::{AssetIndex, ExchangeIndex, InstrumentIndex}; // already tucano prefixed
use tucano_integration::channel::{mpsc_unbounded, Channel, UnboundedTx};
use tucano_instrument::exchange::ExchangeId;

/// Placeholder types
pub type AssetNameExchange = String;
pub type InstrumentNameExchange = String;

#[derive(Debug, Clone, PartialEq)]
pub enum InstrumentKind {
    Spot,
}

#[derive(Debug, Clone)]
pub struct InstrumentSpec {
    pub quantity: InstrumentSpecQuantity,
    pub price: Option<f64>,
    pub notional: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct InstrumentSpecQuantity {
    pub units: OrderQuantityUnits,
    pub unit: String,
    pub min: f64,
    pub increment: f64,
}

#[derive(Debug, Clone)]
pub enum OrderQuantityUnits {
    Contract,
    Quote,
}

type ExecutionInitFuture =
    Pin<Box<dyn Future<Output = Result<(RunFuture, RunFuture), ExecutionError>> + Send>>;
type RunFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Builder completo da infraestrutura de execução.
///
/// Adicione configurações de [`ExecutionClient`] Mock e Live e o builder monta a infraestrutura.
///
/// Após adicionar as configurações, chame [`ExecutionBuilder::build`] para obter o
/// [`ExecutionBuild`]. Em seguida chame [`ExecutionBuild::init`] para inicializar
/// toda a infraestrutura construída.
///
/// Responsabilidades:
/// - Construir managers de execução mock (simula internamente uma exchange via [`MockExchange`]).
/// - Construir managers live conectando-se externamente a cada exchange.
/// - Montar um [`MultiExchangeTxMap`] com entrada para cada manager mock/live.
/// - Unificar todos os streams de conta de exchanges em um único `Stream` de [`AccountStreamEvent`].
#[allow(missing_debug_implementations)]
pub struct ExecutionBuilder<'a> {
    instruments: &'a IndexedInstruments,
    execution_txs: FnvHashMap<ExchangeId, (ExchangeIndex, UnboundedTx<ExecutionRequest>)>,
    merged_channel: Channel<AccountStreamEvent<ExchangeIndex, AssetIndex, InstrumentIndex>>,
    mock_exchange_futures: Vec<RunFuture>,
    execution_init_futures: Vec<ExecutionInitFuture>,
}
impl<'a> ExecutionBuilder<'a> {
    /// Constrói novo `ExecutionBuilder` usando os `IndexedInstruments` fornecidos.
    pub fn new(instruments: &'a IndexedInstruments) -> Self {
        Self {
            instruments,
            execution_txs: FnvHashMap::default(),
            merged_channel: Channel::default(),
            mock_exchange_futures: Vec::default(),
            execution_init_futures: Vec::default(),
        }
    }

    /// Adiciona um [`ExecutionManager`] para uma exchange mock, montando internamente um [`MockExchange`].
    ///
    /// O [`MockExecutionConfig`] configura a [`MockExchange`] e provê estado inicial de conta.
    pub fn add_mock<Clock>(
        mut self,
        config: MockExecutionConfig,
        clock: Clock,
    ) -> Result<Self, TucanoError>
    where
        Clock: EngineClock + Clone + Send + Sync + 'static,
    {
        const ACCOUNT_STREAM_CAPACITY: usize = 256;
        const DUMMY_EXECUTION_REQUEST_TIMEOUT: Duration = Duration::from_secs(1);

        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = broadcast::channel(ACCOUNT_STREAM_CAPACITY);

        let mock_execution_client_config = MockExecutionClientConfig {
            mocked_exchange: config.mocked_exchange,
            clock: move || clock.time(),
            request_tx,
            event_rx,
        };

        // Register MockExchange init Future
        let mock_exchange_future = Self::init_mock_exchange(config, request_rx, event_tx);
        self.mock_exchange_futures.push(mock_exchange_future);

        self.add_execution::<MockExecution<_>>(
            mock_execution_client_config.mocked_exchange,
            mock_execution_client_config,
            DUMMY_EXECUTION_REQUEST_TIMEOUT,
        )
    }

    fn init_mock_exchange(
        config: MockExecutionConfig,
        request_rx: mpsc::UnboundedReceiver<MockExchangeRequest>,
        event_tx: broadcast::Sender<UnindexedAccountEvent>,
    ) -> RunFuture {
        // TODO: implement real filtering when instruments structure finalized
        let instruments = FnvHashMap::default();
        Box::pin(MockExchange::new(config, request_rx, event_tx, instruments).run())
    }

    /// Adiciona um [`ExecutionManager`] para uma exchange live.
    pub fn add_live<Client>(
        self,
        config: Client::Config,
        request_timeout: Duration,
    ) -> Result<Self, TucanoError>
    where
        Client: ExecutionClient + Send + Sync + 'static,
        Client::AccountStream: Send,
        Client::Config: Send,
    {
        self.add_execution::<Client>(Client::EXCHANGE, config, request_timeout)
    }

    fn add_execution<Client>(
        mut self,
        exchange: ExchangeId,
        config: Client::Config,
        request_timeout: Duration,
    ) -> Result<Self, TucanoError>
    where
        Client: ExecutionClient + Send + Sync + 'static,
        Client::AccountStream: Send,
        Client::Config: Send,
    {
        let instrument_map = generate_execution_instrument_map(self.instruments, exchange)?;
        let exchange_index_clone = instrument_map.exchange.key.clone();

        let (execution_tx, execution_rx) = mpsc_unbounded();

        if self
            .execution_txs
            .insert(exchange, (exchange_index_clone, execution_tx))
            .is_some()
        {
            return Err(TucanoError::ExecutionBuilder(format!(
                "ExecutionBuilder does not support duplicate mocked ExecutionManagers: {exchange}"
            )));
        }

        let merged_tx = self.merged_channel.tx.clone();

        // Init ExecutionManager Future
        let future_result = ExecutionManager::init(
            execution_rx.into_stream(),
            request_timeout,
            Arc::new(Client::new(config)),
            AccountEventIndexer::new(Arc::new(instrument_map)),
            STREAM_RECONNECTION_POLICY,
        );

        let future_result = future_result.map(|result| {
            result.map(|(manager, account_stream)| {
                let manager_future: RunFuture = Box::pin(manager.run());
                let stream_future: RunFuture = Box::pin(account_stream.forward_to(merged_tx));

                (manager_future, stream_future)
            })
        });

        self.execution_init_futures.push(Box::pin(future_result));

        Ok(self)
    }

    /// Consome este `ExecutionBuilder` e constrói um [`ExecutionBuild`] contendo todos os
    /// futures de [`ExecutionManager`] (mock & live) e [`MockExchange`].
    ///
    /// **Para a maioria dos usuários, chamar [`ExecutionBuild::init`] após isto é suficiente.**
    ///
    /// Se você quiser mais controle sobre qual runtime dirige os futures até a conclusão,
    /// chame [`ExecutionBuild::init_with_runtime`].
    pub fn build(mut self) -> ExecutionBuild {
        // Construct indexed ExecutionTx map
        let execution_tx_map = self
            .instruments
            .exchanges()
            .map(|exchange_id| {
                // Attempt to remove transmitter entry keyed by ExchangeId
                let Some((_exchange_index, execution_tx)) = self.execution_txs.remove(&exchange_id)
                else {
                    return (exchange_id, None);
                };
                // Add transmitter, cloning ExchangeId for map key
                (exchange_id, Some(execution_tx))
            })
            .collect();

        ExecutionBuild {
            execution_tx_map,
            account_channel: self.merged_channel,
            futures: ExecutionBuildFutures {
                mock_exchange_run_futures: self.mock_exchange_futures,
                execution_init_futures: self.execution_init_futures,
            },
        }
    }
}

/// Container holding execution infrastructure components ready to be initialised.
///
/// Chame [`ExecutionBuild::init`] para rodar todos os futures necessários dos componentes
/// de execução em tasks tokio - retorna o [`MultiExchangeTxMap`] e o stream multi‑exchange de
/// [`AccountStreamEvent`].
#[allow(missing_debug_implementations)]
pub struct ExecutionBuild {
    pub execution_tx_map: MultiExchangeTxMap,
    pub account_channel: Channel<AccountStreamEvent>,
    pub futures: ExecutionBuildFutures,
}

impl ExecutionBuild {
    /// Inicializa todos os componentes de execução no runtime tokio atual.
    ///
    /// Este método:
    /// - Faz spawn de tasks tokio com os runners de [`MockExchange`].
    /// - Inicializa todos os [`ExecutionManager`] e seus AccountStreams.
    /// - Retorna o `MultiExchangeTxMap` e o AccountStream multi‑exchange.
    pub async fn init(self) -> Result<Execution, TucanoError> {
        self.init_internal(tokio::runtime::Handle::current()).await
    }

    /// Inicializa todos os componentes de execução em um runtime tokio fornecido.
    ///
    /// Use quando quiser controlar qual runtime executa os componentes de execução.
    ///
    /// Este método:
    /// - Faz spawn de tasks tokio com os runners de [`MockExchange`].
    /// - Inicializa todos os [`ExecutionManager`] e seus AccountStreams.
    /// - Retorna o `MultiExchangeTxMap` e o AccountStream multi‑exchange.
    pub async fn init_with_runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<Execution, TucanoError> {
        self.init_internal(runtime).await
    }

    async fn init_internal(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<Execution, TucanoError> {
        let handles = self.futures.init_with_runtime(runtime).await?;

        Ok(Execution {
            execution_txs: self.execution_tx_map,
            account_channel: self.account_channel,
            handles,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct ExecutionBuildFutures {
    pub mock_exchange_run_futures: Vec<RunFuture>,
    pub execution_init_futures: Vec<ExecutionInitFuture>,
}

impl ExecutionBuildFutures {
    /// Inicializa todos os componentes de execução no runtime tokio atual.
    ///
    /// Este método:
    /// - Faz spawn de tasks tokio com os runners de [`MockExchange`].
    /// - Inicializa todos os [`ExecutionManager`] e seus AccountStreams.
    /// - Faz spawn de tasks para encaminhar AccountStreams para o AccountStream multi‑exchange
    pub async fn init(self) -> Result<ExecutionHandles, TucanoError> {
        self.init_internal(tokio::runtime::Handle::current()).await
    }

    /// Inicializa todos os componentes de execução em um runtime tokio fornecido.
    ///
    /// Use quando quiser mais controle sobre qual runtime executa os componentes.
    ///
    /// Este método:
    /// - Faz spawn de tasks tokio com os runners de [`MockExchange`].
    /// - Inicializa todos os [`ExecutionManager`] e seus AccountStreams.
    /// - Faz spawn de tasks para encaminhar AccountStreams para o AccountStream multi‑exchange
    pub async fn init_with_runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<ExecutionHandles, TucanoError> {
        self.init_internal(runtime).await
    }

    async fn init_internal(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<ExecutionHandles, TucanoError> {
        let mock_exchanges = self
            .mock_exchange_run_futures
            .into_iter()
            .map(|mock_exchange_run_future| runtime.spawn(mock_exchange_run_future))
            .collect();

        // Await ExecutionManager build futures and ensure success
        let (managers, account_to_engines) =
            futures::future::try_join_all(self.execution_init_futures)
                .await?
                .into_iter()
                .map(|(manager_run_future, account_event_forward_future)| {
                    (
                        runtime.spawn(manager_run_future),
                        runtime.spawn(account_event_forward_future),
                    )
                })
                .unzip();

        Ok(ExecutionHandles {
            mock_exchanges,
            managers,
            account_to_engines,
        })
    }
}

#[allow(missing_debug_implementations)]
pub struct ExecutionHandles {
    pub mock_exchanges: Vec<JoinHandle<()>>,
    pub managers: Vec<JoinHandle<()>>,
    pub account_to_engines: Vec<JoinHandle<()>>,
}

impl AsyncShutdown for ExecutionHandles {
    type Result = Result<(), JoinError>;

    async fn shutdown(&mut self) -> Self::Result {
        let handles = self
            .mock_exchanges
            .drain(..)
            .chain(self.managers.drain(..))
            .chain(self.account_to_engines.drain(..));

        try_join_all(handles).await?;
        Ok(())
    }
}

impl IntoIterator for ExecutionHandles {
    type Item = JoinHandle<()>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.mock_exchanges
            .into_iter()
            .chain(self.managers)
            .chain(self.account_to_engines)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

// (Removed unused generate_mock_exchange_instruments helper)

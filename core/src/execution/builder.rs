use crate::{
    engine::{
        clock::EngineClock, execution_tx::MultiExchangeTxMap,
    },
    error::ToucanError,
    execution::{
        error::ExecutionError, manager::ExecutionManager, request::ExecutionRequest,
        AccountStreamEvent, Execution,
    },
    shutdown::AsyncShutdown,
};
use data::streams::{consumer::STREAM_RECONNECTION_POLICY, reconnect::stream::ReconnectingStream};
use execution::{
    client::{
        mock::{MockExecution, MockExecutionClientConfig, MockExecutionConfig},
        ExecutionClient,
    },
    exchange::mock::{request::MockExchangeRequest, MockExchange},
    indexer::AccountEventIndexer,
    map::generate_execution_instrument_map,
    UnindexedAccountEvent,
};
use execution::{AssetIndex, ExchangeIndex, InstrumentIndex};
use fnv::FnvHashMap;
use futures::{future::try_join_all, FutureExt};
use integration::channel::{mpsc_unbounded, Channel, UnboundedTx};
use markets::{exchange::ExchangeId, ConcreteInstrument};
use crate::engine::state::{IndexedInstruments, IndexedInstrumentsExt};
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, mpsc},
    task::{JoinError, JoinHandle},
};

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

/// Full execution infrastructure builder.
///
/// Add Mock and Live [`ExecutionClient`] configurations and let the builder set up the required
/// infrastructure.
///
/// Once you have added all the configurations, call [`ExecutionBuilder::build`] to return the
/// full [`ExecutionBuild`]. Then calling [`ExecutionBuild::init`] will then initialise
/// the built infrastructure.
///
/// Handles:
/// - Building mock execution managers (mocks a specific exchange internally via the [`MockExchange`]).
/// - Building live execution managers, setting up an external connection to each exchange.
/// - Constructs a [`MultiExchangeTxMap`] with an entry for each mock/live execution manager.
/// - Combines all exchange account streams into a unified [`AccountStreamEvent`] `Stream`.
#[allow(missing_debug_implementations)]
pub struct ExecutionBuilder<'a> {
    instruments: &'a IndexedInstruments,
    execution_txs: FnvHashMap<ExchangeId, (ExchangeIndex, UnboundedTx<ExecutionRequest>)>,
    merged_channel: Channel<AccountStreamEvent<ExchangeIndex, AssetIndex, InstrumentIndex>>,
    mock_exchange_futures: Vec<RunFuture>,
    execution_init_futures: Vec<ExecutionInitFuture>,
}
impl<'a> ExecutionBuilder<'a> {
    /// Construct a new `ExecutionBuilder` using the provided `IndexedInstruments`.
    pub fn new(instruments: &'a IndexedInstruments) -> Self {
        Self {
            instruments,
            execution_txs: FnvHashMap::default(),
            merged_channel: Channel::default(),
            mock_exchange_futures: Vec::default(),
            execution_init_futures: Vec::default(),
        }
    }

    /// Adds an [`ExecutionManager`] for a mocked exchange, setting up a [`MockExchange`]
    /// internally.
    ///
    /// The provided [`MockExecutionConfig`] is used to configure the [`MockExchange`] and provide
    /// the initial account state.
    pub fn add_mock<Clock>(
        mut self,
        config: MockExecutionConfig,
        clock: Clock,
    ) -> Result<Self, ToucanError>
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
        let mock_exchange_future = self.init_mock_exchange(config, request_rx, event_tx);
        self.mock_exchange_futures.push(mock_exchange_future);

        self.add_execution::<MockExecution<_>>(
            mock_execution_client_config.mocked_exchange,
            mock_execution_client_config,
            DUMMY_EXECUTION_REQUEST_TIMEOUT,
        )
    }

    fn init_mock_exchange(
        &self,
        config: MockExecutionConfig,
        request_rx: mpsc::UnboundedReceiver<MockExchangeRequest>,
        event_tx: broadcast::Sender<UnindexedAccountEvent>,
    ) -> RunFuture {
        // TODO: implement real filtering when instruments structure finalized
        let instruments = FnvHashMap::default();
        Box::pin(MockExchange::new(config, request_rx, event_tx, instruments).run())
    }

    /// Adds an [`ExecutionManager`] for a live exchange.
    pub fn add_live<Client>(
        self,
        config: Client::Config,
        request_timeout: Duration,
    ) -> Result<Self, ToucanError>
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
    ) -> Result<Self, ToucanError>
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
            return Err(ToucanError::ExecutionBuilder(format!(
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

    /// Consume this `ExecutionBuilder` and build a full [`ExecutionBuild`] containing all the
    /// [`ExecutionManager`] (mock & live) and [`MockExchange`] futures.
    ///
    /// **For most users, calling [`ExecutionBuild::init`] after this is satisfactory.**
    ///
    /// If you want more control over what runtime drives the futures to completion, you can
    /// call [`ExecutionBuild::init_with_runtime`].
    pub fn build(mut self) -> ExecutionBuild {
        // Construct indexed ExecutionTx map
        let execution_tx_map = self
            .instruments
            .exchanges()
            .map(|exchange_id| {
                // Attempt to remove transmitter entry keyed by ExchangeId
                let Some((exchange_index, execution_tx)) = self.execution_txs.remove(&exchange_id) else {
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
/// Call [`ExecutionBuild::init`] to run all the required execution component futures on tokio
/// tasks - returns the [`MultiExchangeTxMap`] and multi-exchange [`AccountStreamEvent`] stream.
#[allow(missing_debug_implementations)]
pub struct ExecutionBuild {
    pub execution_tx_map: MultiExchangeTxMap,
    pub account_channel: Channel<AccountStreamEvent>,
    pub futures: ExecutionBuildFutures,
}

impl ExecutionBuild {
    /// Initialises all execution components on the current tokio runtime.
    ///
    /// This method:
    /// - Spawns [`MockExchange`] runners tokio tasks.
    /// - Initialises all [`ExecutionManager`]s and their AccountStreams.
    /// - Returns the `MultiExchangeTxMap` and multi-exchange AccountStream.
    pub async fn init(self) -> Result<Execution, ToucanError> {
        self.init_internal(tokio::runtime::Handle::current()).await
    }

    /// Initialises all execution components on the provided tokio runtime.
    ///
    /// Use this method if you want more control over which tokio runtime handles running
    /// execution components.
    ///
    /// This method:
    /// - Spawns [`MockExchange`] runners tokio tasks.
    /// - Initialises all [`ExecutionManager`]s and their AccountStreams.
    /// - Returns the `MultiExchangeTxMap` and multi-exchange AccountStream.
    pub async fn init_with_runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<Execution, ToucanError> {
        self.init_internal(runtime).await
    }

    async fn init_internal(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<Execution, ToucanError> {
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
    /// Initialises all execution components on the current tokio runtime.
    ///
    /// This method:
    /// - Spawns [`MockExchange`] runner tokio tasks.
    /// - Initialises all [`ExecutionManager`]s and their AccountStreams.
    /// - Spawns tokio tasks to forward AccountStreams to multi-exchange AccountStream
    pub async fn init(self) -> Result<ExecutionHandles, ToucanError> {
        self.init_internal(tokio::runtime::Handle::current()).await
    }

    /// Initialises all execution components on the provided tokio runtime.
    ///
    /// Use this method if you want more control over which tokio runtime handles running
    /// execution components.
    ///
    /// This method:
    /// - Spawns [`MockExchange`] runner tokio tasks.
    /// - Initialises all [`ExecutionManager`]s and their AccountStreams.
    /// - Spawns tokio tasks to forward AccountStreams to multi-exchange AccountStream
    pub async fn init_with_runtime(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<ExecutionHandles, ToucanError> {
        self.init_internal(runtime).await
    }

    async fn init_internal(
        self,
        runtime: tokio::runtime::Handle,
    ) -> Result<ExecutionHandles, ToucanError> {
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

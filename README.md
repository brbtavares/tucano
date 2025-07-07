# Toucan

Toucan is an algorithmic trading ecosystem of Rust libraries for building high-performance live-trading, paper-trading and back-testing systems.

* **Fast**: Written in native Rust. Minimal allocations. Data-oriented state management system with direct index lookups.
* **Robust**: Strongly typed. Thread safe. Extensive test coverage.
* **Customisable**: Plug and play Strategy and RiskManager components that facilitates most trading strategies (MarketMaking, StatArb, HFT, etc.).
* **Scalable**: Multithreaded architecture with modular design. Leverages Tokio for I/O. Memory efficient data structures.

**See: [`Toucan`], [`Toucan-Data`], [`Toucan-Instrument`], [`Toucan-Execution`] & [`Toucan-Integration`] for comprehensive documentation and examples for each library.**

[`Toucan`]: https://github.com/brbtavares/toucan
[`Toucan-Instrument`]: https://github.com/brbtavares/toucan/tree/main/toucan-instrument
[`Toucan-Data`]: https://github.com/brbtavares/toucan/tree/main/toucan-data
[`Toucan-Execution`]: https://github.com/brbtavares/toucan/tree/main/toucan-execution
[`Toucan-Integration`]: https://github.com/brbtavares/toucan/tree/main/toucan-integration

## Overview

Toucan is an algorithmic trading ecosystem of Rust libraries for building high-performance live-trading, paper-trading and back-testing systems. It is made up of several easy-to-use, extensible crates:

* **Toucan**: Algorithmic trading Engine with feature rich state management system.
* **Toucan-Instrument**: Exchange, Instrument and Asset data structures and utilities.
* **Toucan-Data**: Stream public market data from financial venues. Easily extensible via the MarketStream interface.
* **Toucan-Execution**: Stream private account data and execute orders. Easily extensible via the ExecutionClient interface.
* **Toucan-Integration**: Low-level frameworks for flexible REST/WebSocket integrations.

## Notable Features

* Stream public market data from financial venues via the [`Toucan-Data`] library.
* Stream private account data, execute orders (live or mock)** via the [`Toucan-Execution`] library.
* Plug and play Strategy and RiskManager components that facilitate most trading strategies.
* Backtest utilities for efficiently running thousands of concurrent backtests.
* Flexible Engine that facilitates trading strategies that execute on many exchanges simultaneously.
* Use mock MarketStream or Execution components to enable back-testing on a near-identical trading system as live-trading.
* Centralised cache friendly state management system with O(1) constant lookups using indexed data structures.
* Robust Order management system - use stand-alone or with Toucan.
* Trading summaries with comprehensive performance metrics (PnL, Sharpe, Sortino, Drawdown, etc.).
* Turn on/off algorithmic trading from an external process (eg/ UI, Telegram, etc.) whilst still processing market/account data.
* Issue Engine Commands from an external process (eg/ UI, Telegram, etc.) to initiate actions (CloseAllPositions, OpenOrders, CancelOrders, etc.).
* EngineState replica manager that processes the Engine AuditStream to facilitate non-hot path monitoring components (eg/ UI, Telegram, etc.).

[toucan-examples]: https://github.com/brbtavares/toucan/tree/main/toucan/examples

## Examples

* See [toucan examples][toucan-examples] for the compilable example including imports.
* See sub-crates for further examples of each library.

### Paper Trading With Live Market Data & Mock Execution

```rust,no_run
const FILE_PATH_SYSTEM_CONFIG: &str = "toucan/examples/config/system_config.json";
const RISK_FREE_RETURN: Decimal = dec!(0.05);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise Tracing
    init_logging();

    // Load SystemConfig
    let SystemConfig {
        instruments,
        executions,
    } = load_config()?;

    // Construct IndexedInstruments
    let instruments = IndexedInstruments::new(instruments);

    // Initialise MarketData Stream
    let market_stream = init_indexed_multi_exchange_market_stream(
        &instruments,
        &[SubKind::PublicTrades, SubKind::OrderBooksL1],
    )
    .await?;

    // Construct System Args
    let args = SystemArgs::new(
        &instruments,
        executions,
        LiveClock,
        DefaultStrategy::default(),
        DefaultRiskManager::default(),
        market_stream,
    );

    // Build & run full system:
    // See SystemBuilder for all configuration options
    let mut system = SystemBuilder::new(args)
        // Engine feed in Sync mode (Iterator input)
        .engine_feed_mode(EngineFeedMode::Iterator)

        // Audit feed is enabled (Engine sends audits)
        .audit_mode(AuditMode::Enabled)

        // Engine starts with TradingState::Disabled
        .trading_state(TradingState::Disabled)

        // Build System, but don't start spawning tasks yet
        .build::<EngineEvent, DefaultGlobalData, DefaultInstrumentMarketData>()?

        // Init System, spawning component tasks on the current runtime
        .init_with_runtime(tokio::runtime::Handle::current())
        .await?;

    // Take ownership of Engine audit receiver
    let audit_rx = system.audit_rx.take().unwrap();

    // Run dummy asynchronous AuditStream consumer
    // Note: you probably want to use this Stream to replicate EngineState, or persist events, etc.
    //  --> eg/ see examples/engine_sync_with_audit_replica_engine_state
    let audit_task = tokio::spawn(async move {
        let mut audit_stream = audit_rx.into_stream();
        while let Some(audit) = audit_stream.next().await {
            debug!(?audit, "AuditStream consumed AuditTick");
            if let EngineAudit::Shutdown(_) = audit.event {
                break;
            }
        }
        audit_stream
    });

    // Enable trading
    system.trading_state(TradingState::Enabled);

    // Let the example run for 5 seconds...
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Before shutting down, CancelOrders and then ClosePositions
    system.cancel_orders(InstrumentFilter::None);
    system.close_positions(InstrumentFilter::None);

    // Shutdown
    let (engine, _shutdown_audit) = system.shutdown().await?;
    let _audit_stream = audit_task.await?;

    // Generate TradingSummary<Daily>
    let trading_summary = engine
        .trading_summary_generator(RISK_FREE_RETURN)
        .generate(Daily);

    // Print TradingSummary<Daily> to terminal (could save in a file, send somewhere, etc.)
    trading_summary.print_summary();

    Ok(())
}

fn load_config() -> Result<SystemConfig, Box<dyn std::error::Error>> {
    let file = File::open(FILE_PATH_SYSTEM_CONFIG)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}
```

## Credits

This project is based on the excellent [Barter-rs](https://github.com/barter-rs/barter-rs) project. See [CREDITS.md](CREDITS.md) for full attribution and acknowledgments to the original developers.


## LEGAL DISCLAIMER AND LIMITATION OF LIABILITY

PLEASE READ THIS DISCLAIMER CAREFULLY BEFORE USING THE SOFTWARE. BY ACCESSING OR USING THE SOFTWARE, YOU ACKNOWLEDGE AND AGREE TO BE BOUND BY THE TERMS HEREIN.

1. EDUCATIONAL PURPOSE
   This software and related documentation ("Software") are provided solely for educational and research purposes. The Software is not intended, designed, tested, verified or certified for commercial deployment, live trading, or production use of any kind.

2. NO FINANCIAL ADVICE
   Nothing contained in the Software constitutes financial, investment, legal, or tax advice. No aspect of the Software should be relied upon for trading decisions or financial planning. Users are strongly advised to consult qualified professionals for investment guidance specific to their circumstances.

3. ASSUMPTION OF RISK
   Trading in financial markets, including but not limited to cryptocurrencies, securities, derivatives, and other financial instruments, carries substantial risk of loss. Users acknowledge that:
   a) They may lose their entire investment;
   b) Past performance does not indicate future results;
   c) Hypothetical or simulated performance results have inherent limitations and biases.

4. DISCLAIMER OF WARRANTIES
   THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED. TO THE MAXIMUM EXTENT PERMITTED BY LAW, THE AUTHORS AND COPYRIGHT HOLDERS EXPRESSLY DISCLAIM ALL WARRANTIES, INCLUDING BUT NOT LIMITED TO:
   a) MERCHANTABILITY
   b) FITNESS FOR A PARTICULAR PURPOSE
   c) NON-INFRINGEMENT
   d) ACCURACY OR RELIABILITY OF RESULTS
   e) SYSTEM INTEGRATION
   f) QUIET ENJOYMENT

5. LIMITATION OF LIABILITY
   IN NO EVENT SHALL THE AUTHORS, COPYRIGHT HOLDERS, CONTRIBUTORS, OR ANY AFFILIATED PARTIES BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING BUT NOT LIMITED TO PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES, LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

6. REGULATORY COMPLIANCE
   The Software is not registered with, endorsed by, or approved by any financial regulatory authority. Users are solely responsible for:
   a) Determining whether their use complies with applicable laws and regulations
   b) Obtaining any required licenses, permits, or registrations
   c) Meeting any regulatory obligations in their jurisdiction

7. INDEMNIFICATION
   Users agree to indemnify, defend, and hold harmless the authors, copyright holders, and any affiliated parties from and against any claims, liabilities, damages, losses, and expenses arising from their use of the Software.

8. ACKNOWLEDGMENT
   BY USING THE SOFTWARE, USERS ACKNOWLEDGE THAT THEY HAVE READ THIS DISCLAIMER, UNDERSTOOD IT, AND AGREE TO BE BOUND BY ITS TERMS AND CONDITIONS.

THE ABOVE LIMITATIONS MAY NOT APPLY IN JURISDICTIONS THAT DO NOT ALLOW THE EXCLUSION OF CERTAIN WARRANTIES OR LIMITATIONS OF LIABILITY.

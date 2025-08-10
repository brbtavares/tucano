//! # Engine Event Processing Integration Test
//!
//! This comprehensive integration test validates the complete event processing flow
//! of the Toucan trading engine with full audit trail capabilities. It simulates
//! a realistic trading scenario including market data events, account updates,
//! order execution, and position management.
//!
//! ## Test Scenario Overview
//!
//! The test simulates a complete trading session with the following flow:
//! 1. **System Initialization**: Set up engine with mock exchange and initial balances
//! 2. **Market Data Processing**: Process market events to establish instrument prices
//! 3. **Strategy Execution**: Enable trading and generate algorithmic orders
//! 4. **Order Lifecycle**: Simulate complete order execution from request to fill
//! 5. **Position Management**: Test position opening, tracking, and closing
//! 6. **Risk Events**: Handle exchange disconnections and trading state changes
//! 7. **Manual Commands**: Test manual position closure commands
//! 8. **Performance Analysis**: Generate trading summary and validate metrics
//!
//! ## Key Components Tested
//!
//! ### Engine Event Processing
//! - Event sequencing and audit trail generation
//! - State transitions and consistency validation
//! - Multi-instrument coordination
//!
//! ### Trading Strategy Integration
//! - Algorithmic order generation (Buy and Hold strategy)
//! - Position sizing and risk management
//! - Strategy state management across events
//!
//! ### Execution System
//! - Order request routing and response handling
//! - Balance tracking and synchronization
//! - Trade settlement and fee calculation
//!
//! ### Market Data Integration
//! - Real-time price updates and instrument data
//! - Exchange connectivity status management
//! - Market event processing and state updates
//!
//! ## Test Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    TEST ENVIRONMENT                        │
//! ├─────────────────┬─────────────────┬─────────────────────────┤
//! │ Mock Exchange   │ Test Strategy   │ Audit Verification      │
//! │                 │                 │                         │
//! │ • Mock orders   │ • Buy & Hold    │ • Sequence tracking     │
//! │ • Mock balances │ • Auto positions│ • Event validation      │
//! │ • Mock trades   │ • Close on cmd  │ • State consistency     │
//! └─────────────────┴─────────────────┴─────────────────────────┘
//! ```
//!
//! ## Test Data Structure
//!
//! The test uses generic financial instruments instead of specific crypto pairs:
//! - **Primary Instrument**: Base/Quote trading pair (represents major asset pair)
//! - **Secondary Instrument**: Alt/Base trading pair (represents alternative asset pair)
//! - **Quote Asset**: Primary quote currency (e.g., USD, USDT, etc.)
//! - **Base Assets**: Primary and alternative base assets
//!
//! ## Assertions and Validations
//!
//! Each test step includes comprehensive assertions for:
//! - **Sequence Numbers**: Ensuring proper event ordering
//! - **State Consistency**: Validating engine state after each event
//! - **Balance Accuracy**: Verifying balance calculations including fees
//! - **Position Tracking**: Confirming position opening, updating, and closing
//! - **Audit Trail**: Ensuring complete audit log generation
//! - **Performance Metrics**: Validating PnL calculations and trading statistics

use core::{
    engine::{
        action::{
            generate_algo_orders::GenerateAlgoOrdersOutput,
            send_requests::{SendCancelsAndOpensOutput, SendRequestsOutput},
            ActionOutput,
        },
        audit::EngineAudit,
        clock::HistoricalClock,
        command::Command,
        execution_tx::MultiExchangeTxMap,
        process_with_audit,
        state::{
            asset::AssetStates,
            connectivity::Health,
            global::DefaultGlobalData,
            instrument::{
                data::{DefaultInstrumentMarketData, InstrumentDataState},
                filter::InstrumentFilter,
            },
            position::PositionExited,
            trading::TradingState,
            EngineState,
        },
        Engine, EngineOutput,
    },
    execution::{request::ExecutionRequest, AccountStreamEvent},
    test_utils::time_plus_days,
    EngineEvent, Sequence, Timed,
};

use data::{
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
    subscription::trade::PublicTrade,
};

use execution::{
    balance::{AssetBalance, Balance},
    order::{
        id::{ClientOrderId, OrderId, StrategyId},
        request::{OrderRequestCancel, OrderRequestOpen, RequestOpen},
        state::{ActiveOrderState, Open, OrderState},
        Order, OrderKey, OrderKind, TimeInForce,
    },
    trade::{AssetFees, Trade, TradeId},
    AccountEvent, AccountEventKind, AccountSnapshot,
};

use markets::{ExchangeId, Side};
use core::engine::state::IndexedInstruments; // instrument list alias

use risk::DefaultRiskManager;

use strategy::{
    algo::AlgoStrategy, close_positions::ClosePositionsStrategy,
    on_disconnect::OnDisconnectStrategy, on_trading_disabled::OnTradingDisabled,
};

use chrono::{DateTime, Utc};
use fnv::FnvHashMap;
use integration::{
    channel::{mpsc_unbounded, UnboundedTx},
    collection::{none_one_or_many::NoneOneOrMany, one_or_many::OneOrMany},
    snapshot::Snapshot,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Test configuration constants
const STARTING_TIMESTAMP: DateTime<Utc> = DateTime::<Utc>::MIN_UTC;
const RISK_FREE_RETURN: Decimal = dec!(0.05);

// Initial account balances for testing
const STARTING_BALANCE_QUOTE: Balance = Balance {
    total: dec!(40_000.0), // Quote currency (e.g., USD, USDT)
    free: dec!(40_000.0),
};
const STARTING_BALANCE_BASE: Balance = Balance {
    total: dec!(1.0), // Primary base asset
    free: dec!(1.0),
};
const STARTING_BALANCE_ALT: Balance = Balance {
    total: dec!(10.0), // Alternative base asset
    free: dec!(10.0),
};
const QUOTE_FEES_PERCENT: f64 = 0.1; // 10% trading fees

/// Integration test for engine event processing with comprehensive audit trail.
///
/// This test simulates a complete trading session including:
/// - Market data processing for two trading pairs
/// - Algorithmic strategy execution (Buy and Hold)
/// - Order lifecycle management with fills and settlements
/// - Position tracking and PnL calculation
/// - Exchange connectivity events and recovery
/// - Manual position closure commands
/// - Trading performance analysis and reporting
#[test]
fn test_engine_process_engine_event_with_audit() {
    let (execution_tx, mut execution_rx) = mpsc_unbounded();

    let mut engine = build_engine(TradingState::Disabled, execution_tx);
    assert_eq!(engine.meta.sequence, Sequence(0));
    assert_eq!(engine.state.connectivity.global, Health::Reconnecting);

    // Simulate AccountSnapshot from ExecutionManager::init
    let event = account_event_snapshot(&engine.state.assets);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(0));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(engine.state.connectivity.global, Health::Reconnecting);

    // Process 1st MarketEvent for primary_pair (base/quote)
    let event = market_event_trade(1, 0, 10_000.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(1));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(engine.state.connectivity.global, Health::Healthy);

    // Process 1st MarketEvent for secondary_pair (alt/base)
    let event = market_event_trade(1, 1, 0.1);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(2));
    assert_eq!(audit.event, EngineAudit::process(event));

    // TradingState::Enabled -> expect BuyAndHoldStrategy to open Buy orders
    let event = EngineEvent::TradingStateUpdate(TradingState::Enabled);
    let audit = process_with_audit(&mut engine, event);
    assert_eq!(audit.context.sequence, Sequence(3));
    let primary_buy_order = OrderRequestOpen {
        key: OrderKey {
            exchange: "mock".to_string(),
            instrument: "inst0".to_string(),
            strategy: strategy_id(),
            cid: gen_cid(0),
        },
        state: RequestOpen {
            side: Side::Buy,
            kind: OrderKind::Market,
            time_in_force: TimeInForce::ImmediateOrCancel,
            price: dec!(10_000),
            quantity: dec!(1),
        },
    };
    let secondary_buy_order = OrderRequestOpen {
        key: OrderKey {
            exchange: "mock".to_string(),
            instrument: "inst1".to_string(),
            strategy: strategy_id(),
            cid: gen_cid(1),
        },
        state: RequestOpen {
            side: Side::Buy,
            kind: OrderKind::Market,
            time_in_force: TimeInForce::ImmediateOrCancel,
            price: dec!(0.1),
            quantity: dec!(1),
        },
    };
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(
            EngineEvent::TradingStateUpdate(TradingState::Enabled),
            EngineOutput::AlgoOrders(GenerateAlgoOrdersOutput {
                cancels_and_opens: SendCancelsAndOpensOutput {
                    cancels: SendRequestsOutput::default(),
                    opens: SendRequestsOutput {
                        sent: NoneOneOrMany::Many(vec![
                            primary_buy_order.clone(),
                            secondary_buy_order.clone(),
                        ]),
                        errors: NoneOneOrMany::None,
                    },
                },
                ..Default::default()
            })
        )
    );

    // Ensure ExecutionRequests were sent to ExecutionManager
    assert_eq!(
        execution_rx.next().unwrap(),
        ExecutionRequest::Open(primary_buy_order)
    );
    assert_eq!(
        execution_rx.next().unwrap(),
        ExecutionRequest::Open(secondary_buy_order)
    );

    // TradingState::Disabled
    let event = EngineEvent::TradingStateUpdate(TradingState::Disabled);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(4));
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(
            event,
            EngineOutput::OnTradingDisabled(OnTradingDisabledOutput)
        )
    );

    // Simulate OpenOrder response for Sequence(3) primary_buy_order
    let event = account_event_order_response(0, 2, Side::Buy, 10_000.0, 1.0, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(5));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert!(engine
        .state
        .instruments
        .instrument_index(&"inst0".to_string())
        .orders
        .0
        .is_empty());

    // Simulate Trade update for Sequence(3) primary_buy_order (fees 10% -> 1000 quote)
    let event = account_event_trade(0, 2, Side::Buy, 10_000.0, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(6));
    assert_eq!(audit.event, EngineAudit::process(event));

    // Simulate Balance update for Sequence(3) primary_buy_order, AssetIndex(2)/quote reduction
    let event = account_event_balance(2, 2, 9_000.0, 9_000.0); // 10k - 10% fees
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(7));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"quote".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(9_000.0), dec!(9_000.0)),
            time_plus_days(STARTING_TIMESTAMP, 2)
        )
    );
    // Simulate Balance update for Sequence(3) primary_buy_order, AssetIndex(0)/base increase
    let event = account_event_balance(0, 2, 2.0, 2.0); // 1 base + 1 base
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(8));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"base".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(2.0), dec!(2.0)),
            time_plus_days(STARTING_TIMESTAMP, 2)
        )
    );

    // Simulate OpenOrder response for Sequence(3) secondary_buy_order
    let event = account_event_order_response(1, 2, Side::Buy, 0.1, 1.0, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(9));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert!(engine
        .state
        .instruments
        .instrument_index(&"inst1".to_string())
        .orders
        .0
        .is_empty());

    // Simulate Trade update for Sequence(3) secondary_buy_order (fees 10% -> 0.01 base)
    let event = account_event_trade(1, 2, Side::Buy, 0.1, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(10));
    assert_eq!(audit.event, EngineAudit::process(event));

    // Simulate Balance update for Sequence(3) secondary_buy_order, AssetIndex(0)/base reduction
    let event = account_event_balance(0, 2, 0.99, 0.99); // 1 base - 10% fees
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(11));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"base".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(0.99), dec!(0.99)),
            time_plus_days(STARTING_TIMESTAMP, 2)
        )
    );

    // Simulate Balance update for Sequence(3) secondary_buy_order, AssetIndex(1)/alt increase
    let event = account_event_balance(1, 2, 11.0, 11.0); // 10 alt + 1 alt
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(12));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"alt".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(11.0), dec!(11.0)),
            time_plus_days(STARTING_TIMESTAMP, 2)
        )
    );

    // Process 2nd MarketEvent for primary_pair
    let event = market_event_trade(2, 0, 20_000.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(13));
    assert_eq!(audit.event, EngineAudit::process(event));

    // Process 2nd MarketEvent for secondary_pair
    let event = market_event_trade(2, 1, 0.05);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(14));
    assert_eq!(audit.event, EngineAudit::process(event));

    // Send ClosePositionsCommand for primary_pair
    let event = command_close_position(0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(15));
    let primary_sell_order = OrderRequestOpen {
        key: OrderKey {
            exchange: "mock".to_string(),
            instrument: "inst0".to_string(),
            strategy: strategy_id(),
            cid: gen_cid(0),
        },
        state: RequestOpen {
            side: Side::Sell,
            kind: OrderKind::Market,
            time_in_force: TimeInForce::ImmediateOrCancel,
            price: dec!(20_000),
            quantity: dec!(1),
        },
    };
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(
            event,
            EngineOutput::Commanded(ActionOutput::ClosePositions(SendCancelsAndOpensOutput {
                cancels: SendRequestsOutput::default(),
                opens: SendRequestsOutput {
                    sent: NoneOneOrMany::One(primary_sell_order.clone()),
                    errors: NoneOneOrMany::None,
                },
            }))
        )
    );

    // Ensure ClosePositions ExecutionRequest was sent to ExecutionManager
    assert_eq!(
        execution_rx.next().unwrap(),
        ExecutionRequest::Open(primary_sell_order)
    );

    // Simulate OpenOrder response for Sequence(15) ClosePositionsCommand primary_sell_order
    let event = account_event_order_response(0, 3, Side::Sell, 20_000.0, 1.0, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(16));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert!(engine
        .state
        .instruments
        .instrument_index(&"inst0".to_string())
        .orders
        .0
        .is_empty());

    // Simulate Balance update for Sequence(15) primary_sell_order, AssetIndex(2)/quote increase
    let event = account_event_balance(2, 3, 27_000.0, 27_000.0); // 9k + 20k - 10% fees
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(17));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"quote".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(27_000.0), dec!(27_000.0)),
            time_plus_days(STARTING_TIMESTAMP, 3)
        )
    );

    // Simulate Balance update for Sequence(15) primary_sell_order, AssetIndex(0)/base decrease
    let event = account_event_balance(0, 3, 1.0, 1.0); // 2 base - 1 base
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(18));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"base".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(1.0), dec!(1.0)),
            time_plus_days(STARTING_TIMESTAMP, 3)
        )
    );

    // Simulate Trade update for Sequence(15) primary_sell_order (fees 10% -> 2000 quote)
    let event = account_event_trade(0, 3, Side::Sell, 20_000.0, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(19));
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(
            event,
            PositionExited {
                instrument: "inst0".to_string(),
                side: Side::Buy,
                price_entry_average: dec!(10_000.0),
                quantity_abs_max: dec!(1.0),
                pnl_realised: dec!(7000.0), // (-10k entry - 1k fees)+(20k exit - 2k fees) = 7k
                fees_enter: AssetFees::quote_fees(dec!(1_000.0)),
                fees_exit: AssetFees::quote_fees(dec!(2_000.0)),
                time_enter: time_plus_days(STARTING_TIMESTAMP, 2),
                time_exit: time_plus_days(STARTING_TIMESTAMP, 3),
                trades: vec![gen_trade_id(0), gen_trade_id(0)],
            }
        )
    ); // Simulate exchange disconnection
    let event = EngineEvent::Market(MarketStreamEvent::Reconnecting(ExchangeId::Mock));
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(20));
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(event, EngineOutput::MarketDisconnect(OnDisconnectOutput))
    );
    assert_eq!(engine.state.connectivity.global, Health::Reconnecting);
    assert_eq!(
        engine
            .state
            .connectivity
            .connectivity(&ExchangeId::Mock)
            .market_data,
        Health::Reconnecting
    );
    assert_eq!(
        engine
            .state
            .connectivity
            .connectivity(&ExchangeId::Mock)
            .account,
        Health::Healthy
    );

    // Issue Command::SendOpenRequests OrderKind::LIMIT to close secondary position
    let secondary_sell_order = OrderRequestOpen {
        key: OrderKey {
            exchange: "mock".to_string(),
            instrument: "inst1".to_string(),
            strategy: strategy_id(),
            cid: gen_cid(1),
        },
        state: RequestOpen {
            side: Side::Sell,
            kind: OrderKind::Limit,
            time_in_force: TimeInForce::GoodUntilCancelled { post_only: true },
            price: dec!(0.05),
            quantity: dec!(1),
        },
    };
    let event = EngineEvent::Command(Command::SendOpenRequests(OneOrMany::One(
        secondary_sell_order.clone(),
    )));
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(21));
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(
            event,
            EngineOutput::Commanded(ActionOutput::OpenOrders(SendRequestsOutput {
                sent: NoneOneOrMany::One(secondary_sell_order.clone()),
                errors: NoneOneOrMany::None,
            }))
        )
    );

    // Ensure ExecutionRequest for Sequence(21) Command::SendOpenRequests was sent to ExecutionManager
    assert_eq!(
        execution_rx.next().unwrap(),
        ExecutionRequest::Open(secondary_sell_order)
    );

    // Simulate LIMIT OpenOrder response for Sequence(21) secondary_sell_order (0/1 quantity filled)
    let event = account_event_order_response(1, 4, Side::Sell, 0.05, 1.0, 0.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(22));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .instruments
            .instrument_index(&"inst1".to_string())
            .orders
            .0
            .len(),
        1
    );
    assert_eq!(
        engine
            .state
            .instruments
            .instrument_index(&"inst1".to_string())
            .orders
            .0
            .get(&gen_cid(1))
            .unwrap(),
        &Order {
            key: OrderKey {
                exchange: "mock".to_string(),
                instrument: "inst1".to_string(),
                strategy: strategy_id(),
                cid: gen_cid(1),
            },
            side: Side::Sell,
            price: dec!(0.05),
            quantity: dec!(1),
            kind: OrderKind::Limit,
            time_in_force: TimeInForce::GoodUntilCancelled { post_only: true },
            state: ActiveOrderState::Open(Open {
                id: gen_order_id(1),
                time_exchange: time_plus_days(STARTING_TIMESTAMP, 4),
                filled_quantity: dec!(0),
            }),
        }
    );

    // Simulate Balance update for Sequence(21) secondary_sell_order, AssetIndex(1)/alt free reduction
    let event = account_event_balance(1, 4, 11.0, 10.0); // 1 alt in order
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(23));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"alt".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(11.0), dec!(10.0)),
            time_plus_days(STARTING_TIMESTAMP, 4)
        )
    );

    // Simulate Order FullyFilled update for Sequence(21) LIMIT secondary_sell_order
    let event = EngineEvent::Account(AccountStreamEvent::Item(AccountEvent {
        exchange: "mock".to_string(),
        broker: Some("mock-broker".to_string()),
        account: Some("mock-account".to_string()),
        kind: AccountEventKind::OrderSnapshot(Snapshot(Order {
            key: OrderKey {
                exchange: "mock".to_string(),
                instrument: "inst1".to_string(),
                strategy: strategy_id(),
                cid: gen_cid(1),
            },
            side: Side::Sell,
            price: dec!(0.05),
            quantity: dec!(1),
            kind: OrderKind::Limit,
            time_in_force: TimeInForce::GoodUntilCancelled { post_only: true },
            state: OrderState::fully_filled(),
        })),
    }));
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(24));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert!(engine
        .state
        .instruments
        .instrument_index(&"inst1".to_string())
        .orders
        .0
        .is_empty());

    // Simulate Trade update for Sequence(21) LIMIT secondary_sell_order
    let event = account_event_trade(1, 5, Side::Sell, 0.05, 1.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(25));
    assert_eq!(
        audit.event,
        EngineAudit::process_with_output(
            event,
            PositionExited {
                instrument: "inst1".to_string(),
                side: Side::Buy,
                price_entry_average: dec!(0.1),
                quantity_abs_max: dec!(1.0),
                pnl_realised: dec!(-0.065), // 0.05 - 0.01 - 0.01 entry fees - 0.005 exit fees
                fees_enter: AssetFees::quote_fees(dec!(0.01)), // 0.01
                fees_exit: AssetFees::quote_fees(dec!(0.005)), // 0.005
                time_enter: time_plus_days(STARTING_TIMESTAMP, 2),
                time_exit: time_plus_days(STARTING_TIMESTAMP, 5),
                trades: vec![gen_trade_id(1), gen_trade_id(1)],
            }
        )
    );

    // Simulate Balance update
    let event = account_event_balance(1, 5, 10.0, 10.0);
    let audit = process_with_audit(&mut engine, event.clone());
    assert_eq!(audit.context.sequence, Sequence(26));
    assert_eq!(audit.event, EngineAudit::process(event));
    assert_eq!(
        engine
            .state
            .assets
            .asset_index(&"alt".to_string())
            .balance
            .unwrap(),
        Timed::new(
            Balance::new(dec!(10.0), dec!(10.0)),
            time_plus_days(STARTING_TIMESTAMP, 5)
        )
    );

    // End trading session and produce TradingSummaryGenerator
    let mut summary = engine.trading_summary_generator(RISK_FREE_RETURN);
    summary.update_time_now(time_plus_days(STARTING_TIMESTAMP, 5));

    assert_eq!(summary.risk_free_return, RISK_FREE_RETURN);
    assert_eq!(
        summary.time_engine_now,
        time_plus_days(STARTING_TIMESTAMP, 5)
    );

    // Collect realised PnL values across instruments and assert expected ones exist
    let mut pnls: Vec<_> = summary
        .instruments
        .values()
        .map(|ts| ts.pnl_returns.pnl_raw)
        .collect();
    pnls.sort();
    assert!(pnls.contains(&dec!(7000.0)), "Missing primary instrument PnL 7000.0: {:?}", pnls);
    assert!(pnls.contains(&dec!(-0.065)), "Missing secondary instrument PnL -0.065: {:?}", pnls);

    // Todo: Additional assertions + TradingSummary assertions once generated (to test TimeInterval)
}

/// Test implementation of a simple Buy and Hold algorithmic trading strategy.
///
/// This strategy demonstrates basic algorithmic order generation by:
/// 1. Opening buy positions when no position exists and no orders are in-flight
/// 2. Using market orders for immediate execution
/// 3. Sizing positions to a fixed quantity (1 unit)
/// 4. Supporting position closure on command
///
/// The strategy serves as a testing vehicle for validating engine event processing,
/// order lifecycle management, and audit trail generation.
struct TestBuyAndHoldStrategy {
    id: StrategyId,
}

impl AlgoStrategy for TestBuyAndHoldStrategy {
    type State = EngineState<DefaultGlobalData, DefaultInstrumentMarketData>;

    fn generate_algo_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<String, String>>,
        impl IntoIterator<Item = OrderRequestOpen<String, String>>,
    ) {
        let opens = state
            .instruments
            .instruments(&InstrumentFilter::None)
            .filter_map(|state| {
                if state.position.current.is_some() { return None; }
                if !state.orders.0.is_empty() { return None; }
                let price = state.data.price()?;
                let cid = if state.key == "inst0" { gen_cid(0) } else { gen_cid(1) };
                Some(OrderRequestOpen {
                    key: OrderKey {
                        exchange: state.instrument.exchange.to_string().to_lowercase(),
                        instrument: state.key.clone(),
                        strategy: self.id.clone(),
                        cid,
                    },
                    state: RequestOpen { side: Side::Buy, kind: OrderKind::Market, time_in_force: TimeInForce::ImmediateOrCancel, price, quantity: dec!(1) },
                })
            });
        (std::iter::empty(), opens)
    }
}

/// Returns the standard strategy identifier used across all test scenarios.
fn strategy_id() -> StrategyId {
    StrategyId::new("TestBuyAndHoldStrategy")
}

/// Generates a client order ID based on the instrument index.
/// This ensures unique order IDs for each instrument in the test.
fn gen_cid(instrument: usize) -> ClientOrderId {
    ClientOrderId::new(format!("inst{}", instrument))
}

/// Generates a trade ID based on the instrument index.
/// Used for simulating trade events in the test scenarios.
fn gen_trade_id(instrument: usize) -> TradeId {
    TradeId::new(format!("trade_inst{}", instrument))
}

/// Generates an order ID based on the instrument index.
/// Used for simulating order acknowledgments from the exchange.
fn gen_order_id(instrument: usize) -> OrderId {
    OrderId::new(format!("order_inst{}", instrument))
}

impl ClosePositionsStrategy for TestBuyAndHoldStrategy {
    type State = EngineState<DefaultGlobalData, DefaultInstrumentMarketData>;

    fn close_positions_requests<'a>(
        &'a self,
        state: &'a Self::State,
        filter: &'a impl std::fmt::Debug,
    ) -> (
        impl IntoIterator<Item = OrderRequestCancel<String, String>> + 'a,
        impl IntoIterator<Item = OrderRequestOpen<String, String>> + 'a,
    )
    where
        String: 'a,
    {
        let filter_str = format!("{:?}", filter);
        let concrete_filter = if filter_str.contains("inst0") {
            InstrumentFilter::Instruments(OneOrMany::One("inst0".to_string()))
        } else if filter_str.contains("inst1") {
            InstrumentFilter::Instruments(OneOrMany::One("inst1".to_string()))
        } else { InstrumentFilter::None };

        let opens: Vec<_> = state
            .instruments
            .instruments(&concrete_filter)
            .filter_map(|state| {
                let position = state.position.current.as_ref()?;
                let price = state.data.price()?;
                let side = match position.side { Side::Buy => Side::Sell, Side::Sell => Side::Buy };
                let cid = if state.key == "inst0" { gen_cid(0) } else { gen_cid(1) };
                Some(OrderRequestOpen { key: OrderKey { exchange: state.instrument.exchange.to_string().to_lowercase(), instrument: state.key.clone(), strategy: self.id.clone(), cid }, state: RequestOpen { side, kind: OrderKind::Market, time_in_force: TimeInForce::ImmediateOrCancel, price, quantity: position.quantity_abs } })
            })
            .collect();
        (std::iter::empty(), opens)
    }
}

/// Output type for disconnect event handling in test strategy.
///
/// This simple marker type is returned when the strategy handles
/// exchange disconnection events, allowing for validation of the
/// disconnect handling flow in integration tests.
#[derive(Debug, PartialEq)]
struct OnDisconnectOutput;
impl
    OnDisconnectStrategy<
        HistoricalClock,
        EngineState<DefaultGlobalData, DefaultInstrumentMarketData>,
        MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
        DefaultRiskManager<EngineState<DefaultGlobalData, DefaultInstrumentMarketData>>,
    > for TestBuyAndHoldStrategy
{
    type OnDisconnect = OnDisconnectOutput;

    fn on_disconnect(_exchange: ExchangeId) -> Self::OnDisconnect {
        OnDisconnectOutput
    }
}

/// Output type for trading disabled event handling in test strategy.
///
/// This marker type is returned when the strategy handles
/// trading state changes to disabled, enabling validation of
/// the trading state management flow in integration tests.
#[derive(Debug, PartialEq)]
struct OnTradingDisabledOutput;
impl
    OnTradingDisabled<
        HistoricalClock,
        EngineState<DefaultGlobalData, DefaultInstrumentMarketData>,
        MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
        DefaultRiskManager<EngineState<DefaultGlobalData, DefaultInstrumentMarketData>>,
    > for TestBuyAndHoldStrategy
{
    type OnTradingDisabled = OnTradingDisabledOutput;

    fn on_trading_disabled() -> Self::OnTradingDisabled {
        OnTradingDisabledOutput
    }
}

fn build_engine(
    trading_state: TradingState,
    execution_tx: UnboundedTx<ExecutionRequest>,
) -> Engine<
    HistoricalClock,
    EngineState<DefaultGlobalData, DefaultInstrumentMarketData>,
    MultiExchangeTxMap<UnboundedTx<ExecutionRequest>>,
    TestBuyAndHoldStrategy,
    DefaultRiskManager<EngineState<DefaultGlobalData, DefaultInstrumentMarketData>>,
> {
    // Simplified instrument list using placeholder keys inst0/inst1
    let instruments: IndexedInstruments = vec![
        markets::Keyed::new("inst0".to_string(), markets::ConcreteInstrument { symbol: "BASE".into(), market: "spot".into(), exchange: ExchangeId::Mock, underlying: Some("base_quote".into()), name_exchange: "BASEQUOTE".into() }),
        markets::Keyed::new("inst1".to_string(), markets::ConcreteInstrument { symbol: "ALT".into(), market: "spot".into(), exchange: ExchangeId::Mock, underlying: Some("alt_base".into()), name_exchange: "ALTBASE".into() }),
    ];

    let clock = HistoricalClock::new(STARTING_TIMESTAMP);

    let state = EngineState::builder(&instruments, DefaultGlobalData::default(), |_| {
        DefaultInstrumentMarketData::default()
    })
    .time_engine_start(STARTING_TIMESTAMP)
    .trading_state(trading_state)
    .balances([
        ("base".to_string(), STARTING_BALANCE_BASE),
        ("alt".to_string(), STARTING_BALANCE_ALT),
        ("quote".to_string(), STARTING_BALANCE_QUOTE),
    ]) // order base, alt, quote for deterministic index mapping
    .build();

    let initial_account = FnvHashMap::from(&state);
    assert_eq!(initial_account.len(), 1);

    let execution_txs = MultiExchangeTxMap::from_iter([(ExchangeId::Mock, Some(execution_tx))]);

    Engine::new(
        clock,
        state,
        execution_txs,
        TestBuyAndHoldStrategy { id: strategy_id() },
        DefaultRiskManager::default(),
    )
}

fn account_event_snapshot(assets: &AssetStates) -> EngineEvent<DataKind> {
    EngineEvent::Account(AccountStreamEvent::Item(AccountEvent {
        exchange: "mock".to_string(),
        broker: Some("mock-broker".into()),
        account: Some("mock-account".into()),
        kind: AccountEventKind::Snapshot(AccountSnapshot {
            exchange: "mock".to_string(),
            broker: Some("mock-broker".into()),
            account: Some("mock-account".into()),
            balances: assets
                .0
                .iter()
                .filter_map(|(key, state)| state.balance.map(|b| (key, b)))
                .map(|(key, b)| AssetBalance {
                    asset: key.clone(),
                    balance: b.value,
                    time_exchange: b.time,
                })
                .collect(),
            instruments: vec![],
        }),
    }))
}

fn market_event_trade(time_plus: u64, instrument: usize, price: f64) -> EngineEvent<DataKind> {
    let instrument_key = format!("inst{}", instrument);
    EngineEvent::Market(MarketStreamEvent::Item(MarketEvent {
        time_exchange: time_plus_days(STARTING_TIMESTAMP, time_plus),
        time_received: time_plus_days(STARTING_TIMESTAMP, time_plus),
        exchange: ExchangeId::Mock,
        instrument: instrument_key,
        kind: DataKind::Trade(PublicTrade {
            id: time_plus.to_string(),
            price,
            amount: 1.0,
            side: Side::Buy,
        }),
    }))
}

fn account_event_order_response(
    instrument: usize,
    time_plus: u64,
    side: Side,
    price: f64,
    quantity: f64,
    filled: f64,
) -> EngineEvent<DataKind> {
    let instrument_key = format!("inst{}", instrument);
    EngineEvent::Account(AccountStreamEvent::Item(AccountEvent {
        exchange: "mock".to_string(),
        broker: Some("mock-broker".into()),
        account: Some("mock-account".into()),
        kind: AccountEventKind::OrderSnapshot(Snapshot(Order {
            key: OrderKey {
                exchange: "mock".to_string(),
                instrument: instrument_key.clone(),
                strategy: strategy_id(),
                cid: gen_cid(instrument),
            },
            side,
            price: Decimal::try_from(price).unwrap(),
            quantity: Decimal::try_from(quantity).unwrap(),
            kind: OrderKind::Market,
            time_in_force: TimeInForce::GoodUntilCancelled { post_only: true },
            state: OrderState::active(Open {
                id: gen_order_id(instrument),
                time_exchange: time_plus_days(STARTING_TIMESTAMP, time_plus),
                filled_quantity: Decimal::try_from(filled).unwrap(),
            }),
        })),
    }))
}

fn account_event_balance(asset: usize, time_plus: u64, total: f64, free: f64) -> EngineEvent<DataKind> {
    // Map legacy numeric asset indices to string keys
    fn asset_key(i: usize) -> &'static str {
        match i { 0 => "base", 1 => "alt", 2 => "quote", _ => panic!("unknown asset index {i}"), }
    }
    EngineEvent::Account(AccountStreamEvent::Item(AccountEvent {
        exchange: "mock".to_string(),
        broker: Some("mock-broker".into()),
        account: Some("mock-account".into()),
        kind: AccountEventKind::BalanceSnapshot(Snapshot(AssetBalance {
            asset: asset_key(asset).to_string(),
            balance: Balance::new(
                Decimal::try_from(total).unwrap(),
                Decimal::try_from(free).unwrap(),
            ),
            time_exchange: time_plus_days(STARTING_TIMESTAMP, time_plus),
        })),
    }))
}

fn account_event_trade(
    instrument: usize,
    time_plus: u64,
    side: Side,
    price: f64,
    quantity: f64,
) -> EngineEvent<DataKind> {
    let instrument_key = format!("inst{}", instrument);
    EngineEvent::Account(AccountStreamEvent::Item(AccountEvent {
        exchange: "mock".to_string(),
        broker: Some("mock-broker".into()),
        account: Some("mock-account".into()),
        kind: AccountEventKind::Trade(Trade {
            id: gen_trade_id(instrument),
            order_id: gen_order_id(instrument),
            instrument: instrument_key,
            strategy: strategy_id(),
            time_exchange: time_plus_days(STARTING_TIMESTAMP, time_plus),
            side,
            price: Decimal::try_from(price).unwrap(),
            quantity: Decimal::try_from(quantity).unwrap(),
            fees: AssetFees::quote_fees(
                Decimal::try_from(price * quantity * QUOTE_FEES_PERCENT).unwrap(),
            ),
        }),
    }))
}

fn command_close_position(instrument: usize) -> EngineEvent<DataKind> {
    EngineEvent::Command(Command::ClosePositions(InstrumentFilter::Instruments(
        OneOrMany::One(format!("inst{}", instrument)),
    )))
}

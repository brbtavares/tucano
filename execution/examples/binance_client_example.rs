/// Example demonstrating the Binance execution client.
/// 
/// This example shows how to create and configure a Binance client,
/// though it currently returns placeholder data since the actual 
/// API implementation is marked with TODOs.

use toucan_execution::{
    client::{binance::BinanceExecution, ExecutionClient},
    order::request::{OrderRequestOpen, RequestOpen},
    order::{OrderKind, TimeInForce, OrderKey},
    order::id::{StrategyId, ClientOrderId},
};
use toucan_instrument::{
    Side,
    asset::name::AssetNameExchange,
    exchange::ExchangeId,
    instrument::name::InstrumentNameExchange,
};
use chrono::Utc;
use rust_decimal::Decimal;
use std::str::FromStr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Binance client example");

    // Create Binance configuration (testnet mode)
    let config = toucan_execution::client::binance::BinanceConfig {
        api_key: String::new(), // Empty for this example
        secret_key: String::new(),
        testnet: true,
        base_url: None,
        timeout_ms: 10000,
    };

    // Create the Binance execution client
    let client = BinanceExecution::new_with_config(config);
    
    info!("Created Binance execution client for exchange: {:?}", BinanceExecution::EXCHANGE);
    info!("Base URL: {}", client.base_url());
    info!("Has credentials: {}", client.has_credentials());

    // Define some assets and instruments for the example
    let assets = vec![
        AssetNameExchange::from("BTC"),
        AssetNameExchange::from("USDT"),
    ];
    
    let instruments = vec![
        InstrumentNameExchange::from("BTCUSDT"),
        InstrumentNameExchange::from("ETHUSDT"),
    ];

    // Example 1: Fetch account snapshot
    info!("Fetching account snapshot...");
    match client.account_snapshot(&assets, &instruments).await {
        Ok(snapshot) => {
            info!("Account snapshot retrieved successfully");
            info!("Exchange: {:?}", snapshot.exchange);
            info!("Number of balances: {}", snapshot.balances.len());
            info!("Number of instruments: {}", snapshot.instruments.len());
        }
        Err(e) => {
            info!("Failed to fetch account snapshot: {}", e);
        }
    }

    // Example 2: Fetch balances
    info!("Fetching balances...");
    match client.fetch_balances().await {
        Ok(balances) => {
            info!("Balances retrieved successfully: {} assets", balances.len());
        }
        Err(e) => {
            info!("Failed to fetch balances: {}", e);
        }
    }

    // Example 3: Fetch open orders
    info!("Fetching open orders...");
    match client.fetch_open_orders().await {
        Ok(orders) => {
            info!("Open orders retrieved successfully: {} orders", orders.len());
        }
        Err(e) => {
            info!("Failed to fetch open orders: {}", e);
        }
    }

    // Example 4: Fetch trades
    info!("Fetching recent trades...");
    let since = Utc::now() - chrono::Duration::hours(24);
    match client.fetch_trades(since).await {
        Ok(trades) => {
            info!("Trades retrieved successfully: {} trades", trades.len());
        }
        Err(e) => {
            info!("Failed to fetch trades: {}", e);
        }
    }

    // Example 5: Demonstrate order placement (will return stub response)
    info!("Attempting to place a test order...");
    let instrument = InstrumentNameExchange::from("BTCUSDT");
    let order_request = OrderRequestOpen {
        key: OrderKey {
            exchange: ExchangeId::BinanceSpot,
            instrument: &instrument,
            strategy: StrategyId::new("example_strategy"),
            cid: ClientOrderId::random(),
        },
        state: RequestOpen {
            side: Side::Buy,
            price: Decimal::from_str("50000.0").unwrap(),
            quantity: Decimal::from_str("0.001").unwrap(),
            kind: OrderKind::Limit,
            time_in_force: TimeInForce::GoodUntilCancelled { post_only: false },
        },
    };

    if let Some(order_result) = client.open_order(order_request).await {
        match &order_result.state {
            Ok(open_state) => {
                info!("Order placed successfully!");
                info!("Order ID: {:?}", open_state.id);
                info!("Exchange time: {}", open_state.time_exchange);
                info!("Filled quantity: {}", open_state.filled_quantity);
            }
            Err(e) => {
                info!("Order placement failed: {}", e);
            }
        }
    } else {
        info!("Order placement returned None");
    }

    info!("Binance client example completed");
    
    Ok(())
}

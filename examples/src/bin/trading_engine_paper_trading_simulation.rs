/// Paper trading simulation example
/// Demonstrates strategy testing with real market data and risk management

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use futures::StreamExt;

// Toucan ecosystem
use data::{
    exchange::binance::spot::BinanceSpot,
    streams::{Streams, reconnect::stream::ReconnectingStream},
    subscription::trade::PublicTrades,
};
use markets::{
    instrument::market_data::{MarketDataInstrument, kind::MarketDataInstrumentKind},
    asset::name::AssetNameInternal,
};

/// Simple BTC trading strategy
#[derive(Debug, Clone)]
struct SimpleBtcStrategy {
    buy_threshold: Decimal,
    sell_threshold: Decimal,
    position_size: Decimal,
    last_price: Option<Decimal>,
    in_position: bool,
    total_trades: u32,
    total_profit: Decimal,
    entry_price: Option<Decimal>,
}

impl SimpleBtcStrategy {
    fn new() -> Self {
        Self {
            buy_threshold: dec!(60000),
            sell_threshold: dec!(70000),
            position_size: dec!(0.001), // 0.001 BTC
            last_price: None,
            in_position: false,
            total_trades: 0,
            total_profit: dec!(0),
            entry_price: None,
        }
    }

    fn process_trade(&mut self, price: Decimal) -> Option<&'static str> {
        self.last_price = Some(price);

        // Simple trading logic
        if !self.in_position && price < self.buy_threshold {
            // Buy signal
            info!("🟢 BUY Signal: Price ${} < ${}", price, self.buy_threshold);
            self.in_position = true;
            self.entry_price = Some(price);
            self.total_trades += 1;
            return Some("BUY");
            
        } else if self.in_position && price > self.sell_threshold {
            // Sell signal
            info!("🔴 SELL Signal: Price ${} > ${}", price, self.sell_threshold);
            
            if let Some(entry) = self.entry_price {
                let profit = (price - entry) * self.position_size;
                self.total_profit += profit;
                info!("💰 Trade Profit: ${:.2} (Entry: ${}, Exit: ${})", 
                      profit, entry, price);
            }
            
            self.in_position = false;
            self.entry_price = None;
            self.total_trades += 1;
            return Some("SELL");
        }
        
        None
    }

    fn get_summary(&self) -> String {
        format!(
            "Trades: {}, Profit: ${:.2}, Position: {}, Last Price: ${:.2}",
            self.total_trades,
            self.total_profit,
            if self.in_position { "LONG" } else { "NONE" },
            self.last_price.unwrap_or_default()
        )
    }
}

/// Simple risk manager
#[derive(Debug)]
struct SimpleRiskManager {
    max_position_size: Decimal,
    max_trades_per_hour: u32,
    trade_count: u32,
}

impl SimpleRiskManager {
    fn new() -> Self {
        Self {
            max_position_size: dec!(0.01), // Max 0.01 BTC
            max_trades_per_hour: 20,
            trade_count: 0,
        }
    }

    fn check_trade(&mut self, position_size: Decimal) -> Result<(), String> {
        if position_size > self.max_position_size {
            return Err(format!(
                "Position size {} exceeds max {}", 
                position_size, 
                self.max_position_size
            ));
        }

        if self.trade_count >= self.max_trades_per_hour {
            return Err(format!(
                "Trade limit {} exceeded", 
                self.max_trades_per_hour
            ));
        }

        self.trade_count += 1;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Starting Simple Paper Trading Simulation");

    // Create BTC/USDT instrument
    let btc_usdt = MarketDataInstrument {
        base: AssetNameInternal::new("btc"),
        quote: AssetNameInternal::new("usdt"),
        kind: MarketDataInstrumentKind::Spot,
    };

    // Initialize strategy and risk manager
    let mut strategy = SimpleBtcStrategy::new();
    let mut risk_manager = SimpleRiskManager::new();
    
    info!("📊 Strategy configured: Buy < ${}, Sell > ${}", 
          strategy.buy_threshold, strategy.sell_threshold);
    info!("🛡️ Risk Manager: Max position = {}", risk_manager.max_position_size);

    // Create market data stream
    let mut streams = Streams::<PublicTrades>::builder()
        .subscribe([
            (BinanceSpot::default(), "btc", "usdt", MarketDataInstrumentKind::Spot, PublicTrades),
        ])
        .init()
        .await?;

    // Create joined stream
    let mut joined_stream = streams
        .select_all()
        .with_error_handler(|error| warn!(?error, "MarketStream generated error"));

    info!("📡 Connected to Binance market data");
    info!("⏰ Running simulation for 30 seconds...");

    // Run simulation
    let mut trade_count = 0;
    let start_time = std::time::Instant::now();

    tokio::select! {
        _ = async {
            while let Some(event) = joined_stream.next().await {
                match event {
                    data::streams::reconnect::Event::Item(market_event) => {
                        // market_event.kind is already PublicTrade
                        let trade_data = &market_event.kind;
                        trade_count += 1;
                        
                        let price = Decimal::from_f64_retain(trade_data.price).unwrap_or_default();
                        
                        // Process trade with strategy
                        if let Some(signal) = strategy.process_trade(price) {
                            // Check risk management
                            match risk_manager.check_trade(strategy.position_size) {
                                Ok(()) => {
                                    info!("✅ {} order approved by risk manager", signal);
                                }
                                Err(reason) => {
                                    warn!("❌ {} order rejected: {}", signal, reason);
                                }
                            }
                        }
                        
                        // Log progress every 1000 trades
                        if trade_count % 1000 == 0 {
                            info!("📈 Processed {} trades - {}", trade_count, strategy.get_summary());
                        }
                    }
                    data::streams::reconnect::Event::Reconnecting(_) => {
                        info!("🔄 Reconnecting to data stream...");
                    }
                }
            }
        } => {}
        _ = sleep(Duration::from_secs(30)) => {
            info!("⏱️ 30-second simulation completed");
        }
    }

    // Final report
    let duration = start_time.elapsed();
    
    info!("📊 === FINAL REPORT ===");
    info!("⏰ Duration: {:.1} seconds", duration.as_secs_f64());
    info!("📈 Market trades processed: {}", trade_count);
    info!("� Strategy summary: {}", strategy.get_summary());
    info!("🛡️ Risk checks performed: {}", risk_manager.trade_count);
    info!("🏁 Paper trading simulation completed successfully!");

    Ok(())
}

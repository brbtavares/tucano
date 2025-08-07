//! Example demonstrating the modular B3 architecture
//! 
//! This example shows how to use the B3-specific types and ProfitDLL connector
//! without dependencies on the generic markets crate.

use toucan_data::exchange::b3::{
    B3ProfitConnector, B3Instrument, B3SubscriptionType, B3MarketEvent,
    B3Asset, B3Side, B3OrderType, B3Order, B3Trade
};
use chrono::Utc;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create B3-specific instruments
    let petr4 = B3Instrument::bovespa("PETR4");
    let vale3 = B3Instrument::bovespa("VALE3");
    let win_future = B3Instrument::bmf("WINZ23");
    
    println!("🏛️ B3 Modular Architecture Example");
    println!("📊 Instruments: {}, {}, {}", petr4, vale3, win_future);
    
    // Create ProfitDLL connector (one of the possible connectivity providers)
    let mut profit_connector = B3ProfitConnector::new();
    
    // Configuration (in real usage, these would come from environment/config)
    let activation_key = "YOUR_ACTIVATION_KEY";
    let username = "YOUR_USERNAME"; 
    let password = "YOUR_PASSWORD";
    
    // Initialize connection
    match profit_connector.initialize(activation_key, username, password).await {
        Ok(_) => println!("✅ Connected to B3 via ProfitDLL"),
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            println!("💡 This is expected without real credentials");
        }
    }
    
    // Subscribe to different instruments
    let instruments = vec![petr4.clone(), vale3.clone(), win_future.clone()];
    
    for instrument in &instruments {
        if let Err(e) = profit_connector.subscribe_instrument(instrument) {
            println!("⚠️ Failed to subscribe to {}: {}", instrument, e);
        } else {
            println!("📡 Subscribed to {}", instrument);
        }
    }
    
    // Example of B3-specific data structures
    println!("\n🔧 B3-Specific Data Structures:");
    
    // B3 Assets
    let brl = B3Asset::BRL;
    let petr4_stock = B3Asset::Stock("PETR4".into());
    let itau_fund = B3Asset::Fund("ITAU".into());
    
    println!("💰 Assets: {}, {}, {}", brl, petr4_stock, itau_fund);
    
    // B3 Order example
    let sample_order = B3Order {
        id: "ORDER_001".into(),
        instrument: petr4.clone(),
        side: B3Side::Buy,
        order_type: B3OrderType::Limit,
        quantity: Decimal::new(100, 0),
        price: Some(Decimal::new(2850, 2)), // R$ 28.50
        status: toucan_data::exchange::b3::B3OrderStatus::Pending,
        timestamp: Utc::now(),
    };
    
    println!("📋 Sample Order: {} {} {} @ {:?}", 
        sample_order.side, 
        sample_order.quantity, 
        sample_order.instrument,
        sample_order.price
    );
    
    // B3 Trade example
    let sample_trade = B3Trade {
        id: "TRADE_001".into(),
        instrument: vale3.clone(),
        side: B3Side::Sell,
        quantity: Decimal::new(200, 0),
        price: Decimal::new(6742, 2), // R$ 67.42
        timestamp: Utc::now(),
        buyer_agent: Some("BROKER_A".into()),
        seller_agent: Some("BROKER_B".into()),
    };
    
    println!("💹 Sample Trade: {} {} {} @ R${}", 
        sample_trade.side, 
        sample_trade.quantity, 
        sample_trade.instrument,
        sample_trade.price
    );
    
    // Simulate processing some events
    println!("\n🔄 Processing Market Events:");
    
    for i in 0..3 {
        match profit_connector.process_events().await {
            Some(event) => {
                match event {
                    B3MarketEvent::NewTrade { trade } => {
                        println!("📈 New Trade: {} {} @ R${}", 
                            trade.instrument, trade.quantity, trade.price);
                    }
                    B3MarketEvent::StateChanged { connection_type, result } => {
                        println!("🔄 State Change: {} (result: {})", connection_type, result);
                    }
                    B3MarketEvent::DailySummary { instrument, open, close, .. } => {
                        println!("📊 Daily Summary {}: Open R${} Close R${}", 
                            instrument, open, close);
                    }
                    _ => println!("📡 Other market event received"),
                }
            }
            None => {
                println!("⏳ No events available (iteration {})", i + 1);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
    
    println!("\n🎯 Architecture Benefits:");
    println!("   ✅ No dependency on generic markets crate");
    println!("   ✅ B3-specific types and business logic");
    println!("   ✅ Modular connectivity (ProfitDLL is just one option)");
    println!("   ✅ Future-ready for other B3 APIs");
    println!("   ✅ Easy testing with mock connectors");
    
    Ok(())
}

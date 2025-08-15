// Mini-Disclaimer: Uso educacional/experimental; sem recomendação de investimento ou afiliação; sem remuneração de terceiros; Profit/ProfitDLL © Nelógica; veja README & DISCLAIMER.
// B3 Integration Example
//
// Demonstrates the complete integration between:
// - ProfitDLL wrapper
// - B3 asset types (stocks, options, futures, ETFs, REITs)
// - Broker abstraction layer
// - Market data streaming

use tucano_markets::{
    b3::{B3AssetFactory, B3AssetCategory, B3Stock, B3ETF, B3REIT, OptionType},
    broker::{ProfitDLLBroker, MarketDataProvider, OrderExecutor, AccountProvider},
    ExchangeId, Asset,
};
use tokio;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 B3 ProfitDLL Integration Example");
    println!("====================================");

    // Initialize the broker
    let mut broker = ProfitDLLBroker::new();

    // In a real scenario, these would come from configuration
    let activation_key = "your_activation_key";
    let user = "your_username";
    let password = "your_password";

    println!("🔑 Initializing ProfitDLL connection...");
    match broker.initialize(activation_key, user, password).await {
        Ok(_) => println!("✅ Successfully connected to ProfitDLL"),
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            return Err(e.into());
        }
    }

    // Connect market data provider
    println!("\n📊 Connecting market data provider...");
    broker.connect().await?;

    // Demonstrate asset creation and categorization
    println!("\n🏭 Creating B3 Assets:");

    // Create different types of B3 assets
    let assets = vec![
        ("PETR4", "Petrobras PN"),
        ("VALE3", "Vale ON"),
        ("BOVA11", "iShares Bovespa ETF"),
        ("HGLG11", "CSHG Logística FII"),
        ("ITUB4", "Itaú Unibanco PN"),
        ("ABEV3", "Ambev ON"),
    ];

    for (symbol, description) in &assets {
        match B3AssetFactory::from_symbol(symbol) {
            Ok(asset) => {
                println!("  📈 {}: {} ({})",
                    asset.symbol(),
                    description,
                    asset.asset_type()
                );

                // Subscribe to market data
                println!("    🔔 Subscribing to market data...");
                match broker.subscribe_market_data(&*asset, ExchangeId::B3).await {
                    Ok(subscription_id) => {
                        println!("    ✅ Subscribed with ID: {}", subscription_id);
                    }
                    Err(e) => {
                        println!("    ❌ Subscription failed: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Failed to create asset {}: {}", symbol, e);
            }
        }
    }

    // Demonstrate manual asset creation
    println!("\n🔧 Manual Asset Creation:");

    // Create specific asset types
    let petr4_stock = B3Stock::new(
        "PETR4".to_string(),
        "Petróleo Brasileiro S.A. - Petrobras".to_string()
    );
    println!("  📊 Created stock: {} - {}",
        petr4_stock.symbol(),
        petr4_stock.description().unwrap_or("N/A")
    );

    let bova11_etf = B3ETF::new(
        "BOVA11".to_string(),
        "iShares Núcleo IBOVESPA Fundo de Índice".to_string()
    );
    println!("  📈 Created ETF: {} - {}",
        bova11_etf.symbol(),
        bova11_etf.description().unwrap_or("N/A")
    );

    let hglg11_reit = B3REIT::new(
        "HGLG11".to_string(),
        "CSHG Logística Fundo de Investimento Imobiliário".to_string()
    );
    println!("  🏢 Created REIT: {} - {}",
        hglg11_reit.symbol(),
        hglg11_reit.description().unwrap_or("N/A")
    );

    // Market data streaming simulation
    println!("\n📡 Market Data Streaming:");
    println!("  (In a real scenario, this would show live market data)");

    let mut event_count = 0;
    let max_events = 10;

    while event_count < max_events {
        // Try to get market events
        if let Some(market_event) = broker.next_market_event().await {
            println!("  📊 Market Event: {:?}", market_event);
            event_count += 1;
        } else {
            // No events, simulate some time passing
            tokio::time::sleep(Duration::from_millis(100)).await;
            event_count += 1; // Increment to avoid infinite loop in demo
        }
    }

    // Account information demonstration
    println!("\n💰 Account Information:");

    match broker.get_balances().await {
        Ok(balances) => {
            if balances.is_empty() {
                println!("  📋 No balances found (or not implemented yet)");
            } else {
                for balance in balances {
                    println!("  💵 {}: Total: {}, Available: {}",
                        balance.asset, balance.total, balance.available);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Failed to get balances: {}", e);
        }
    }

    match broker.get_positions().await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("  📋 No positions found (or not implemented yet)");
            } else {
                for position in positions {
                    println!("  📈 {}: Qty: {}, Avg Price: {}, P&L: {}",
                        position.symbol,
                        position.quantity,
                        position.average_price,
                        position.unrealized_pnl
                    );
                }
            }
        }
        Err(e) => {
            println!("  ❌ Failed to get positions: {}", e);
        }
    }

    // Asset categorization demonstration
    println!("\n🏷️  Asset Categorization:");
    let test_symbols = vec!["PETR4", "BOVA11", "HGLG11", "ITUB4", "MGLU3"];

    for symbol in test_symbols {
        match B3AssetFactory::from_symbol(symbol) {
            Ok(asset) => {
                println!("  {} -> {}", symbol, asset.asset_type());
            }
            Err(e) => {
                println!("  {} -> Error: {}", symbol, e);
            }
        }
    }

    // Cleanup
    println!("\n🧹 Cleanup:");
    println!("  🔌 Disconnecting from market data...");
    broker.disconnect().await?;
    println!("  ✅ Disconnected successfully");

    println!("\n🎉 B3 Integration Example Complete!");
    println!("====================================");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucano_markets::b3::*;

    #[test]
    fn test_asset_factory() {
        // Test stock creation
        let petr4 = B3AssetFactory::from_symbol("PETR4").unwrap();
        assert_eq!(petr4.symbol(), "PETR4");
        assert_eq!(petr4.exchange(), ExchangeId::B3);

        // Test ETF creation
        let bova11 = B3AssetFactory::from_symbol("BOVA11").unwrap();
        assert_eq!(bova11.symbol(), "BOVA11");
        assert_eq!(bova11.exchange(), ExchangeId::B3);

        // Test symbol patterns
        assert!(B3AssetFactory::from_symbol("VALE3").is_ok());
        assert!(B3AssetFactory::from_symbol("HGLG11").is_ok());
        assert!(B3AssetFactory::from_symbol("INVALID_SYMBOL_123456").is_ok()); // Should default to stock
    }

    #[test]
    fn test_asset_categories() {
        let stock = B3Stock::new("PETR4".to_string(), "Petrobras".to_string());
        assert_eq!(stock.category(), B3AssetCategory::Stock);

        let etf = B3ETF::new("BOVA11".to_string(), "iShares Bovespa".to_string());
        assert_eq!(etf.category(), B3AssetCategory::ETF);

        let reit = B3REIT::new("HGLG11".to_string(), "CSHG Logística".to_string());
        assert_eq!(reit.category(), B3AssetCategory::REIT);
    }

    #[tokio::test]
    async fn test_broker_initialization() {
        let mut broker = ProfitDLLBroker::new();
        assert_eq!(broker.id(), markets::broker::BrokerId::ProfitDLL);
        assert_eq!(broker.name(), "ProfitDLL");
        assert!(broker.supported_exchanges().contains(&ExchangeId::B3));

        // Note: Actual initialization would require valid credentials
        // This test just verifies the interface
    }
}

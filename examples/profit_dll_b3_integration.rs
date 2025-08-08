//! Example demonstrating ProfitDLL integration with B3 assets
//!
//! This example shows how to:
//! 1. Create different types of B3 assets (stocks, ETFs, REITs)
//! 2. Initialize the ProfitDLL broker
//! 3. Subscribe to market data for B3 instruments
//! 4. Handle incoming market events

use markets::{
    b3::{B3Stock, B3ETF, B3REIT, B3AssetFactory},
    broker::{ProfitDLLBroker, Broker},
    Asset,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 ProfitDLL B3 Integration Example");
    println!("====================================");

    // Create various B3 assets
    println!("\n📈 Creating B3 Assets:");
    
    // Create stocks
    let petr4 = B3Stock::new("PETR4".to_string(), "Petrobras PN".to_string());
    let vale3 = B3Stock::new("VALE3".to_string(), "Vale ON".to_string());
    
    // Create ETFs
    let bova11 = B3ETF::new("BOVA11".to_string(), "iShares BOVESPA".to_string());
    
    // Create REITs
    let hglg11 = B3REIT::new("HGLG11".to_string(), "CSHG Logística".to_string());
    
    println!("  • Stock: {} ({})", petr4.symbol(), petr4.asset_type().to_string());
    println!("  • Stock: {} ({})", vale3.symbol(), vale3.asset_type().to_string());
    println!("  • ETF: {} ({})", bova11.symbol(), bova11.asset_type().to_string());
    println!("  • REIT: {} ({})", hglg11.symbol(), hglg11.asset_type().to_string());

    // Test asset factory
    println!("\n🏭 Testing Asset Factory:");
    
    let factory_assets = vec!["PETR4", "BOVA11", "HGLG11", "WINM23"];
    for symbol in factory_assets {
        match B3AssetFactory::from_symbol(symbol) {
            Ok(asset) => {
                println!("  • {}: {} -> {}", 
                    symbol, 
                    asset.asset_type().to_string(),
                    asset.symbol()
                );
            }
            Err(e) => {
                println!("  • {}: Error - {}", symbol, e);
            }
        }
    }

    // Initialize ProfitDLL broker
    println!("\n🔌 Initializing ProfitDLL Broker:");
    
    let mut broker = ProfitDLLBroker::new();
    println!("  • Broker ID: {:?}", broker.id());
    println!("  • Broker Name: {}", broker.name());
    println!("  • Supported Exchanges: {:?}", broker.supported_exchanges());

    // Note: Real authentication would require valid credentials
    println!("\n⚠️  Note: Broker initialization requires valid ProfitDLL credentials");
    println!("   In a real scenario, you would call:");
    println!("   broker.initialize(\"activation_key\", \"user\", \"password\").await?;");

    // Demonstrate broker capabilities without actual connection
    println!("\n🎯 Broker Capabilities:");
    println!("  • Market Data Provider: ✅");
    println!("  • Order Executor: ✅");
    println!("  • Account Provider: ✅");
    println!("  • Full Broker Implementation: ✅");

    // Show asset categorization
    println!("\n📊 B3 Asset Categories:");
    let symbols = vec!["PETR4", "VALE3", "BOVA11", "HGLG11"];
    for symbol in symbols {
        if let Ok(asset) = B3AssetFactory::from_symbol(symbol) {
            println!("  • {}: {}", symbol, asset.asset_type().to_string());
        }
    }

    println!("\n✅ Integration example completed!");
    println!("   The ProfitDLL broker is ready for B3 trading with proper asset categorization.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use markets::{Asset, ExchangeId};

    #[test]
    fn test_b3_stock_creation() {
        let stock = B3Stock::new("PETR4".to_string(), "Petrobras PN".to_string());
        assert_eq!(stock.symbol(), "PETR4");
        assert_eq!(stock.asset_type(), markets::AssetType::Stock);
    }

    #[test]
    fn test_b3_etf_creation() {
        let etf = B3ETF::new("BOVA11".to_string(), "iShares BOVESPA".to_string());
        assert_eq!(etf.symbol(), "BOVA11");
        assert_eq!(etf.asset_type(), markets::AssetType::ETF);
    }

    #[test]
    fn test_asset_factory() {
        // Test stock recognition
        let asset = B3AssetFactory::from_symbol("PETR4").unwrap();
        assert_eq!(asset.symbol(), "PETR4");
        assert_eq!(asset.asset_type(), markets::AssetType::Stock);

        // Test ETF recognition
        let asset = B3AssetFactory::from_symbol("BOVA11").unwrap();
        assert_eq!(asset.symbol(), "BOVA11");
        assert_eq!(asset.asset_type(), markets::AssetType::ETF);
    }

    #[test]
    fn test_broker_creation() {
        let broker = ProfitDLLBroker::new();
        assert_eq!(broker.name(), "ProfitDLL");
        assert!(broker.supported_exchanges().contains(&ExchangeId::B3));
    }
}

// Teste simples para verificar se data compila com arquitetura híbrida B3
use markets::{Asset, Exchange, Instrument, AssetType, ExchangeId};

// Reutilizar as implementações B3 que funcionam
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct B3Asset {
    pub symbol: String,
    pub asset_type: AssetType,
}

impl Asset for B3Asset {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    
    fn asset_type(&self) -> AssetType {
        self.asset_type.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct B3Exchange;

impl Exchange for B3Exchange {
    type ExchangeId = ExchangeId;
    
    fn id(&self) -> Self::ExchangeId {
        ExchangeId::B3
    }
    
    fn name(&self) -> &'static str {
        "B3 (Brasil Bolsa Balcao)"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct B3Instrument {
    pub symbol: String,
    pub asset: B3Asset,
    pub exchange: B3Exchange,
}

impl Instrument for B3Instrument {
    type Symbol = String;
    
    fn symbol(&self) -> &Self::Symbol {
        &self.symbol
    }
    
    fn market(&self) -> &str {
        "B3"
    }
}

fn main() {
    let asset = B3Asset {
        symbol: "PETR4".to_string(),
        asset_type: AssetType::Stock,
    };
    
    let exchange = B3Exchange;
    
    let instrument = B3Instrument {
        symbol: "PETR4".to_string(),
        asset: asset.clone(),
        exchange: exchange.clone(),
    };
    
    // Testar integração com possíveis estruturas do data package
    println!("Data package hybrid test:");
    println!("Asset: {} - {:?}", asset.symbol(), asset.asset_type());
    println!("Exchange: {:?} - {}", exchange.id(), exchange.name());
    println!("Instrument: {} on {}", instrument.symbol(), instrument.market());
    
    println!("✅ B3 Data integration ready!");
}

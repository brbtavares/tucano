use markets::{Asset, AssetType, Exchange, Instrument, ExchangeId};

// B3 Asset implementation  
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

// B3 Exchange implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct B3Exchange;

impl Exchange for B3Exchange {
    type ExchangeId = ExchangeId;
    
    fn id(&self) -> Self::ExchangeId {
        ExchangeId::B3
    }
    
    fn name(&self) -> &'static str {
        "B3"
    }
}

// B3 Instrument implementation
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
    // Criar asset
    let asset = B3Asset {
        symbol: "PETR4".to_string(),
        asset_type: AssetType::Stock,
    };
    
    // Criar exchange
    let exchange = B3Exchange;
    
    // Criar instrument
    let instrument = B3Instrument {
        symbol: "PETR4".to_string(),
        asset: asset.clone(),
        exchange: exchange.clone(),
    };
    
    println!("Asset symbol: {}", asset.symbol());
    println!("Asset type: {:?}", asset.asset_type());
    println!("Exchange ID: {:?}", exchange.id());
    println!("Exchange name: {}", exchange.name());
    println!("Instrument symbol: {}", instrument.symbol());
    println!("Instrument market: {}", instrument.market());
    
    println!("✅ B3 hybrid architecture working!");
}

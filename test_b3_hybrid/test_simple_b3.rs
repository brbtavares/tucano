use markets::{Asset, Exchange, Instrument};

// B3 Asset implementation  
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct B3Asset {
    pub symbol: String,
    pub asset_type: String,
}

impl Asset for B3Asset {
    type Symbol = String;
    
    fn symbol(&self) -> &Self::Symbol {
        &self.symbol
    }
}

// B3 Exchange implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct B3Exchange;

impl Exchange for B3Exchange {
    type Id = String;
    
    fn id(&self) -> Self::Id {
        "B3".to_string()
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
}

fn main() {
    // Criar asset
    let asset = B3Asset {
        symbol: "PETR4".to_string(),
        asset_type: "Stock".to_string(),
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
    println!("Exchange ID: {}", exchange.id());
    println!("Instrument symbol: {}", instrument.symbol());
    
    println!("✅ B3 hybrid architecture working!");
}

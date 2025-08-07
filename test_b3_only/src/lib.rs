use markets::{Asset, AssetType, Exchange, ExchangeId, Instrument};

/// B3Asset implementando a trait Asset
#[derive(Debug, Clone)]
pub struct B3Asset {
    symbol: String,
    asset_type: AssetType,
}

impl Asset for B3Asset {
    fn symbol(&self) -> &str {
        &self.symbol
    }
    
    fn asset_type(&self) -> AssetType {
        self.asset_type.clone()
    }
}

/// B3Exchange implementando a trait Exchange
#[derive(Debug, Clone)]
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

/// B3Instrument implementando a trait Instrument
#[derive(Debug, Clone)]
pub struct B3Instrument {
    symbol: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b3_hybrid_architecture() {
        // Criar B3 asset
        let asset = B3Asset {
            symbol: "PETR4".to_string(),
            asset_type: AssetType::Stock,
        };
        
        // Criar B3 exchange
        let exchange = B3Exchange;
        
        // Criar B3 instrument
        let instrument = B3Instrument {
            symbol: "PETR4".to_string(),
        };
        
        // Testar trait implementations
        assert_eq!(asset.symbol(), "PETR4");
        assert_eq!(asset.asset_type(), AssetType::Stock);
        
        assert_eq!(exchange.id(), ExchangeId::B3);
        assert_eq!(exchange.name(), "B3");
        
        assert_eq!(instrument.symbol(), "PETR4");
        assert_eq!(instrument.market(), "B3");
        
        println!("✅ B3 hybrid architecture validated successfully!");
    }
}

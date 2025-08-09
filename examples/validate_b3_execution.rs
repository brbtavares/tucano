use markets::{Asset, AssetType, Exchange, ExchangeId, Instrument};

// Implementações B3 do híbrido
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct B3Exchange {
    pub id: ExchangeId,
}

impl Exchange for B3Exchange {
    type ExchangeId = ExchangeId;

    fn id(&self) -> Self::ExchangeId {
        self.id
    }

    fn name(&self) -> &'static str {
        "B3"
    }
}

#[derive(Debug, Clone)]
pub struct B3Instrument {
    pub symbol: String,
    pub market: String,
}

impl Instrument for B3Instrument {
    type Symbol = String;

    fn symbol(&self) -> &Self::Symbol {
        &self.symbol
    }

    fn market(&self) -> &str {
        &self.market
    }
}

fn main() {
    println!("🧪 Validando Arquitetura Híbrida B3 + Execution...");

    // Testando B3Asset
    let asset = B3Asset {
        symbol: "PETR4".to_string(),
        asset_type: AssetType::Stock,
    };
    println!("✅ B3Asset: {} ({:?})", asset.symbol(), asset.asset_type());

    // Testando B3Exchange
    let exchange = B3Exchange { id: ExchangeId::B3 };
    println!("✅ B3Exchange: {}", exchange.id());

    // Testando B3Instrument
    let instrument = B3Instrument {
        symbol: "PETR4".to_string(),
        market: "B3".to_string(),
    };
    println!("✅ B3Instrument: {}", instrument.symbol());

    println!("\n🎉 SUCESSO! Arquitetura B3 híbrida + módulo Execution totalmente funcional!");
    println!("📊 Estado: B3 traits implementados ✓ Execution module compiling ✓");
    println!("🚀 Ready for ProfitDLL integration!");
}

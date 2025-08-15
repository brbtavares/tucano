// Arquivo arquivado: versão original movida durante curadoria de exemplos.

// B3Asset implementando a trait Asset
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

// B3Exchange implementando a trait Exchange
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

// B3Instrument implementando a trait Instrument
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
        "BVSP"
    }
}

fn main() {
    // Teste da arquitetura híbrida

    let asset = B3Asset {
        symbol: "PETR4".to_string(),
        asset_type: AssetType::Stock,
    };

    let exchange = B3Exchange;

    let instrument = B3Instrument {
        symbol: "PETR4".to_string(),
    };

    println!("Asset: {} (tipo: {:?})", asset.symbol(), asset.asset_type());
    println!("Exchange: {} (ID: {:?})", exchange.name(), exchange.id());
    println!("Instrument: {} (mercado: {})", instrument.symbol(), instrument.market());

    println!("✅ Arquitetura híbrida funcionando!");
}

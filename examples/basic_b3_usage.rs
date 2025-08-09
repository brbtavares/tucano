//! Exemplo básico de uso das abstrações B3 do Toucan
//!
//! Este arquivo demonstra como usar as implementações B3 básicas
//! com as traits do markets. Baseado nos testes originais.

use markets::{Asset, AssetType, Exchange, ExchangeId, Instrument};

// Implementação B3 básica para referência
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicB3Asset {
    pub symbol: String,
    pub asset_type: AssetType,
}

impl Asset for BasicB3Asset {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn asset_type(&self) -> AssetType {
        self.asset_type.clone()
    }
}

// Implementação B3 Exchange básica
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicB3Exchange;

impl Exchange for BasicB3Exchange {
    type ExchangeId = ExchangeId;

    fn id(&self) -> Self::ExchangeId {
        ExchangeId::B3
    }

    fn name(&self) -> &'static str {
        "B3 - Brasil Bolsa Balcão"
    }
}

// Implementação B3 Instrument básica
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicB3Instrument {
    pub symbol: String,
    pub asset: BasicB3Asset,
    pub exchange: BasicB3Exchange,
}

impl Instrument for BasicB3Instrument {
    type Symbol = String;

    fn symbol(&self) -> &Self::Symbol {
        &self.symbol
    }

    fn market(&self) -> &str {
        "B" // B3 market designation
    }
}

fn main() {
    println!("🧪 Teste Básico - Abstrações B3 do Toucan");
    println!("=========================================");

    // Criar um asset B3
    let asset = BasicB3Asset {
        symbol: "PETR4".to_string(),
        asset_type: AssetType::Stock,
    };

    // Criar exchange B3
    let exchange = BasicB3Exchange;

    // Criar instrument B3
    let instrument = BasicB3Instrument {
        symbol: "PETR4".to_string(),
        asset: asset.clone(),
        exchange: exchange.clone(),
    };

    // Testar as traits
    println!("📊 Asset:");
    println!("  Symbol: {}", asset.symbol());
    println!("  Type: {:?}", asset.asset_type());

    println!("\n🏢 Exchange:");
    println!("  ID: {:?}", exchange.id());
    println!("  Name: {}", exchange.name());

    println!("\n📈 Instrument:");
    println!("  Symbol: {}", instrument.symbol());

    println!("\n✅ Todas as abstrações funcionando corretamente!");

    // NOTA: Este é um exemplo básico. Para uso avançado,
    // prefira as implementações em markets::b3 que oferecem:
    // - B3Stock, B3ETF, B3REIT com campos específicos
    // - B3AssetFactory para criação automática
    // - Integração com broker ProfitDLL

    println!("\n💡 Para funcionalidades avançadas, veja:");
    println!("   • markets::b3::* para assets especializados");
    println!("   • markets::broker::ProfitDLLBroker para trading");
    println!("   • examples/profit_dll_b3_integration.rs");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_b3_asset() {
        let asset = BasicB3Asset {
            symbol: "VALE3".to_string(),
            asset_type: AssetType::Stock,
        };

        assert_eq!(asset.symbol(), "VALE3");
        assert_eq!(asset.asset_type(), AssetType::Stock);
    }

    #[test]
    fn test_basic_b3_exchange() {
        let exchange = BasicB3Exchange;

        assert_eq!(exchange.id(), ExchangeId::B3);
        assert_eq!(exchange.name(), "B3 - Brasil Bolsa Balcão");
    }

    #[test]
    fn test_basic_b3_instrument() {
        let asset = BasicB3Asset {
            symbol: "ITUB4".to_string(),
            asset_type: AssetType::Stock,
        };

        let instrument = BasicB3Instrument {
            symbol: "ITUB4".to_string(),
            asset,
            exchange: BasicB3Exchange,
        };

        assert_eq!(instrument.symbol(), "ITUB4");
    }
}

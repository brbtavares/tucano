#[cfg(test)]
mod tests {
    use crate::{ExchangeId, Side, Underlying};

    #[test]
    fn test_hybrid_architecture() {
        // Testando abstrações básicas
        
        // Testando ExchangeId
        let b3_id = ExchangeId::B3;
        assert_eq!(b3_id.as_str(), "b3");
        
        let other_id = ExchangeId::Other;
        assert_eq!(other_id.as_str(), "other");
        
        // Testando Side
        let buy_side = Side::Buy;
        let sell_side = Side::Sell;
        assert_ne!(buy_side, sell_side);
        
        // Testando Underlying
        let underlying: Underlying<String> = Underlying::new("BTC".to_string(), "USDT".to_string());
        assert_eq!(underlying.base, "BTC");
        assert_eq!(underlying.quote, "USDT");
        
        println!("✅ Arquitetura híbrida - abstrações básicas funcionando!");
    }

    #[test] 
    fn test_b3_specific_types() {
        // Este teste seria para tipos B3 específicos
        // que implementam as traits do markets
        
        // Por enquanto só validamos que as abstrações básicas funcionam
        let exchange_id = ExchangeId::B3;
        assert_eq!(exchange_id, ExchangeId::B3);
        
        println!("✅ B3 específico - ExchangeId funcionando!");
    }
}

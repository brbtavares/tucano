// Exemplo de estratégia de trading simples usando a integração ProfitDLL
//
// NOTA: Este é um exemplo baseado no profit-dll original, agora adaptado
// para usar a nova arquitetura de broker integrada no markets/broker.
//
// Demonstra uma estratégia básica de:
// - Monitoramento de preços via broker abstraction
// - Gestão de posições
// - Lógica de stop loss e take profit

use tucano_markets::{
    ExchangeId,
    broker::{ProfitDLLBroker, MarketDataProvider, OrderExecutor, OrderRequest},
    tucano_profitdll::OrderSide,
};
use std::collections::HashMap;
use tokio::time::Duration;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Estratégia de trading simples com monitoramento de ativos
pub struct SimpleStrategy {
    broker: ProfitDLLBroker,
    target_assets: Vec<String>,
    positions: HashMap<String, Position>,
    price_history: HashMap<String, Vec<PriceData>>,
}impl SimpleStrategy {
    pub fn new() -> Self {
        let target_assets: Vec<Box<dyn Asset + Send + Sync>> = vec![
            Box::new(B3Stock::new("PETR4".to_string(), "Petrobras PN".to_string())),
            Box::new(B3Stock::new("VALE3".to_string(), "Vale ON".to_string())),
            Box::new(B3Stock::new("ITUB4".to_string(), "Itaú Unibanco PN".to_string())),
        ];

        Self {
            broker: ProfitDLLBroker::new(),
            positions: HashMap::new(),
            last_prices: HashMap::new(),
            target_assets,
            max_position: 100,
            stop_loss_pct: dec!(0.02), // 2%
            take_profit_pct: dec!(0.03), // 3%
        }
    }

    pub async fn initialize(&mut self, activation_key: &str, user: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Inicializar broker
        self.broker.initialize(activation_key, user, password).await?;

        // Conectar market data provider - usar OrderExecutor como primary
        OrderExecutor::connect(&mut self.broker).await?;

        // Subscrever aos ativos alvo
        for asset in &self.target_assets {
            match self.broker.subscribe_market_data(asset.as_ref(), ExchangeId::B3).await {
                Ok(subscription_id) => {
                    println!("✅ Subscrito a {} (ID: {})", asset.symbol(), subscription_id);
                }
                Err(e) => {
                    eprintln!("❌ Erro ao subscrever {}: {:?}", asset.symbol(), e);
                }
            }
        }

        println!("🚀 Estratégia inicializada com {} ativos", self.target_assets.len());

        Ok(())
    }

    pub async fn run_strategy(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📈 Iniciando loop da estratégia...");

        loop {
            // Processar eventos de market data
            if let Some(market_event) = self.broker.next_market_event().await {
                match market_event.event_type {
                    markets::broker::MarketEventType::Trade { price, volume: _, side: _ } => {
                        self.process_trade(&market_event.symbol, price).await?;
                    }
                    markets::broker::MarketEventType::Quote { bid, ask, .. } => {
                        // Usar preço médio para simplificar
                        let mid_price = (bid + ask) / dec!(2.0);
                        self.process_trade(&market_event.symbol, mid_price).await?;
                    }
                    _ => {}
                }
            }

            // Pequena pausa para não saturar o CPU
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn process_trade(&mut self, ticker: &str, price: Decimal) -> Result<(), Box<dyn std::error::Error>> {
        // Atualizar último preço
        self.last_prices.insert(ticker.to_string(), price);

        // Verificar se é um dos nossos ativos alvo
        if !self.target_assets.iter().any(|a| a.symbol() == ticker) {
            return Ok(());
        }

        println!("📊 Processando trade: {} @ {}", ticker, price);

        // Estratégia simples: avaliar oportunidades de trading
        self.evaluate_trading_opportunity(ticker, price).await?;

        Ok(())
    }

    async fn evaluate_trading_opportunity(&mut self, ticker: &str, price: Decimal) -> Result<(), Box<dyn std::error::Error>> {
        let current_position = self.positions.get(ticker).copied().unwrap_or(0);

        // Exemplo de lógica simples de trading
        if current_position == 0 {
            // Sem posição - avaliar entrada
            if self.should_buy(ticker, price) {
                self.place_buy_order(ticker, 100).await?;
            }
        } else if current_position > 0 {
            // Posição comprada - avaliar saída
            if self.should_sell_long(ticker, price) {
                self.place_sell_order(ticker, current_position).await?;
            }
        } else if current_position < 0 {
            // Posição vendida - avaliar cobertura
            if self.should_cover_short(ticker, price) {
                self.place_buy_order(ticker, -current_position).await?;
            }
        }

        Ok(())
    }

    fn should_buy(&self, _ticker: &str, _price: Decimal) -> bool {
        // Lógica simplificada - na prática usaria indicadores técnicos
        // Por exemplo: RSI < 30, preço abaixo da média móvel, etc.
        rand::random::<f32>() < 0.1 // 10% de chance aleatória para demonstração
    }

    fn should_sell_long(&self, ticker: &str, current_price: Decimal) -> bool {
        if let Some(&entry_price) = self.last_prices.get(ticker) {
            let profit_pct = (current_price - entry_price) / entry_price;
            let loss_pct = (entry_price - current_price) / entry_price;

            // Take profit ou stop loss
            profit_pct >= self.take_profit_pct || loss_pct >= self.stop_loss_pct
        } else {
            false
        }
    }

    fn should_cover_short(&self, ticker: &str, current_price: Decimal) -> bool {
        if let Some(&entry_price) = self.last_prices.get(ticker) {
            let profit_pct = (entry_price - current_price) / entry_price;
            let loss_pct = (current_price - entry_price) / entry_price;

            // Take profit ou stop loss para posição vendida
            profit_pct >= self.take_profit_pct || loss_pct >= self.stop_loss_pct
        } else {
            false
        }
    }

    async fn place_buy_order(&mut self, ticker: &str, quantity: i64) -> Result<(), Box<dyn std::error::Error>> {
        let order = OrderRequest {
            symbol: ticker.to_string(),
            exchange: ExchangeId::B3,
            side: OrderSide::Buy,
            quantity: Decimal::from(quantity),
            price: None, // Market order
        };

        match self.broker.submit_order(order).await {
            Ok(order_id) => {
                println!("🛒 Ordem de compra enviada: {} x {} (ID: {})", quantity, ticker, order_id);
                // Atualizar posição estimada (na prática aguardaria confirmação)
                let current_pos = self.positions.get(ticker).copied().unwrap_or(0);
                self.positions.insert(ticker.to_string(), current_pos + quantity);
            }
            Err(e) => {
                eprintln!("❌ Erro ao enviar ordem de compra: {:?}", e);
            }
        }

        Ok(())
    }

    async fn place_sell_order(&mut self, ticker: &str, quantity: i64) -> Result<(), Box<dyn std::error::Error>> {
        let order = OrderRequest {
            symbol: ticker.to_string(),
            exchange: ExchangeId::B3,
            side: OrderSide::Sell,
            quantity: Decimal::from(quantity),
            price: None, // Market order
        };

        match self.broker.submit_order(order).await {
            Ok(order_id) => {
                println!("💰 Ordem de venda enviada: {} x {} (ID: {})", quantity, ticker, order_id);
                // Atualizar posição estimada
                let current_pos = self.positions.get(ticker).copied().unwrap_or(0);
                self.positions.insert(ticker.to_string(), current_pos - quantity);
            }
            Err(e) => {
                eprintln!("❌ Erro ao enviar ordem de venda: {:?}", e);
            }
        }

        Ok(())
    }

    pub fn print_positions(&self) {
        println!("\n📊 Posições Atuais:");
        if self.positions.is_empty() {
            println!("   Nenhuma posição aberta");
        } else {
            for (ticker, &quantity) in &self.positions {
                if quantity != 0 {
                    let direction = if quantity > 0 { "COMPRADO" } else { "VENDIDO" };
                    println!("   {} {}: {} {}", direction, ticker, quantity.abs(), if quantity > 0 { "📈" } else { "📉" });
                }
            }
        }
        println!();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Estratégia de Trading Simples - Tucano B3");
    println!("===========================================");

    let mut strategy = SimpleStrategy::new();

    // NOTA: Em um cenário real, você forneceria credenciais válidas
    println!("⚠️  Este exemplo requer credenciais válidas do ProfitDLL");
    println!("   Para testar, substitua pelos seus dados reais:");
    println!("   strategy.initialize(\"sua_chave\", \"seu_usuario\", \"sua_senha\").await?;");

    // Simular algumas operações sem conectar de fato
    println!("\n📝 Demonstração da estratégia (modo simulado):");

    // Simular alguns trades
    strategy.process_trade("PETR4", dec!(25.50)).await?;
    strategy.process_trade("VALE3", dec!(45.80)).await?;
    strategy.process_trade("ITUB4", dec!(32.10)).await?;

    strategy.print_positions();

    println!("✅ Exemplo concluído!");
    println!("   Para usar em produção:");
    println!("   1. Configure credenciais válidas do ProfitDLL");
    println!("   2. Implemente lógica de trading real (indicadores técnicos)");
    println!("   3. Adicione gestão de risco adequada");
    println!("   4. Configure logging e monitoramento");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = SimpleStrategy::new();
        assert_eq!(strategy.target_assets.len(), 3);
        assert_eq!(strategy.max_position, 100);
    }

    #[test]
    fn test_position_management() {
        let mut strategy = SimpleStrategy::new();

        // Simular entrada de posição
        strategy.positions.insert("PETR4".to_string(), 100);
        assert_eq!(strategy.positions.get("PETR4"), Some(&100));

        // Simular saída
        strategy.positions.insert("PETR4".to_string(), 0);
        assert_eq!(strategy.positions.get("PETR4"), Some(&0));
    }
}

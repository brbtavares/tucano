//! Exemplo de estratégia de trading simples
//! 
//! Demonstra uma estratégia básica de:
//! - Monitoramento de preços
//! - Envio de ordens condicionais
//! - Gestão de posições

use profit_dll::{
    ProfitConnector, CallbackEvent, AssetIdentifier, AccountIdentifier, 
    OrderSide, SendOrder
};
use std::collections::HashMap;
use tokio::time::Duration;
use tracing::{info, error};
use rust_decimal::Decimal;

/// Estratégia simples de trading
struct SimpleStrategy {
    connector: ProfitConnector,
    account: AccountIdentifier,
    password: String,
    
    // Estado da estratégia
    positions: HashMap<String, i64>, // ticker -> quantidade
    last_prices: HashMap<String, Decimal>, // ticker -> último preço
    target_assets: Vec<AssetIdentifier>,
    
    // Parâmetros
    max_position: i64,
    stop_loss_pct: Decimal,
    take_profit_pct: Decimal,
}

impl SimpleStrategy {
    pub fn new(
        connector: ProfitConnector,
        account: AccountIdentifier,
        password: String,
    ) -> Self {
        let target_assets = vec![
            AssetIdentifier::bovespa("PETR4"),
            AssetIdentifier::bovespa("VALE3"),
            AssetIdentifier::bovespa("ITUB4"),
        ];
        
        Self {
            connector,
            account,
            password,
            positions: HashMap::new(),
            last_prices: HashMap::new(),
            target_assets,
            max_position: 100,
            stop_loss_pct: Decimal::new(2, 2), // 2%
            take_profit_pct: Decimal::new(3, 2), // 3%
        }
    }
    
    pub async fn initialize(&self) -> Result<tokio::sync::mpsc::UnboundedReceiver<CallbackEvent>, Box<dyn std::error::Error>> {
        // Subscrever aos ativos alvo
        for asset in &self.target_assets {
            self.connector.subscribe_ticker(asset.ticker(), asset.exchange())?;
            self.connector.subscribe_price_book(asset.ticker(), asset.exchange())?;
        }
        
        // Configurar day trade
        self.connector.set_day_trade(true)?;
        
        info!("Estratégia inicializada com {} ativos", self.target_assets.len());
        
        // Retornar receiver para processar eventos externamente
        // (na prática você faria isso no initialize_login)
        let (_, receiver) = tokio::sync::mpsc::unbounded_channel();
        Ok(receiver)
    }
    
    pub async fn process_trade(&mut self, ticker: &str, price: Decimal) -> Result<(), Box<dyn std::error::Error>> {
        // Atualizar último preço
        self.last_prices.insert(ticker.to_string(), price);
        
        // Verificar se é um dos nossos ativos alvo
        if !self.target_assets.iter().any(|a| a.ticker() == ticker) {
            return Ok(());
        }
        
        info!("Processando trade: {} @ {}", ticker, price);
        
        // Estratégia simples: comprar na baixa, vender na alta
        self.evaluate_trading_opportunity(ticker, price).await?;
        
        Ok(())
    }
    
    async fn evaluate_trading_opportunity(&mut self, ticker: &str, price: Decimal) -> Result<(), Box<dyn std::error::Error>> {
        let current_position = self.positions.get(ticker).copied().unwrap_or(0);
        
        // Exemplo de lógica simples:
        // Se não temos posição e preço está "baixo" (exemplo), comprar
        // Se temos posição, verificar stop loss ou take profit
        
        if current_position == 0 {
            // Sem posição - avaliar entrada
            if self.should_buy(ticker, price) {
                self.place_buy_order(ticker, price).await?;
            }
        } else if current_position > 0 {
            // Posição comprada - avaliar saída
            if self.should_sell(ticker, price, current_position) {
                self.place_sell_order(ticker, price, current_position).await?;
            }
        }
        
        Ok(())
    }
    
    fn should_buy(&self, _ticker: &str, _price: Decimal) -> bool {
        // Lógica simples: sempre permitir compra se não ultrapassar limite
        // Na prática, você usaria indicadores técnicos, ML, etc.
        true
    }
    
    fn should_sell(&self, ticker: &str, current_price: Decimal, position: i64) -> bool {
        // Verificar stop loss ou take profit
        if let Some(&entry_price) = self.last_prices.get(ticker) {
            let price_change = (current_price - entry_price) / entry_price;
            
            if position > 0 {
                // Posição comprada
                if price_change <= -self.stop_loss_pct || price_change >= self.take_profit_pct {
                    return true;
                }
            }
        }
        
        false
    }
    
    async fn place_buy_order(&mut self, ticker: &str, price: Decimal) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(asset) = self.target_assets.iter().find(|a| a.ticker() == ticker) {
            let quantity = std::cmp::min(self.max_position, 100); // Quantidade fixa para exemplo
            
            let order = SendOrder::new_limit_order(
                self.account.clone(),
                asset.clone(),
                self.password.clone(),
                OrderSide::Buy,
                price.try_into().unwrap_or(0.0),
                quantity,
            );
            
            match self.connector.send_order(&order) {
                Ok(order_id) => {
                    info!("Ordem de compra enviada: {} x {} @ {} - ID: {}", 
                          quantity, ticker, price, order_id);
                    
                    // Atualizar posição esperada (seria atualizada via callback na realidade)
                    let current = self.positions.get(ticker).copied().unwrap_or(0);
                    self.positions.insert(ticker.to_string(), current + quantity);
                }
                Err(e) => {
                    error!("Erro ao enviar ordem de compra: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn place_sell_order(&mut self, ticker: &str, price: Decimal, quantity: i64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(asset) = self.target_assets.iter().find(|a| a.ticker() == ticker) {
            let order = SendOrder::new_limit_order(
                self.account.clone(),
                asset.clone(),
                self.password.clone(),
                OrderSide::Sell,
                price.try_into().unwrap_or(0.0),
                quantity,
            );
            
            match self.connector.send_order(&order) {
                Ok(order_id) => {
                    info!("Ordem de venda enviada: {} x {} @ {} - ID: {}", 
                          quantity, ticker, price, order_id);
                    
                    // Atualizar posição esperada
                    let current = self.positions.get(ticker).copied().unwrap_or(0);
                    self.positions.insert(ticker.to_string(), current - quantity);
                }
                Err(e) => {
                    error!("Erro ao enviar ordem de venda: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn print_status(&self) {
        info!("=== STATUS DA ESTRATÉGIA ===");
        for (ticker, &position) in &self.positions {
            if position != 0 {
                let last_price = self.last_prices.get(ticker).copied().unwrap_or_default();
                info!("  {}: {} @ {}", ticker, position, last_price);
            }
        }
        info!("===========================");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Iniciando estratégia de trading");

    // === CONFIGURAÇÃO ===
    let activation_key = "SUA_CHAVE_ATIVACAO_AQUI";
    let user = "seu_usuario";
    let password_login = "sua_senha_login";
    let password_trading = "sua_senha_trading";
    
    // Criar connector e conta
    let connector = ProfitConnector::new(None)?;
    let account = AccountIdentifier::new(
        12345, // Seu broker ID
        "123456".to_string(), // Sua conta
        "".to_string() // Subconta
    );
    
    // Inicializar
    let mut events = connector.initialize_login(activation_key, user, password_login).await?;
    
    // Criar estratégia
    let mut strategy = SimpleStrategy::new(connector, account, password_trading.to_string());
    strategy.initialize().await?;
    
    info!("Estratégia iniciada - processando eventos...");
    
    let mut event_count = 0;
    let max_events = 200; // Limitar para exemplo
    
    // Timer para status periódico
    let mut status_interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        tokio::select! {
            Some(event) = events.recv() => {
                match event {
                    CallbackEvent::StateChanged { connection_type, result } => {
                        info!("Conexão: {:?} -> {}", connection_type, result);
                    }
                    
                    CallbackEvent::NewTrade { 
                        ticker, price, .. 
                    } => {
                        // Processar trade através da estratégia
                        if let Err(e) = strategy.process_trade(&ticker, price).await {
                            error!("Erro ao processar trade: {}", e);
                        }
                    }
                    
                    // CallbackEvent::OrderChanged não existe na implementação atual
                    // Este seria um evento futuro para mudanças de ordem
                    
                    CallbackEvent::PriceBookOffer { ticker, .. } | 
                    CallbackEvent::OfferBookBid { ticker, .. } => {
                        // Poderia usar para melhor timing de entrada/saída
                        if event_count % 50 == 0 { // Log ocasional
                            info!("Book update: {}", ticker);
                        }
                    }
                    
                    _ => {
                        // Outros eventos
                    }
                }
                
                event_count += 1;
                if event_count >= max_events {
                    info!("Limite de eventos atingido para exemplo");
                    break;
                }
            }
            
            _ = status_interval.tick() => {
                strategy.print_status();
            }
        }
    }
    
    info!("Finalizando estratégia...");
    strategy.print_status();
    
    Ok(())
}

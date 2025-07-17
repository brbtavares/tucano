/*!
 * Exemplo completo: Paper Trading simples com BTCUSDT na Binance
 * 
 * Este exemplo demonstra como usar o ecossistema Toucan para:
 * - Conectar com dados de mercado em tempo real
 * - Implementar uma estratégia simples
 * - Usar gerenciamento de risco
 * - Executar paper trading
 * - Gerar relatórios de performance
 */

use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// Toucan ecosystem
use data::{
    event::{MarketEvent, DataKind},
    exchange::binance::spot::BinanceSpot,
    subscription::trade::PublicTrades,
    streams::builder::StreamBuilder,
};
use markets::{
    exchange::ExchangeId,
    instrument::market_data::MarketDataInstrument,
    Side,
};
use strategy::{
    AlgoStrategy, StrategyEvent, StrategyEventType,
    close_positions::ClosePositionsStrategy,
};
use risk::{RiskManager, RiskApproved, RiskRefused, NoRiskManager};
use execution::order::request::{OrderRequestOpen, OrderRequestCancel};
use analytics::summary::trading::TradingSummary;

/// Estratégia de exemplo: Trading baseado em preço
/// Compra quando o preço está abaixo de $60,000 e vende quando está acima de $70,000
#[derive(Debug, Clone)]
struct SimpleBtcStrategy {
    buy_threshold: Decimal,
    sell_threshold: Decimal,
    position_size: Decimal,
    last_price: Option<Decimal>,
    in_position: bool,
}

impl SimpleBtcStrategy {
    fn new() -> Self {
        Self {
            buy_threshold: dec!(60000),
            sell_threshold: dec!(70000),
            position_size: dec!(0.001), // 0.001 BTC
            last_price: None,
            in_position: false,
        }
    }
}

impl AlgoStrategy for SimpleBtcStrategy {
    type State = ();
    type Event = MarketEvent<MarketDataInstrument>;

    fn id(&self) -> &'static str {
        "simple_btc_strategy"
    }

    fn on_market_event(
        &mut self,
        _state: &Self::State,
        event: Self::Event,
    ) -> Vec<StrategyEvent> {
        let mut events = Vec::new();

        if let DataKind::Trade(trade) = event.kind {
            let price = Decimal::from_f64_retain(trade.price).unwrap_or_default();
            self.last_price = Some(price);

            // Lógica de trading simples
            if !self.in_position && price < self.buy_threshold {
                // Sinal de compra
                info!("🟢 Sinal de COMPRA: Preço ${} < ${}", price, self.buy_threshold);
                
                let order_request = OrderRequestOpen {
                    exchange: event.exchange.into(),
                    instrument: event.instrument.into(),
                    side: Side::Buy,
                    quantity: self.position_size,
                    order_type: execution::order::OrderType::Market,
                    time_in_force: execution::order::TimeInForce::GoodTillCancel,
                    client_order_id: execution::order::ClientOrderId::random(),
                    reduce_only: false,
                };

                events.push(StrategyEvent {
                    timestamp: Utc::now(),
                    event_type: StrategyEventType::OrderRequest(order_request),
                });
                
                self.in_position = true;
                
            } else if self.in_position && price > self.sell_threshold {
                // Sinal de venda
                info!("🔴 Sinal de VENDA: Preço ${} > ${}", price, self.sell_threshold);
                
                let order_request = OrderRequestOpen {
                    exchange: event.exchange.into(),
                    instrument: event.instrument.into(),
                    side: Side::Sell,
                    quantity: self.position_size,
                    order_type: execution::order::OrderType::Market,
                    time_in_force: execution::order::TimeInForce::GoodTillCancel,
                    client_order_id: execution::order::ClientOrderId::random(),
                    reduce_only: false,
                };

                events.push(StrategyEvent {
                    timestamp: Utc::now(),
                    event_type: StrategyEventType::OrderRequest(order_request),
                });
                
                self.in_position = false;
            }
        }

        events
    }
}

impl ClosePositionsStrategy for SimpleBtcStrategy {
    type Filter = ();

    fn close_positions_requests<T>(
        &self,
        _state: &Self::State,
        _filter: &Self::Filter,
    ) -> impl IntoIterator<Item = OrderRequestCancel<T, T>> {
        // Implementação simples - sem posições para fechar
        std::iter::empty()
    }
}

/// Risk Manager customizado para BTC
#[derive(Debug, Clone)]
struct BtcRiskManager {
    max_position_size: Decimal,
    max_orders_per_minute: u32,
    order_count: u32,
    last_reset: DateTime<Utc>,
}

impl BtcRiskManager {
    fn new() -> Self {
        Self {
            max_position_size: dec!(0.01), // Máximo 0.01 BTC
            max_orders_per_minute: 10,
            order_count: 0,
            last_reset: Utc::now(),
        }
    }

    fn reset_counter_if_needed(&mut self) {
        let now = Utc::now();
        if now.signed_duration_since(self.last_reset).num_minutes() >= 1 {
            self.order_count = 0;
            self.last_reset = now;
        }
    }
}

impl RiskManager for BtcRiskManager {
    type State = ();

    fn check(
        &self,
        _state: &Self::State,
        cancels: impl IntoIterator<Item = OrderRequestCancel<markets::exchange::ExchangeIndex, markets::instrument::InstrumentIndex>>,
        opens: impl IntoIterator<Item = OrderRequestOpen<markets::exchange::ExchangeIndex, markets::instrument::InstrumentIndex>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<OrderRequestCancel<markets::exchange::ExchangeIndex, markets::instrument::InstrumentIndex>>>,
        impl IntoIterator<Item = RiskApproved<OrderRequestOpen<markets::exchange::ExchangeIndex, markets::instrument::InstrumentIndex>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestCancel<markets::exchange::ExchangeIndex, markets::instrument::InstrumentIndex>>>,
        impl IntoIterator<Item = RiskRefused<OrderRequestOpen<markets::exchange::ExchangeIndex, markets::instrument::InstrumentIndex>>>,
    ) {
        let mut approved_cancels = Vec::new();
        let mut approved_opens = Vec::new();
        let mut refused_cancels = Vec::new();
        let mut refused_opens = Vec::new();

        // Aprovar todos os cancelamentos
        for cancel in cancels {
            approved_cancels.push(RiskApproved::new(cancel));
        }

        // Verificar ordens abertas
        for open in opens {
            // Verificar tamanho da posição
            if open.quantity > self.max_position_size {
                refused_opens.push(RiskRefused::new(
                    open,
                    format!("Tamanho da posição {} excede o máximo {}", open.quantity, self.max_position_size)
                ));
                continue;
            }

            // Verificar limite de ordens por minuto
            if self.order_count >= self.max_orders_per_minute {
                refused_opens.push(RiskRefused::new(
                    open,
                    format!("Limite de {} ordens por minuto excedido", self.max_orders_per_minute)
                ));
                continue;
            }

            approved_opens.push(RiskApproved::new(open));
        }

        (approved_cancels, approved_opens, refused_cancels, refused_opens)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Iniciando Paper Trading BTCUSDT na Binance");

    // Criar instrumentos
    let btc_usdt = MarketDataInstrument {
        exchange: ExchangeId::BinanceSpot,
        symbol: "BTCUSDT".to_string(),
        kind: markets::instrument::market_data::kind::MarketDataInstrumentKind::Spot,
    };

    // Configurar estratégia
    let mut strategy = SimpleBtcStrategy::new();
    info!("📊 Estratégia configurada: Compra < ${}, Venda > ${}", 
          strategy.buy_threshold, strategy.sell_threshold);

    // Configurar risk manager
    let risk_manager = BtcRiskManager::new();
    info!("🛡️ Risk Manager configurado: Max posição = {}", risk_manager.max_position_size);

    // Configurar stream de dados
    let mut stream = StreamBuilder::<MarketEvent<MarketDataInstrument>>::new()
        .subscribe([
            PublicTrades::new(btc_usdt.clone())
        ])
        .await?;

    info!("📡 Conectado ao stream de dados da Binance");
    info!("⏰ Executando por 30 segundos...");

    // Simular trading por 30 segundos
    let start_time = Utc::now();
    let mut trade_count = 0;
    let mut last_price = None;

    tokio::select! {
        _ = async {
            while let Some(event) = stream.next().await {
                match event {
                    Ok(market_event) => {
                        // Processar evento de mercado
                        let strategy_events = strategy.on_market_event(&(), market_event.clone());
                        
                        // Contar trades
                        if let DataKind::Trade(trade) = &market_event.kind {
                            trade_count += 1;
                            last_price = Some(trade.price);
                            
                            // Log a cada 100 trades
                            if trade_count % 100 == 0 {
                                info!("📈 Processados {} trades, último preço: ${:.2}", trade_count, trade.price);
                            }
                        }

                        // Processar eventos de estratégia
                        for strategy_event in strategy_events {
                            match strategy_event.event_type {
                                StrategyEventType::OrderRequest(order) => {
                                    // Simular verificação de risco
                                    let (_, approved_opens, _, refused_opens) = risk_manager.check(
                                        &(), 
                                        std::iter::empty(), 
                                        std::iter::once(order.clone())
                                    );
                                    
                                    let approved: Vec<_> = approved_opens.into_iter().collect();
                                    let refused: Vec<_> = refused_opens.into_iter().collect();
                                    
                                    if !approved.is_empty() {
                                        info!("✅ Ordem aprovada: {} {} de {}", 
                                              order.side, order.quantity, order.instrument);
                                    }
                                    
                                    for refused_order in refused {
                                        warn!("❌ Ordem rejeitada: {}", refused_order.reason);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Erro no stream: {}", e);
                    }
                }
            }
        } => {}
        _ = sleep(Duration::from_secs(30)) => {
            info!("⏱️ Tempo limite de 30 segundos atingido");
        }
    }

    // Relatório final
    let end_time = Utc::now();
    let duration = end_time.signed_duration_since(start_time);
    
    info!("📊 === RELATÓRIO FINAL ===");
    info!("⏰ Duração: {} segundos", duration.num_seconds());
    info!("📈 Trades processados: {}", trade_count);
    if let Some(price) = last_price {
        info!("💰 Último preço: ${:.2}", price);
    }
    info!("🎯 Estratégia: {} ({})", strategy.id(), 
          if strategy.in_position { "EM POSIÇÃO" } else { "SEM POSIÇÃO" });
    info!("🏁 Paper Trading finalizado com sucesso!");

    Ok(())
}

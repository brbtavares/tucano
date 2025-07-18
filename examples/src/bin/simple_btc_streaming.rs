/*!
 * Exemplo completo: Streaming de dados de mercado BTCUSDT da Binance
 * 
 * Este exemplo demonstra como usar o ecossistema Toucan para:
 * - Conectar com dados de mercado em tempo real
 * - Processar eventos de trade
 * - Implementar logging estruturado
 * - Manter estatísticas simples
 */

use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use chrono::Utc;
use futures::StreamExt;

// Toucan ecosystem
use data::{
    exchange::binance::spot::BinanceSpot,
    streams::{Streams, reconnect::{stream::ReconnectingStream, Event}},
    subscription::trade::{PublicTrades, PublicTrade},
    event::MarketEvent,
};
use markets::{
    instrument::market_data::{MarketDataInstrument, kind::MarketDataInstrumentKind},
    Side,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 Iniciando stream de dados BTCUSDT da Binance");

    info!("📊 Configurando stream para BTC/USDT");

    // Configurar stream de dados usando o padrão correto
    let streams = Streams::<PublicTrades>::builder()
        .subscribe([
            (BinanceSpot::default(), "btc", "usdt", MarketDataInstrumentKind::Spot, PublicTrades),
        ])
        .init()
        .await?;

    // Criar stream unificado
    let mut stream = streams
        .select_all()
        .with_error_handler(|error| warn!(?error, "MarketStream gerou erro"));

    info!("📡 Conectado ao stream de dados da Binance");
    info!("⏰ Executando por 30 segundos...");

    // Estatísticas simples
    let mut stats = TradingStats::new();
    let start_time = Utc::now();

    // Processar eventos por 30 segundos
    tokio::select! {
        _ = async {
            while let Some(event) = stream.next().await {
                match event {
                    Event::Item(market_event) => {
                        stats.process_event(&market_event);
                        
                        // Log a cada 100 trades
                        if stats.trade_count % 100 == 0 {
                            info!("📈 Trades: {} | Último preço: ${:.2} | Volume: {:.4} BTC", 
                                  stats.trade_count, 
                                  stats.last_price.unwrap_or(0.0),
                                  stats.volume_btc);
                        }
                    }
                    Event::Reconnecting(exchange_id) => {
                        warn!("🔄 Reconectando exchange: {:?}", exchange_id);
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
    info!("📈 Total de trades: {}", stats.trade_count);
    info!("💰 Preço mínimo: ${:.2}", stats.min_price.unwrap_or(0.0));
    info!("💰 Preço máximo: ${:.2}", stats.max_price.unwrap_or(0.0));
    info!("💰 Último preço: ${:.2}", stats.last_price.unwrap_or(0.0));
    info!("📊 Volume total: {:.4} BTC", stats.volume_btc);
    info!("📊 Volume compras: {:.4} BTC", stats.buy_volume_btc);
    info!("📊 Volume vendas: {:.4} BTC", stats.sell_volume_btc);
    info!("📊 Trades de compra: {}", stats.buy_trades);
    info!("📊 Trades de venda: {}", stats.sell_trades);
    info!("🏁 Streaming finalizado com sucesso!");

    Ok(())
}

/// Estrutura para manter estatísticas de trading
#[derive(Debug, Clone)]
struct TradingStats {
    trade_count: u64,
    min_price: Option<f64>,
    max_price: Option<f64>,
    last_price: Option<f64>,
    volume_btc: f64,
    buy_volume_btc: f64,
    sell_volume_btc: f64,
    buy_trades: u64,
    sell_trades: u64,
}

impl TradingStats {
    fn new() -> Self {
        Self {
            trade_count: 0,
            min_price: None,
            max_price: None,
            last_price: None,
            volume_btc: 0.0,
            buy_volume_btc: 0.0,
            sell_volume_btc: 0.0,
            buy_trades: 0,
            sell_trades: 0,
        }
    }

    fn process_event(&mut self, event: &MarketEvent<MarketDataInstrument, PublicTrade>) {
        let trade = &event.kind;
        self.trade_count += 1;
        self.last_price = Some(trade.price);
        self.volume_btc += trade.amount;

        // Atualizar min/max preços
        match self.min_price {
            None => self.min_price = Some(trade.price),
            Some(min) if trade.price < min => self.min_price = Some(trade.price),
            _ => {}
        }

        match self.max_price {
            None => self.max_price = Some(trade.price),
            Some(max) if trade.price > max => self.max_price = Some(trade.price),
            _ => {}
        }

        // Separar por side
        match trade.side {
            Side::Buy => {
                self.buy_trades += 1;
                self.buy_volume_btc += trade.amount;
            }
            Side::Sell => {
                self.sell_trades += 1;
                self.sell_volume_btc += trade.amount;
            }
        }
    }
}

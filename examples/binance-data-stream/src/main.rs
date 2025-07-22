use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{io, time::Duration};
use tokio::sync::mpsc;
use tracing::{info, warn};

mod ui;
mod data;
mod config;
mod toucan_integration;

use ui::App;
use data::{OrderBookData, TradeData};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Binance Data Stream TUI");

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create communication channels
    let (orderbook_tx, orderbook_rx) = mpsc::unbounded_channel::<OrderBookData>();
    let (trades_tx, trades_rx) = mpsc::unbounded_channel::<TradeData>();

    // Create and run app
    let app = App::new(orderbook_rx, trades_rx);
    let res = run_app(&mut terminal, app, orderbook_tx, trades_tx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    orderbook_tx: mpsc::UnboundedSender<OrderBookData>,
    trades_tx: mpsc::UnboundedSender<TradeData>,
) -> Result<()> {
    // Start data streams
    tokio::spawn(async move {
        if let Err(e) = start_data_streams(orderbook_tx, trades_tx).await {
            warn!("Data stream error: {}", e);
        }
    });

    loop {
        terminal.draw(|f| app.render(f))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') => app.reset(),
                    _ => {}
                }
            }
        }

        app.update().await;
    }
}

async fn start_data_streams(
    orderbook_tx: mpsc::UnboundedSender<OrderBookData>,
    trades_tx: mpsc::UnboundedSender<TradeData>,
) -> Result<()> {
    info!("Starting Binance WebSocket streams for BTCUSDT perpetual futures");
    info!("Note: Using mock data for demonstration. Real integration coming soon!");
    
    /* TODO: Implementar integração real com Toucan framework
    
    Para implementar streams reais, descomente e use este código:
    
    use data::{
        streams::Streams,
        subscription::{trade::PublicTrades, book::OrderBooksL1},
        exchange::binance::futures::BinanceFuturesUsd,
    };
    use markets::instrument::market_data::kind::MarketDataInstrumentKind;
    
    // Criar streams de trades reais
    let mut trades_stream = Streams::<PublicTrades>::builder()
        .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, PublicTrades)])
        .init()
        .await?;
    
    // Criar streams de order book reais  
    let mut book_stream = Streams::<OrderBooksL1>::builder()
        .subscribe([(BinanceFuturesUsd::default(), "btc", "usdt", MarketDataInstrumentKind::Perpetual, OrderBooksL1)])
        .init()
        .await?;
    
    // Processar events em paralelo
    let trades_task = tokio::spawn(async move {
        while let Some(trade_event) = trades_stream.next().await {
            if let Ok(trade) = trade_event {
                let trade_data = TradeData {
                    symbol: "BTCUSDT".to_string(),
                    trade_id: trade.id as u64,
                    price: trade.price,
                    quantity: trade.quantity,
                    timestamp: trade.ts,
                    is_buyer_maker: trade.buyer_order_id.is_some(),
                };
                let _ = trades_tx.send(trade_data);
            }
        }
    });
    
    let orderbook_task = tokio::spawn(async move {
        while let Some(book_event) = book_stream.next().await {
            if let Ok(book) = book_event {
                let mut bids = BTreeMap::new();
                let mut asks = BTreeMap::new();
                
                if let Some(best_bid) = book.bid {
                    bids.insert(best_bid.price.into(), best_bid.quantity);
                }
                if let Some(best_ask) = book.ask {
                    asks.insert(best_ask.price.into(), best_ask.quantity);
                }
                
                let orderbook_data = OrderBookData {
                    symbol: "BTCUSDT".to_string(),
                    bids,
                    asks,
                    timestamp: book.ts,
                    last_update_id: 0,
                };
                let _ = orderbook_tx.send(orderbook_data);
            }
        }
    });
    
    let _ = tokio::try_join!(trades_task, orderbook_task);
    */
    
    // Implementação mock atual
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    
    loop {
        interval.tick().await;
        
        // Simulate orderbook data
        let orderbook = OrderBookData::mock_data();
        if orderbook_tx.send(orderbook).is_err() {
            warn!("OrderBook channel closed, stopping stream");
            break;
        }
        
        // Simulate trade data
        let trade = TradeData::mock_data();
        if trades_tx.send(trade).is_err() {
            warn!("Trades channel closed, stopping stream");
            break;
        }
    }
    
    Ok(())
}

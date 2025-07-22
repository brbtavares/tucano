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
    
    // TODO: Implementar streams reais usando Toucan data crate
    // Por enquanto, vamos simular dados para demonstração
    
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    
    loop {
        interval.tick().await;
        
        // Simulate orderbook data
        let orderbook = OrderBookData::mock_data();
        if orderbook_tx.send(orderbook).is_err() {
            break;
        }
        
        // Simulate trade data
        let trade = TradeData::mock_data();
        if trades_tx.send(trade).is_err() {
            break;
        }
    }
    
    Ok(())
}

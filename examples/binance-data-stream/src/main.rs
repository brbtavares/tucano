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
mod config;
mod types;
mod toucan_integration;

use ui::App;
use types::{OrderBookData, TradeData};

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
    // Execute streams e UI concorrentemente usando select
    tokio::select! {
        // Stream de dados em background
        stream_result = toucan_integration::start_real_data_streams(orderbook_tx, trades_tx) => {
            match stream_result {
                Ok(()) => {
                    info!("Data streams ended successfully");
                    Ok(())
                },
                Err(e) => {
                    warn!("Data stream error: {}", e);
                    Err(e)
                }
            }
        }
        
        // Loop da UI
        ui_result = async {
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
        } => {
            ui_result
        }
    }
}

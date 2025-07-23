use tracing::{info, warn};
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
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry, filter::EnvFilter, Layer};
use tracing::Subscriber;

use std::{io, time::Duration};
use tokio::sync::mpsc;


mod ui;
mod config;
mod types;
mod toucan_integration;

use ui::App;
use types::{OrderBookData, TradeData, LogBuffer};
//use types::LogBuffer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with environment filter and colors
    // Create log buffer for TUI
    let log_buffer = LogBuffer::new(10000);

    // Custom layer to push logs to buffer
    let tui_log_layer = TuiLogLayer { buffer: log_buffer.clone() };

    // Compose subscriber
    let env_filter = match std::env::var("RUST_LOG") {
        Ok(_) => EnvFilter::from_default_env(),
        Err(_) => EnvFilter::new("info"),
    };
    let subscriber = Registry::default()
        .with(fmt::Layer::default().with_ansi(true))
        .with(env_filter)
        .with(tui_log_layer);
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

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
    let app = App::new(orderbook_rx, trades_rx, log_buffer);
// Layer to send logs to TUI log buffer
struct TuiLogLayer {
    buffer: LogBuffer,
}

impl<S> Layer<S> for TuiLogLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        use std::fmt::Write;
        struct MsgVisitor<'a> {
            out: &'a mut String,
            first: bool,
        }
        impl<'a> tracing::field::Visit for MsgVisitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                if self.first {
                    let _ = write!(self.out, ": {} = {:?}", field.name(), value);
                    self.first = false;
                } else {
                    let _ = write!(self.out, ", {} = {:?}", field.name(), value);
                }
            }
        }
        let mut msg = String::new();
        let meta = event.metadata();
        let _ = write!(msg, "[{}] {}", meta.level(), meta.target());
        event.record(&mut MsgVisitor { out: &mut msg, first: true });
        self.buffer.push(msg);
    }
}
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

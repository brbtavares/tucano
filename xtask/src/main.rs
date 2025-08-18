use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod commands;
mod tui;
mod workspace;

use app::*;
use commands::*;
use workspace::*;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Workspace automation tool with TUI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch TUI interface
    Tui,
    /// Format all code
    Fmt,
    /// Run clippy
    Clippy,
    /// Check disclaimers in files
    CheckDisclaimer,
    /// Add disclaimer to files (--fix to apply)
    AddDisclaimer {
        #[arg(long)]
        fix: bool,
    },
    /// Show crate sizes comparison
    SizeCheck,
    /// Release crates to crates.io
    Release {
        #[arg(short, long)]
        crate_name: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => run_tui().await,
        Commands::Fmt => run_fmt().await,
        Commands::Clippy => run_clippy().await,
        Commands::CheckDisclaimer => check_disclaimers().await,
        Commands::AddDisclaimer { fix } => add_disclaimers(fix).await,
        Commands::SizeCheck => show_size_comparison().await,
        Commands::Release {
            crate_name,
            dry_run,
        } => release_crates(crate_name, dry_run).await,
    }
}

async fn run_tui() -> Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load workspace info
    let workspace = WorkspaceInfo::load().await?;
    let mut app = App::new(workspace);

    let result = run_app(&mut terminal, &mut app).await;

    // Cleanup terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| tui::ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => app.select().await?,
                        KeyCode::Esc => app.back(),
                        KeyCode::Char('r') => app.refresh().await?,
                        KeyCode::Tab => app.next_tab(),
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

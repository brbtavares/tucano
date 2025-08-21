

mod commands;
mod workspace;
use commands::*;
use anyhow::Result;
use clap::{Parser, Subcommand};
use fmt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Format all code
    Fmt,
    /// Run clippy
    Clippy,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Fmt => {
            fmt::run_fmt().await?
        }
        Commands::Clippy => {
            clippy::run_clippy().await?
        }
    }
    Ok(())
}





// Mini-Disclaimer: Educational/experimental use; not investment advice or affiliation; see README & DISCLAIMER.
mod commands;
mod workspace;
use commands::*;
use anyhow::Result;
use clap::{Parser, Subcommand};

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
    /// Check or fix disclaimers in files
    Disclaimer {
        /// Add missing disclaimers (default: only check)
        #[arg(long)]
        fix: bool,
    },
    /// Show inventory of all crates (types, functions, dependencies)
    Inventory,
    /// Show crate sizes comparison
    Size,
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
    match cli.command {
        Commands::Fmt => {
            fmt::run_fmt().await?
        }
        Commands::Clippy => {
            clippy::run_clippy().await?
        }
        Commands::Disclaimer { fix } => {
            if fix {
                disclaimer::add_disclaimers(true).await?
            } else {
                disclaimer::check_disclaimers().await?
            }
        }
        Commands::Inventory => {
            inventory::run_inventory()?;
        }
        Commands::Size => {
            size::show_size_comparison().await?;
        }
        Commands::Release { crate_name, dry_run } => {
            release::release_crates(crate_name, dry_run).await?;
        }
    }
    Ok(())
}




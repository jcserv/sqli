use anyhow::Result;
use clap::{Parser, Subcommand};

use sqli::tui::run_tui;
use sqli::query::run_query;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(alias = "ui")]
    Tui,
    #[clap(alias = "q")]
    Query,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => {
            run_tui()?;
        },
        Commands::Query => {
            run_query()?;
        }
    }
    Ok(())
}


use anyhow::Result;
use clap::{Parser, Subcommand};
use tokio;

use sqli::tui::run::run_tui;
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
    Query {
        #[arg(short, long, help = "The database connection string to connect to (ex: postgresql://user:password@host:port/database)")]
        url: String,
        #[arg(short, long, help = "The SQL statement(s) to execute")]
        sql: String,

        // todo: add flag for collection
        // todo: add flag for format: table, json, csv
        // todo: add flag for verbose (show success message, execution time, number of rows returned)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Tui) {
        Commands::Tui => {
            run_tui()?;
        },
        Commands::Query { url, sql } => {
            run_query(url, sql).await?;
        }
    }
    Ok(())
}


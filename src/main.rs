use anyhow::Result;
use clap::{Parser, Subcommand};
use sqli::config::{run_config_set, run_config_list, ConfigManager};
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
    /// Open the Terminal UI
    #[clap(alias = "ui")]
    Tui,
    /// Query a SQL database, given a connection string and SQL query
    #[clap(alias = "q")]
    Query {
        #[arg(short, long, help = "The database connection string to connect to")]
        url: Option<String>,
        #[arg(short, long, help = "The connection name from config")]
        connection: Option<String>,
        #[arg(short, long, help = "The SQL statement(s) to execute")]
        sql: String,

        // todo: add flag for collection
        // todo: add flag for format: table, json, csv
        // todo: add flag for verbose (show success message, execution time, number of rows returned)
    },
    /// Configure or list database connections
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Configure a database connection
    Set {
        #[arg(long, help = "The name of the connection (ex. local-db)")]
        name: String,
        #[arg(long, help = "The type of the connection (supported types: postgresql)")]
        conn: String,
        #[arg(long, help = "The host of the connection (ex. localhost)")]
        host: String,
        #[arg(long, help = "The port to connect to (ex. 5432)")]
        port: u16,
        #[arg(long, help = "The database name (ex. my-db)")]
        database: String,
        #[arg(long, help = "The user to connect as (ex. postgres)")]
        user: String,
        #[arg(long, help = "[WARNING: This will save the password in plaintext in the config file]\nIf not provided, it will be prompted for.")]
        password: Option<String>,
    },
    /// List all configured database connections
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config_manager = ConfigManager::new()?;

    match cli.command.unwrap_or(Commands::Tui) {
        
        Commands::Tui => {
            run_tui()?;
        },
        Commands::Query { url, connection, sql } => {
            run_query(sql, url, connection, ).await?;
        },
        Commands::Config { action } => {
            match action {
                ConfigAction::Set { name, conn, host, port, database, user, password } => {
                    run_config_set(&mut config_manager, name, conn, host, port, database, user, password).await?;
                },
                ConfigAction::List => {
                    run_config_list(&mut config_manager).await?;
                }
            }
        }
    }
    Ok(())
}


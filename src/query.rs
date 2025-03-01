use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    config::{ConfigManager, Connection}, file::FileSystem, settings::UserSettings, sql::{factory::create_executor, interface::Executor, result::{format_output, Format, QueryResult}}
};

/// Wrapper function that executes a query and prints results to stdout (for CLI usage)
pub async fn run_query(url: Option<String>, conn: Option<String>, sql: String, format: Option<String>) -> Result<()> {
    let password: Option<String> = if let Some(conn_name) = &conn {
        if let Some(conn) = get_connection(conn_name)? {
            if conn.requires_password() {
                Some(rpassword::prompt_password("Enter database password: ")?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let output_format = match format {
        Some(fmt) => Format::from_str(&fmt)?,
        None => Format::default(),
    };

    let result = execute_query(sql, url, conn, password).await?;
    
    if !result.columns.is_empty() {
        format_output(&result, output_format)?;
    }
    
    Ok(())
}

// Core function to execute a SQL query and return the results
pub async fn execute_query(
    sql: String,
    url: Option<String>,
    connection: Option<String>,
    password: Option<String>,
) -> Result<QueryResult> {
    let connection_url = get_connection_url(url, connection, password)?;
    let sql_content = if Path::new(&sql).exists() && sql.ends_with(".sql") {
        let fs = FileSystem::new()?;
        fs.read_file(&sql)?
    } else {
        sql
    };

    let executor = create_executor(connection_url, sql_content);
    executor.execute().await
}

pub fn get_connection_url(url: Option<String>, connection: Option<String>, password: Option<String>) -> Result<String> {
    let settings = UserSettings::from_env();
    if let Some(conn_name) = connection {
        let fs = FileSystem::with_paths(settings.user_dir, settings.workspace_dir)?;
        let config_manager = ConfigManager::with_filesystem(fs);
        let conn = config_manager
            .get_connection(&conn_name)?
            .ok_or_else(|| anyhow!("Connection '{}' not found", conn_name))?;
        
        Ok(conn.to_url(password))
    } else if let Some(url) = url {
        Ok(url)
    } else {
        Err(anyhow!("Either --url or --connection must be provided"))
    }
}

pub fn get_connection(name: &str) -> Result<Option<Connection>> {
    let settings = UserSettings::from_env();
    let fs = FileSystem::with_paths(settings.user_dir, settings.workspace_dir)?;
    let config_manager = ConfigManager::with_filesystem(fs);
    config_manager.get_connection(name)
}
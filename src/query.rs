use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    config::{ConfigManager, Connection},
    file,
    sql::{factory::create_executor, interface::{Executor, QueryResult}},
};

/// Wrapper function that executes a query and prints results to stdout (for CLI usage)
pub async fn run_query(sql: String, url: Option<String>, conn: Option<String>) -> Result<()> {
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

    let result = execute_query(sql, url, conn, password).await?;

    println!("✅ Query executed successfully in {}ms", result.execution_time.as_millis());
    println!("Rows returned: {}", result.row_count);
    
    if !result.columns.is_empty() {
        println!("\nColumns: {:?}", result.columns);
        
        for (i, row) in result.rows.iter().enumerate() {
            println!("Row {}: {:?}", i + 1, row);
        }
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
    let connection_url = get_connection_url(url, connection, password).await?;
    
    let sql_content = if Path::new(&sql).exists() && sql.ends_with(".sql") {
        file::read_file_to_string(&sql)?
    } else {
        sql
    };

    let executor = create_executor(connection_url, sql_content);
    executor.execute().await
}

pub async fn get_connection_url(url: Option<String>, connection: Option<String>, password: Option<String>) -> Result<String> {
    if let Some(conn_name) = connection {
        let config_manager = ConfigManager::new()?;
        let mut conn = config_manager
            .get_connection(&conn_name)?
            .ok_or_else(|| anyhow!("Connection '{}' not found", conn_name))?;
        
        if let Some(pwd) = password {
            conn.password = Some(pwd);
        }
        
        Ok(conn.to_url())
    } else if let Some(url) = url {
        Ok(url)
    } else {
        Err(anyhow!("Either --url or --connection must be provided"))
    }
}

pub fn get_connection(name: &str) -> Result<Option<Connection>> {
    let config_manager = ConfigManager::new()?;
    config_manager.get_connection(name)
}
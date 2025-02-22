use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    config::ConfigManager,
    file,
    sql::{factory::create_executor, interface::{Executor, QueryResult}},
};

/// Core function to execute a SQL query and return the results
pub async fn execute_query(
    sql: String,
    url: Option<String>,
    connection: Option<String>
) -> Result<QueryResult> {
    let connection_url = get_connection_url(url, connection).await?;
    
    let sql_content = if Path::new(&sql).exists() && sql.ends_with(".sql") {
        file::read_file_to_string(&sql)?
    } else {
        sql
    };

    let executor = create_executor(connection_url, sql_content);
    executor.execute().await
}

/// Wrapper function that executes a query and prints results to stdout (for CLI usage)
pub async fn run_query(sql: String, url: Option<String>, connection: Option<String>) -> Result<()> {
    let result = execute_query(sql, url, connection).await?;

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

async fn get_connection_url(url: Option<String>, connection: Option<String>) -> Result<String> {
    if let Some(conn_name) = connection {
        let config_manager = ConfigManager::new()?;
        let conn = config_manager
            .get_connection(&conn_name)?
            .ok_or_else(|| anyhow!("Connection '{}' not found", conn_name))?;
        Ok(conn.to_url())
    } else if let Some(url) = url {
        Ok(url)
    } else {
        Err(anyhow!("Either --url or --connection must be provided"))
    }
}
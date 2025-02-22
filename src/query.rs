use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    config::ConfigManager,
    file,
    sql::{factory::create_executor, interface::Executor},
};

/// Runs a SQL query with either a direct URL or a named connection from config
pub async fn run_query(sql: String, url: Option<String>, connection: Option<String>) -> Result<()> {
    let connection_url = get_connection_url(url, connection).await?;
    
    let sql_content = if Path::new(&sql).exists() && sql.ends_with(".sql") {
        file::read_file_to_string(&sql)?
    } else {
        sql
    };

    let executor = create_executor(connection_url, sql_content);
    executor.execute().await?;
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

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time: std::time::Duration,
    pub row_count: usize,
}

impl Default for QueryResult {
    fn default() -> Self {
        Self {
            columns: vec!["id".to_string(), "title".to_string(), "author_id".to_string()],
            rows: vec![
                vec!["1".to_string(), "The Martian".to_string(), "1".to_string()],
                vec!["2".to_string(), "Natural Acts".to_string(), "2".to_string()],
                vec!["3".to_string(), "Feminism Interrupted".to_string(), "3".to_string()],
                vec!["4".to_string(), "Project Hail Mary".to_string(), "1".to_string()],
            ],
            execution_time: std::time::Duration::from_millis(42),
            row_count: 4,
        }
    }
}

impl QueryResult {
    pub fn new(columns: Vec<String>, rows: Vec<Vec<String>>, execution_time: std::time::Duration) -> Self {
        let row_count = rows.len();
        Self {
            columns,
            rows,
            execution_time,
            row_count,
        }
    }

    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            execution_time: std::time::Duration::from_secs(0),
            row_count: 0,
        }
    }
}
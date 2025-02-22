use std::fmt;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SQLType {
    Postgresql,
}

impl fmt::Display for SQLType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SQLType::Postgresql => write!(f, "postgresql"),
        }
    }
}

pub fn get_sql_type(conn_type: &str) -> Option<SQLType> {
    let conn = conn_type.to_lowercase();
    match conn.as_str() {
        "postgresql" => {
            Some(SQLType::Postgresql)
        }
        _ => {
            None
        }
    }
}

pub trait Executor {
    fn execute(&self) -> impl std::future::Future<Output = Result<QueryResult>> + Send;
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
            columns: Vec::new(),
            rows: Vec::new(),
            execution_time: std::time::Duration::from_secs(0),
            row_count: 0,
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
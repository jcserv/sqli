use anyhow::Result;

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
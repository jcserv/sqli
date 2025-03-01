use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::result::QueryResult;

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
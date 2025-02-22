use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Column, Row};
use std::time::Instant;
use super::interface::{Executor, QueryResult};

pub struct PostgresExecutor {
    pub url: String,
    pub sql: String,
}

impl Executor for PostgresExecutor {
    async fn execute(&self) -> Result<QueryResult> {
        let start_time = Instant::now();
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(&self.url).await?;

        let mut tx = pool.begin().await?;
        
        sqlx::query(&self.sql)
            .execute(&mut *tx)
            .await?;
            
        let rows = sqlx::query(&self.sql)
            .fetch_all(&mut *tx)
            .await?;
            
        let mut columns = Vec::new();
        let mut result_rows = Vec::new();
        
        if !rows.is_empty() {
            columns = rows[0].columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect();
                
                for row in &rows {
                    let values: Vec<String> = columns.iter()
                        .map(|name| {
                            row.try_get::<String, _>(name.as_str())
                                .or_else(|_| row.try_get::<i32, _>(name.as_str()).map(|v| v.to_string()))
                                .or_else(|_| row.try_get::<i64, _>(name.as_str()).map(|v| v.to_string()))
                                .or_else(|_| row.try_get::<f64, _>(name.as_str()).map(|v| v.to_string()))
                                .or_else(|_| row.try_get::<bool, _>(name.as_str()).map(|v| v.to_string()))
                                .unwrap_or_else(|_| "NULL".to_string())
                        })
                        .collect();
                    result_rows.push(values);
                }
        }
        
        tx.commit().await?;
        
        let execution_time = start_time.elapsed();
        Ok(QueryResult::new(columns, result_rows, execution_time))
    }
}
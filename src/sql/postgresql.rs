use anyhow::Result;
use chrono::{DateTime, Utc, NaiveDate};
use sqlx::{postgres::{PgPoolOptions, PgRow}, Column, Row, ValueRef, TypeInfo};
use std::time::{Duration, Instant};
use super::{interface::Executor, result::QueryResult};

pub struct PostgresExecutor {
    pub url: String,
    pub sql: String,
}

impl Executor for PostgresExecutor {
    async fn execute(&self) -> Result<QueryResult> {
        let start_time = Instant::now();
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&self.url).await?;

        let mut tx = pool.begin().await?;
            
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
                    let values: Vec<String> = row.columns()
                        .iter()
                        .enumerate()
                        .map(|(i, col)| {
                            convert_pg_value_to_string(row, i, col)
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

fn convert_pg_value_to_string(row: &PgRow, idx: usize, col: &sqlx::postgres::PgColumn) -> String {
    let type_info = col.type_info();
    let type_name = type_info.name();

    match type_name {
        _ if row.try_get_raw(idx).map(|v| v.is_null()).unwrap_or(true) => "NULL".to_string(),        
        "INT2" | "SMALLINT" => row.try_get::<i16, _>(idx).map(|v: i16| v.to_string()).unwrap_or_else(|_| "NULL".to_string()),
        "INT4" | "INT" => row.try_get::<i32, _>(idx).map(|v| v.to_string()).unwrap_or_else(|_| "NULL".to_string()),
        "INT8" | "BIGINT" => row.try_get::<i64, _>(idx).map(|v| v.to_string()).unwrap_or_else(|_| "NULL".to_string()),
        "FLOAT4" | "REAL" => row.try_get::<f32, _>(idx).map(|v| v.to_string()).unwrap_or_else(|_| "NULL".to_string()),
        "FLOAT8" | "DOUBLE PRECISION" => row.try_get::<f64, _>(idx).map(|v| v.to_string()).unwrap_or_else(|_| "NULL".to_string()),
        "NUMERIC" | "DECIMAL" => row.try_get::<String, _>(idx).unwrap_or_else(|_| "NULL".to_string()),
        "VARCHAR" | "CHAR" | "TEXT" | "NAME" => row.try_get::<String, _>(idx).map(|v| v).unwrap_or_else(|_| "NULL".to_string()),
        "BOOL" | "BOOLEAN" => row.try_get::<bool, _>(idx).map(|v| v.to_string()).unwrap_or_else(|_| "NULL".to_string()),
        "DATE" => row.try_get::<NaiveDate, _>(idx)
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "NULL".to_string()),
        "TIME" => row.try_get::<String, _>(idx).unwrap_or_else(|_| "NULL".to_string()),
        "TIMESTAMP" => row.try_get::<String, _>(idx).unwrap_or_else(|_| "NULL".to_string()),
        "TIMESTAMPTZ" | "TIMESTAMP WITH TIME ZONE" => row.try_get::<DateTime<Utc>, _>(idx)
            .map(|v| v.to_rfc3339())
            .unwrap_or_else(|_| "NULL".to_string()),
        "JSON" | "JSONB" => row.try_get::<serde_json::Value, _>(idx)
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|_| "NULL".to_string()),

        type_name if type_name.contains("[]") => {
            match type_name {
                "TEXT[]" => format_array::<String>(row, idx),
                "VARCHAR[]" => format_array::<String>(row, idx),
                "INT[]" => format_array::<i32>(row, idx),
                "BIGINT[]" => format_array::<i64>(row, idx),
                "FLOAT8[]" => format_array::<f64>(row, idx),
                "BOOL[]" => format_array::<bool>(row, idx),
                _ => format!("[Unhandled array type: {}]", type_name)
            }
        },

        _ => row.try_get::<String, _>(idx)
            .unwrap_or_else(|_| truncate_string(&format!("[{}]", type_name), 50))
    }
}

fn format_array<T>(row: &PgRow, idx: usize) -> String 
where 
    T: ToString + for<'a> sqlx::decode::Decode<'a, sqlx::postgres::Postgres> + 
       sqlx::types::Type<sqlx::postgres::Postgres> + sqlx::postgres::PgHasArrayType
{
    row.try_get::<Vec<T>, _>(idx)
        .map(|arr| {
            let items: Vec<String> = arr.iter().map(|item| item.to_string()).collect();
            format!("[{}]", items.join(", "))
        })
        .unwrap_or_else(|_| "[]".to_string())
}
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len])
    }
}
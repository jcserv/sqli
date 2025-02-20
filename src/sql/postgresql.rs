use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Column, Row};
use super::interface::Executor;

pub struct PostgresExecutor {
    pub url: String,
    pub sql: String,
}

impl Executor for PostgresExecutor {
    async fn execute(&self) -> Result<()> {
        let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&self.url).await
    {
        Ok(pool) => {
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to database: {:?}", err);
            std::process::exit(1);
        }
    };

    let mut tx = pool.begin().await?;

    match sqlx::query(&self.sql)
        .execute(&mut *tx)
        .await 
    {
        Ok(_) => {
            println!("✅ Query successful");
            // todo: track execution time, number of rows returned
            
            if let Ok(rows) = sqlx::query(&self.sql)
                .fetch_all(&mut *tx)
                .await 
            {
                if !rows.is_empty() {
                    let column_names: Vec<&str> = rows[0].columns()
                        .iter()
                        .map(|c| c.name())
                        .collect();

                    println!("{:?}", column_names);

                    for row in &rows {
                        let values: Vec<String> = column_names.iter()
                            .map(|&name| {
                                row.try_get::<String, _>(name)
                                    .or_else(|_| row.try_get::<i32, _>(name).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<i64, _>(name).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<f64, _>(name).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<bool, _>(name).map(|v| v.to_string()))
                                    .unwrap_or_else(|_| "NULL".to_string())
                            })
                            .collect();
                        println!("{:?}", values);
                    }
                }
            }
            
            tx.commit().await?;
        }
        Err(err) => {
            println!("🔥 Failed to execute query: {:?}", err);
            tx.rollback().await?;
        }
    }
    Ok(())
}
}
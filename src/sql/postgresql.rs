use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Column, Row};
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;
    use std::thread;

    use crate::sql::interface::Executor;
    use crate::sql::postgresql::PostgresExecutor;

    #[tokio::test]
    async fn test_postgres_executor_simple_query() -> Result<()> {
        if !should_run_postgres_tests() {
            return Ok(());
        }

        let db = PostgresTestDb::new();
        db.setup()?;

        let executor = PostgresExecutor {
            url: db.connection_string(),
            sql: "SELECT 1 as test".to_string(),
        };

        let result = executor.execute().await?;
        
        assert_eq!(result.columns.len(), 1);
        assert_eq!(result.columns[0], "test");
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], "1");

        db.teardown()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_executor_query_users() -> Result<()> {
        if !should_run_postgres_tests() {
            return Ok(());
        }

        let db = PostgresTestDb::new();
        db.setup()?;

        let executor = PostgresExecutor {
            url: db.connection_string(),
            sql: "SELECT id, name, email FROM users ORDER BY id".to_string(),
        };

        let result = executor.execute().await?;
        
        assert_eq!(result.columns.len(), 3);
        assert_eq!(result.columns, vec!["id", "name", "email"]);
        assert_eq!(result.rows.len(), 5);
        assert_eq!(result.rows[0][1], "John Doe");
        assert_eq!(result.rows[1][1], "Jane Smith");

        db.teardown()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_executor_join_query() -> Result<()> {
        if !should_run_postgres_tests() {
            return Ok(());
        }

        let db = PostgresTestDb::new();
        db.setup()?;

        let query = r#"
            SELECT u.name, COUNT(o.id) as order_count
            FROM users u
            LEFT JOIN orders o ON u.id = o.user_id
            GROUP BY u.name
            ORDER BY order_count DESC
        "#;

        let executor = PostgresExecutor {
            url: db.connection_string(),
            sql: query.to_string(),
        };

        let result = executor.execute().await?;
        
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.columns, vec!["name", "order_count"]);
        assert!(result.rows.len() > 0);
        
        // Charlie Wilson should have the most orders (3)
        assert_eq!(result.rows[0][0], "Charlie Wilson");
        assert_eq!(result.rows[0][1], "3");

        db.teardown()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_executor_invalid_query() -> Result<()> {
        if !should_run_postgres_tests() {
            return Ok(());
        }

        let db = PostgresTestDb::new();
        db.setup()?;

        let executor = PostgresExecutor {
            url: db.connection_string(),
            sql: "SELECT * FROM nonexistent_table".to_string(),
        };

        let result = executor.execute().await;
        assert!(result.is_err());

        db.teardown()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_executor_multi_statement() -> Result<()> {
        if !should_run_postgres_tests() {
            return Ok(());
        }

        let db = PostgresTestDb::new();
        db.setup()?;

        let multi_query = r#"
            CREATE TEMPORARY TABLE test_temp (id INT, name TEXT);
            INSERT INTO test_temp VALUES (1, 'test1'), (2, 'test2');
            SELECT * FROM test_temp ORDER BY id;
        "#;

        let executor = PostgresExecutor {
            url: db.connection_string(),
            sql: multi_query.to_string(),
        };

        let result = executor.execute().await?;
        
        // We should get the results from the last statement
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][0], "1");
        assert_eq!(result.rows[0][1], "test1");
        assert_eq!(result.rows[1][0], "2");
        assert_eq!(result.rows[1][1], "test2");

        db.teardown()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_executor_parameterized_query() -> Result<()> {
        if !should_run_postgres_tests() {
            return Ok(());
        }

        let db = PostgresTestDb::new();
        db.setup()?;

        // Note: for now, parameters are actually a limitation since we don't have a way to 
        // pass parameter values through CLI. This just confirms the behavior.
        let param_query = "SELECT * FROM users WHERE id = $1";

        let executor = PostgresExecutor {
            url: db.connection_string(),
            sql: param_query.to_string(),
        };

        let result = executor.execute().await;
        
        // This should fail because we haven't provided a value for $1
        assert!(result.is_err());

        db.teardown()?;
        Ok(())
    }

    pub struct PostgresTestDb {
        pub host: String,
        pub port: u16,
        pub user: String,
        pub password: String,
        pub dbname: String,
    }

    impl PostgresTestDb {
        pub fn new() -> Self {
            let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = env::var("POSTGRES_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432);
            let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
            let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
            let dbname = env::var("POSTGRES_TEST_DB").unwrap_or_else(|_| "sqli_test".to_string());

            Self {
                host,
                port,
                user,
                password,
                dbname,
            }
        }

        pub fn connection_string(&self) -> String {
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                self.user, self.password, self.host, self.port, self.dbname
            )
        }

        pub fn setup(&self) -> Result<()> {
            if !self.is_postgres_available() {
                eprintln!("PostgreSQL is not available, skipping database tests");
                return Ok(());
            }

            self.create_database()?;            
            thread::sleep(Duration::from_millis(500));
            self.init_schema()?;

            Ok(())
        }

        pub fn teardown(&self) -> Result<()> {
            if self.is_postgres_available() {
                self.drop_database()?;
            }
            Ok(())
        }

        fn is_postgres_available(&self) -> bool {
            let output = Command::new("psql")
                .args([
                    "--host", &self.host,
                    "--port", &self.port.to_string(),
                    "--username", &self.user,
                    "--command", "SELECT 1;",
                    "postgres"
                ])
                .env("PGPASSWORD", &self.password)
                .output();

            match output {
                Ok(out) => out.status.success(),
                Err(_) => false,
            }
        }

        fn create_database(&self) -> Result<()> {
            let _ = Command::new("psql")
                .args([
                    "--host", &self.host,
                    "--port", &self.port.to_string(),
                    "--username", &self.user,
                    "--command", &format!("DROP DATABASE IF EXISTS {};", self.dbname),
                    "postgres"
                ])
                .env("PGPASSWORD", &self.password)
                .output()?;

            let output = Command::new("psql")
                .args([
                    "--host", &self.host,
                    "--port", &self.port.to_string(),
                    "--username", &self.user,
                    "--command", &format!("CREATE DATABASE {};", self.dbname),
                    "postgres"
                ])
                .env("PGPASSWORD", &self.password)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to create database: {}", error);
            }

            Ok(())
        }

        fn init_schema(&self) -> Result<()> {
            let schema_path = Path::new("./schema.sql");
            if !schema_path.exists() {
                anyhow::bail!("Schema file not found: {:?}", schema_path);
            }

            let schema = fs::read_to_string(schema_path)?;
            
            let output = Command::new("psql")
                .args([
                    "--host", &self.host,
                    "--port", &self.port.to_string(),
                    "--username", &self.user,
                    "--dbname", &self.dbname,
                    "--command", &schema,
                ])
                .env("PGPASSWORD", &self.password)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                anyhow::bail!("Failed to initialize schema: {}", error);
            }

            Ok(())
        }

        fn drop_database(&self) -> Result<()> {
            let _ = Command::new("psql")
                .args([
                    "--host", &self.host,
                    "--port", &self.port.to_string(),
                    "--username", &self.user,
                    "--command", &format!(
                        "SELECT pg_terminate_backend(pg_stat_activity.pid) FROM pg_stat_activity WHERE pg_stat_activity.datname = '{}' AND pid <> pg_backend_pid();",
                        self.dbname
                    ),
                    "postgres"
                ])
                .env("PGPASSWORD", &self.password)
                .output()?;

            let output = Command::new("psql")
                .args([
                    "--host", &self.host,
                    "--port", &self.port.to_string(),
                    "--username", &self.user,
                    "--command", &format!("DROP DATABASE IF EXISTS {};", self.dbname),
                    "postgres"
                ])
                .env("PGPASSWORD", &self.password)
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                eprintln!("Warning: Failed to drop database: {}", error);
            }

            Ok(())
        }
    }

    pub fn should_run_postgres_tests() -> bool {
        if env::var("SKIP_DB_TESTS").is_ok() {
            return false;
        }
        
        let db = PostgresTestDb::new();
        db.is_postgres_available()
    }
}
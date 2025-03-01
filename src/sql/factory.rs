use super::interface::Executor;
use super::postgresql::PostgresExecutor;

pub fn create_executor(url: String, sql: String) -> impl Executor {
    match url.split(":").collect::<Vec<&str>>()[0] {
        "postgresql" => {
            PostgresExecutor {
                url,
                sql,
            }
        }
        _ => {
            PostgresExecutor {
                url,
                sql,
            }
        }
    }
}
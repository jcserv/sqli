use anyhow::Result;

use crate::sql::{factory::create_executor, interface::Executor};

pub async fn run_query(url: String, sql: String) -> Result<()> {
    let executor = create_executor(url, sql);
    executor.execute().await?;
    Ok(())
}
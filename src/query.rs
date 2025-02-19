use anyhow::Result;

use crate::sql::{factory::create_executor, interface::Executor};

pub async fn run_query(url: String, sql: String) -> Result<()> {
    // todo: check if sql is a file, if so, read file contents

    let executor = create_executor(url, sql);
    executor.execute().await?;
    Ok(())
}
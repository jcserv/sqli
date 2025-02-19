use anyhow::Result;

pub trait Executor {
    fn execute(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}
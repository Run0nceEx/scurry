use async_trait::async_trait;

#[async_trait]
pub trait PostProcessor<T> {
    async fn process(&self, unit: T) -> Option<T>;
}

/// Used in scheduler (Command run on)
#[async_trait]
pub trait CRON<R>: Sized {
    /// Run function, and then append to parent if more jobs are needed
    async fn exec(&mut self) -> R;

}


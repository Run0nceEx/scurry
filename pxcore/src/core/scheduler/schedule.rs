use std::time::Duration;
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

    /// check if command should be ran
    fn check(&self) -> bool;

    /// check if reschedule is needed
    fn reschedule(&mut self) -> bool {
        false
    }

    fn max_reschedule(&self) -> usize {
        32
    }

    /// time to live - default time is 1 minute
    fn ttl(&self) -> Duration {
        Duration::from_secs(60)
    }

}


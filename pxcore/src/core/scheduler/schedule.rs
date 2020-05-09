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

    fn max_reschedule(&self) -> usize {
        32
    }

    /// time to live (default: 1 minute) - this is essentially a time mechanism
    fn ttl(&self) -> Duration {
        Duration::from_secs(60)
    }

    /// job sleeping duration - if failure occurs, it will be send to this 
    fn reschedule_duration(&self) -> Duration {
        Duration::from_secs(60)
    }
}


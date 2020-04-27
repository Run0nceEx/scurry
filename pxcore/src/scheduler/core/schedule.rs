use std::time::Duration;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

pub enum Error {}


/// Save Self into a "loadable"/serialized form 
pub trait ScheduleLedger<'a, T: Serialize + Deserialize<'a>> {
    fn save(&self) -> Result<(), Error>;
    fn load(loader: T) -> Self;
}


#[async_trait]
pub trait PostProcessor<T> {
    async fn process(&self, unit: T) -> Option<T>;
}

/// Command run on (CRON)
#[async_trait]
pub trait CRON<R>: Sized {
    /// Run function, and then append to parent if more jobs are needed
    async fn exec(self) -> R;

    /// check if command should be ran
    fn check(&self) -> bool;

    /// time to live - default time is 1 minute
    fn ttl(&self) -> Duration {
        Duration::from_secs(60)
    }
}

/// Used to specify thread scheduler
pub trait ScheduleExecutor {
    fn run(&mut self);
}



mod core;

pub mod meta;
pub mod pool;


pub use crate::schedule::core::*;
use crate::error::Error;

use std::time::Duration;


#[derive(Debug, Copy, Clone)]
pub enum SignalControl {
    /// Operations went according to plan, 
    Success,
    
    /// and requesting to be reschedule again
    Reschedule(Duration),

    /// Operations failed and would like to attemp again without a specified time
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,

    Fuck,
}

/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: Sized + std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error>;

    fn name() -> String;
}


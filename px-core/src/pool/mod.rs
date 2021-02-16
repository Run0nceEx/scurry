#[cfg(test)]
#[cfg(feature="include-tests")]
pub mod test;

pub mod worker;
pub use worker::*;

mod pool;
pub use pool::Pool;

mod stash;


use crate::error::Error;
/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {

    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<JobCtrl<Self::Response>, Error>;
}

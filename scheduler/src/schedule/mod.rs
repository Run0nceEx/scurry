mod core;
pub mod sugar;

pub use crate::schedule::core::*;
pub use crate::schedule::core::CronMeta;


/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: Sized {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: Self::State) -> (Self::Response, Self::State);
}


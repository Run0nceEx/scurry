mod core;
mod sig;

pub mod meta;
pub mod sugar;

pub use crate::schedule::core::*;
pub use sig::*;
use crate::error::Error;


pub type FuckinNonSense = SignalControl<Option<()>>;

/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: Sized {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: Self::State) -> Result<SignalControl<(Option<Self::Response>, Self::State)>, Error>;
}


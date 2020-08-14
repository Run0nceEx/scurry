use crate::error::Error;

/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SignalControl {
    /// Drop memory, and give a boolean to tell if we connected 
    Success(bool), // Boolean to signify to the scheduler if we connected to the target or not
    
    /// Operations failed and would like to attemp again, 
    /// it will sleep again for whatever it's time to sleep paramenter was set to. (tts)
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,

    Stash(std::time::Duration)
}


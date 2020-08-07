
use std::cmp::Ordering;
use crate::error::Error;
use std::time::Duration;


/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error>;

    fn name() -> String;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SignalControl {
    /// Drop memory, and give a boolean to tell if we connected 
    Success(bool), // Boolean to signify to the scheduler if we connected to the target or not
    
    /// and requesting to be reschedule again
    Reschedule(Duration),

    /// Operations failed and would like to attemp again, 
    /// it will sleep again for whatever it's time to sleep paramenter was set to. (tts)
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,
}


impl Ord for SignalControl {
    fn cmp(&self, other: &Self) -> Ordering {
        match other {
            other if other == self => Ordering::Equal,
            SignalControl::Reschedule(_) | SignalControl::Retry => {
                match self {
                    SignalControl::Drop | SignalControl::Success(_) => Ordering::Greater,
                    SignalControl::Retry | SignalControl::Reschedule(_) => Ordering::Equal,
                }
            }
            
            SignalControl::Drop | SignalControl::Success(_) => {
                match self {
                    SignalControl::Drop | SignalControl::Success(_) => Ordering::Equal,
                    SignalControl::Retry | SignalControl::Reschedule(_) => Ordering::Less,
                }
            }
        }
    }
}

impl PartialOrd for SignalControl {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unimplemented!()
    }
}
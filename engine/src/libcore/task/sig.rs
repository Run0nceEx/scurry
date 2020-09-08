use crate::libcore::model::State;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SignalControl<R> {
    /// Drop memory, and give a boolean to tell if we connected 
    Success(State, R), // Boolean to signify to the scheduler if we connected to the target or not
    
    /// Operations failed and would like to attemp again, 
    /// it will sleep again for whatever it's time to sleep paramenter was set to. (tts)
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,

    Stash(std::time::Duration)
}
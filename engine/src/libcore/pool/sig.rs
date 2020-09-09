use crate::libcore::model::State;


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SignalControl<R> {

    //result<(State, R), Error>
    /// Drop memory, and give a boolean to tell if we connected 
    Success(State, R), // Boolean to signify to the scheduler if we connected to the target or not
    
    // (Error
    Stash(std::time::Duration),


    // --- 
    // on a side note if it contains the current retry count for the object
    // we could eliminate the meta component and use signal control for book keeping purposes

    /// Operations failed and would like to attemp again, 
    /// it will sleep again for whatever it's time to sleep paramenter was set to. (tts)
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,
    
}
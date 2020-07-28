#[derive(Debug, Clone)]
pub enum NetworkState {
    /// Everything seems to be working
    Ok,
    
    /// Assumes we can only contact talk locally
    LocalOnly,

    /// Assumes we can't reach anything 
    Failure,
    
    /// We believe that a failure has occured, 
    /// and this state will be temporarily kept
    /// until investigations are resolved 
    CheckingFailure {
        neg: usize,
        pos: usize,
        avg_duration: Duration
    }
}


pub struct MainState {
    netstate: NetworkState
}

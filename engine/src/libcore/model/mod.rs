use serde::Serialize;
use std::net::IpAddr;

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
pub enum State {
    Closed,
    Filtered,
    Open,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            State::Closed => "closed",
            State::Open => "open",
            State::Filtered => "filtered"
        };
        
        write!(f, "{}", x)?;
        Ok(())
    }
}


#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize)]
pub struct Service {
    pub port: u16,
    pub state: State,
}



#[derive(Debug, Clone)]
pub struct Host {
    pub addr: IpAddr,
    pub services: smallvec::SmallVec<[Service; 32]>
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortInput {
    Singleton(u16),
    Range(std::ops::Range<u16>)
}

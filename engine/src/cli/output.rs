use std::net::SocketAddr;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
enum Format {
    Stdout,
    Json
}

impl Format {
    pub fn fmt(&self, data: &OutputEntry) -> Result<String, super::error::Error> {
        match self {
            Format::Stdout => unimplemented!(),
            Format::Json => Ok(serde_json::to_string(data)?)
        }
    }
}


#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
pub enum State {
    Closed,
    Filtered,
    Open
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            State::Closed => "closed",
            State::Open => "open",
            State::Filtered => "filtered"
        };
        
        write!(f, "{}", x);
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
pub struct Service {
    port: u16,
    state: State,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
pub struct OutputEntry<'a> {
    addr: SocketAddr,
    services: &'a [Service]
}


use serde::Serialize;
use super::error::{Error};

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

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Serialize)]
pub struct ServiceTest<T: Serialize> {
    pub port: u16,
    pub state: State,
    pub data: T
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortInput {
    Singleton(u16),
    Range(std::ops::Range<u16>)
}

impl PortInput {
    pub fn contains(&self, other: u16) -> bool {
        match self {
            PortInput::Singleton(s) => s.eq(&other),
            PortInput::Range(rng) => rng.contains(&other)
        }
    }
}

impl std::str::FromStr for PortInput {
    type Err = Error;

    /// Accepts '80-443', '80', '0-10'
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        port_parser(src)
    }
}

pub fn port_parser(src: &str) -> Result<PortInput, Error> {
    let data: Vec<&str> = src.split("-").collect();
    let mut bottom = data.get(0).unwrap().parse::<u16>()?;

    if data.len() > 2 {
        return Err(Error::ParseError("port ranges (-p 20-25) can only be in groups of 2 (not 20-25-30)".to_string()))
    }
    
    if data.len() == 2 {
        let mut top = data.get(1).unwrap().parse::<u16>()?;
        if bottom >= top {
            // swaps addresses
            std::mem::swap(&mut bottom, &mut top);
        }
        return Ok(PortInput::Range(bottom..top))
    }

    Ok(PortInput::Singleton(bottom))
}

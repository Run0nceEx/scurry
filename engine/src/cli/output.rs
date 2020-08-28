use std::net::IpAddr;
use serde::Serialize;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
enum Format {
    Stdout,
    Json
}

impl Format {
    pub fn fmt(&self, data: &Host) -> Result<String, super::error::Error> {
        match self {
            Format::Stdout => {
                let mut service_info = data.services
                    .iter()
                    .map(|s| format!("\t{}\t{}", s.port, s.state))
                    .collect::<Vec<_>>()[..]
                    .join("\n");
                
                Ok(format!("{}\n{}", data.addr, service_info))
            },
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

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize)]
pub struct Service {
    port: u16,
    state: State,
    packet_protocol: String,
    layer_protocol: String,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize)]
pub struct Host {
    addr: IpAddr,
    services: smallvec::SmallVec<[Service; 32]>
}



use crate::error::Error;

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    TCP,
    UDP
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::TCP
    }
}

use std::str::FromStr;
impl FromStr for Protocol {
    type Err = Error;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x {
            "udp" | "UDP" => Protocol::UDP,
            "tcp" | "TCP" => Protocol::TCP,
            _ => return Err(Error::ParseError("Got \"{}\" instead of tcp or udp".to_string())) 
        })
    }
}
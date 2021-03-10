use std::{
    str::FromStr,
    time::Duration
};

use px_core::model::PortInput;
use logos::{Logos, Lexer};

use crate::error::Error;
use super::{
    cpe::Identifier,
    match_expr::MatchLineExpr,
};


#[derive(Debug)]
pub struct ZeroDuration(pub Duration);
impl Default for ZeroDuration {
    fn default() -> Self {
        Self(Duration::from_millis(0))
    }
}

impl ZeroDuration {
    pub fn into_inner(self) -> Duration {
        self.0
    }
}

#[derive(Default, Debug)]
pub struct ProbeExpr {
    pub proto: Protocol,
    pub payload: String,
    pub payload_delimiter: char,
    pub rarity: u8,
    pub load_ord: usize,
    pub name: String,
    pub ports: Vec<PortInput>,
    pub exclude: Vec<PortInput>,
    pub tls_ports: Vec<PortInput>,
    pub matches: Vec<MatchLineExpr>,
    pub fallback: Option<String>,
    pub wait_total_ms: ZeroDuration,
    pub wait_wrapped_ms: ZeroDuration,
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum Token {
    // Tokens can be literal strings, of any length.
    #[token("softmatch")]
    #[token("match")]
    Match,
    
    #[regex("[# ]*NEXT PROBE[# ]*")]
    EndProbe,

    #[token("Probe")]
    Probe,
    
    #[token("tcpwrappedms")]
    WrappedWaitMs,

    #[token("ssl_ports")]
    SslPorts,

    #[token("ports")]
    Ports,

    #[token("totalwaitms")]
    TotalWaitMs,

    #[token("Exclude")]
    Exclude,

    #[regex("[0-9]+-[0-9]+")]
    Rng,

    #[regex("[0-9]+", priority = 2)]
    Num,

    #[regex("[a-zA-Z0-9]+", priority = 3)]
    Word,
    
    #[error]
    #[regex(r"[\t\n\f\r ]+", logos::skip)]
    Error,
}
// const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

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

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x.trim() {
            "udp" | "UDP" => Protocol::UDP,
            "tcp" | "TCP" => Protocol::TCP,
            x => return Err(
                Error::ParseError(format!("Got \"{}\" instead of 'tcp' or 'udp'", x))
            ) 
        })
    }
}


#[derive(Debug, Copy, Clone)]
pub enum Directive {
    SoftMatch,
    Match
}


impl FromStr for Directive {
    type Err = Error;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x.trim() {
            "softmatch" => Directive::SoftMatch,
            "match" => Directive::Match,
            _ => return Err(
                Error::ExpectedToken(Token::Match)
            ) 
        })
    }
}
use px_core::{
    model::PortInput,
    // netlib::parsers::nmap::Error
};
use crate::error::{Error, ErrorKind};

use std::str::FromStr;

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
    type Err = ErrorKind;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x.trim() {
            "udp" | "UDP" => Protocol::UDP,
            "tcp" | "TCP" => Protocol::TCP,
            _ => return Err(ErrorKind::UnknownToken("Got \"{}\" instead of tcp or udp".to_string())) 
        })
    }
}


#[derive(Debug, Copy, Clone)]
pub enum Directive {
    SoftMatch,
    Match
}


impl FromStr for Directive {
    type Err = ErrorKind;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x.trim() {
            "softmatch" => Directive::SoftMatch,
            "match" => Directive::Match,
            _ => return Err(ErrorKind::UnknownToken("Got \"{}\" instead of match or softmatch".to_string())) 
        })
    }
}


// #[derive(Debug)]
// struct Tunnel(String);

#[derive(Debug, PartialEq, Eq)]
pub enum Flags {
    UNIT,
    CaseSensitive,
    IgnoreWhiteSpace    
}

#[derive(Debug)]
pub struct MatchLineExpr {
    // with the syntax being m/[regex]/[opts].
    // The “m” tells Nmap that a match string is beginning. 
    // The forward slash (/) is a delimiter,
    // which can be substituted by almost any printable character as long
    // as the second slash is also replaced to match.
    // The regex is a Perl-style regular expression.
    // This is made possible by the
    // excellent Perl Compatible Regular Expressions (PCRE) library
    // (http://www.pcre.org). 
    pub pattern: String,
    pub flags: [Flags; 2],
    pub match_type: Directive,
    pub name: String,
    pub service_data: String,
    pub cpe: String
    //tunneled: Option<Tunnel>,
}

#[derive(Default, Debug)]
pub struct ProbeExpr {
    pub proto: Protocol,
    pub payload: String,
    pub rarity: u8,
    pub load_ord: usize,
    pub name: String,
    pub ports: Vec<PortInput>,
    pub exclude: Vec<PortInput>,
    pub tls_ports: Vec<PortInput>,
    pub patterns: Vec<MatchLineExpr>,
    pub fallback: Option<String>,
}
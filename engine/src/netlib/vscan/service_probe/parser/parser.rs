use crate::{
    model::PortInput
};
use super::{
    super::{
        common::CPE,
    },
    error::Error
};
use std::{
    io::{Read, BufReader, BufRead},
    str::FromStr
};


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
        Ok(match x {
            "udp" | "UDP" => Protocol::UDP,
            "tcp" | "TCP" => Protocol::TCP,
            _ => return Err(Error::UnknownToken("Got \"{}\" instead of tcp or udp".to_string())) 
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
        Ok(match x {
            "softmatch" => Directive::SoftMatch,
            "match" => Directive::Match,
            _ => return Err(Error::UnknownToken("Got \"{}\" instead of match or softmatch".to_string())) 
        })
    }
}

// with the syntax being m/[regex]/[opts].
// The “m” tells Nmap that a match string is beginning. 
// The forward slash (/) is a delimiter,
// which can be substituted by almost any printable character as long
// as the second slash is also replaced to match.
// The regex is a Perl-style regular expression.
// This is made possible by the
// excellent Perl Compatible Regular Expressions (PCRE) library
// (http://www.pcre.org). 

#[derive(Debug)]
pub struct MatchLineExpr {
    pub pattern: String,
    pub match_type: Directive,
    pub name: String,
    pub cpe: CPE
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


const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

pub fn parse_txt_db<T: Read>(fd: &mut BufReader<T>, expressions: &mut Vec<ProbeExpr>, intensity: u8) -> Result<(), Error> {
    let mut line_buf = String::new();
    let mut entity = ProbeExpr::default();
    // parse line by line
    while fd.read_line(&mut line_buf)? > 0 {
        // if probe delimiter reached, attempt to make a `ProbeEntry` out of `ProbeExpr`
        if line_buf.len() == 0 {
            continue
        }
        
        if line_buf.contains(&DELIMITER) {
            if entity.name.len() > 0 && entity.payload.len() > 0 {
                expressions.push(entity);
                entity = ProbeExpr::default();
            }
        }
        
        else if line_buf.starts_with("#") {
            continue
        }
        
        // Cut line where comment begins
        else if line_buf.contains("#") {
            let tail_position = line_buf.chars()
                .take_while(|c| *c != '#')
                .enumerate()
                .map(|(i, c)| i)
                .last()
                .unwrap();
            
            
            if tail_position >= 1 { // safety check (under-flow unsigned)
                let slice = &line_buf[..tail_position-1];
                line_buf = slice.to_string();
            }
        }
        
        // fuck me i hate parsing code ()
        // this is how we're tokenizing i guess
        // dont really feel like building a whole lexer+expr tree
        let mut tokens = line_buf.split_whitespace();        
        
        let first_token = tokens.next().ok_or_else(||Error::ExpectedToken)?;

        if first_token.eq("Probe") {
            let protocol = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;
            
            let name = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;
            
            let payload = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;

            if entity.name.len() > 0 || entity.payload.len() > 0 {
                eprintln!("probe set previously and over written {} -> {}", entity.name, name);
            }

            entity.name = name.to_string();
            entity.proto = Protocol::from_str(protocol)?;
            entity.payload = payload.to_string();
        }

        else if first_token.eq("rarity") {
            entity.rarity = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?
                .parse()?;
        }

        else if first_token.eq("softmatch") | first_token.eq("match") {
            let name = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;
            let partial_query = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;
            
            let mut cursor = partial_query.chars();
            let regex_char = cursor.next().ok_or_else(|| Error::ExpectedToken)?;
            
            if regex_char == 'm' {
                let delimiter = cursor.next()
                    .ok_or_else(|| Error::ExpectedToken)?;
                
                let mut regex_cursor = line_buf.split(delimiter);
            }
            else {
                // syntax error?
                // match <name> m<pattern> [<version> ...]
                unimplemented!()
            }
            
            // MatchLineExpr {
            //     name,
            //     match_type: Directive::from_str(first_token)?,
                
            // }
            // entity.patterns.push();
        }

        else if first_token.eq("match") {}
        else if first_token.eq("ports") {}
        else if first_token.eq("sslports") {}
        else if first_token.eq("totalwaitms") {}
        else if first_token.eq("fallback") {}


        line_buf.clear();
    }

    Ok(())
}

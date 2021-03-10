use super::parser::{Directive, MatchLineExpr};

use regex::bytes::RegexSet;

use crate::error::Error;

// match http-proxy-ctrl m|^WWWOFFLE Server Status\n-*\nVersion *: (\d.*)\n| p/WWWOFFLE proxy control/ v/$1/ cpe:

#[derive(Debug)]
enum Ident {
    Application,
    Hardware,
    OperationSystem
}

impl Ident {
    fn from_char(c: char) -> Result<Ident, Error> {
        Ok(match c {
            'a' => Ident::Application,
            'h' => Ident::Hardware,
            'o' => Ident::OperationSystem,
            c => return Err(
                unimplemented!()
                //Error::UnknownToken(format!("Expected char 'a', 'h', or 'o', got '{}'", c))
            )
        })
    }
}

#[derive(Debug)]
pub struct Service(String);
impl Service {
    // #![feature(toowned_clone_into)]
    pub fn new(mut data: String) -> Self {
        data.shrink_to_fit();
        Self(data)
    }

    fn getter<'a>(&'a self, index: usize) -> Option<&'a str> {
        match self.0.split_terminator(':').nth(index) {
            None => None,
            Some(data) => {
                if data.len() > 0 {
                    Some(data)
                }
                else { None }
            }
        }
    }


    pub fn as_arr(&self) -> [Option<&str>; 5] {
        let mut buf = [None; 5];
        for i in 0..5 {
            buf[i] = self.getter(i);
        }
        buf
    }

    pub fn vendor(&self) -> Option<&str> {
        self.getter(0)
    }

    pub fn product(&self) -> Option<&str> {
        self.getter(1)
    }

    pub fn version(&self) -> Option<&str> {
        self.getter(2)
    }

    pub fn update(&self) -> Option<&str> {
        self.getter(3)
    }

    pub fn edition(&self) -> Option<&str> {
        self.getter(4)
    }

    pub fn language(&self) -> Option<&str> {
        self.getter(5)
    }
}


#[derive(Debug)]
struct CPE {
    pub part: Ident,
    pub data: Service
}


#[derive(Debug)]
pub struct MatchExpr {
    directive: Directive,
    name: String,
    data: Service,
    cpe: Service
}

impl MatchExpr {
    pub fn new(cpe: Service, name: String, data: Service, directive: Directive) -> Self {
        Self {
            cpe,
            data,
            name,
            directive
        }
    }
}

/// This data structure is used for matching regex patterns expressed inside `nmap-service-probes`.
/// It allows us to store regex patterns inside a set, and their respective partner as 
/// 
/// this grouping is aligned so that indexes in `self.patterns` also correlate to `self.map`
/// where information about the response's data capture
#[derive(Debug)]
pub struct AlignedSet {
    patterns: RegexSet,
    map: Vec<MatchExpr>
}

impl AlignedSet {
    pub fn match_response<'a>(&'a self, input_buf: &[u8], out_buf: &mut Vec<&'a MatchExpr>) {
        self.patterns
            .matches(input_buf)
            .into_iter()
            .for_each(|i| {
                out_buf.push(self.map.get(i).unwrap())
            });
    }

    // new(cpe: CPE, name: String, directive: Directive)
    pub fn new(patterns: &[MatchLineExpr]) -> Result<AlignedSet, regex::Error> {
        // align two buffers so that RegexSet's index correlates with
        // 
        // -- self.patterns
        // -- self.cpe_lookup
            
        let mut regex_buf = Vec::new();
        let mut mapping = Vec::new();
        
        for item in patterns {
            regex_buf.push(item.pattern.clone());
            //TODO
            // mapping.push(MatchExpr::new(
            //     Service(item.cpe.clone()),
            //     item.name.clone(),
            //     Service::new(item.service_data.clone()),
            //     item.match_type,
            // ));
        }

        regex_buf.shrink_to_fit(); 
        mapping.shrink_to_fit();
        
        Ok(AlignedSet {
            patterns: RegexSet::new(regex_buf)?,
            map: mapping
        })
    }
}

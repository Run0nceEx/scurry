use crate::model::PortInput;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    str::FromStr,
    time::Duration,
};
use regex::bytes::RegexSet;
use super::engine::{CPE};

#[derive(Debug)]
enum Error {
    IO(std::io::Error),
    ParsingError {
        line_count: usize,
        //token: Token
    }
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Error {
        Error::IO(x)
    }
}


// impl std::fmt::Display for Error {
 
// }

/*
##############################NEXT PROBE##############################
# Ubiquiti Discovery Protocol
Probe UDP UbiquitiDiscoveryv2 q|\x02\x08\0\0|
rarity 9
ports 10001

# Valid response is protocol version (\x02 ) and cmd followed
# by 2 bytes of length then TLV groups
# Known cmd values are \x06, \x09, and \x0b
match ubiquiti-discovery m|^\x02[\x06\x09\x0b].[^\0].*\x15\0.([\w-]+)\x16\0.([\d.]+)|s p/Ubiquiti Discovery Service/ i/v2 protocol, $1 software ver. $2/
match ubiquiti-discovery m|^\x02[\x06\x09\x0b].[^\0].*\x15\0.([\w-]+)|s p/Ubiquiti Discovery Service/ i/v2 protocol, $1/
softmatch ubiquiti-discovery m|^\x02[\x06\x09\x0b].[^\0].{48}|s p/Ubiquiti Discovery Service/ i/v2 protocol/
*/

/*
##############################NEXT PROBE##############################
Probe TCP HTTPOptions q|OPTIONS / HTTP/1.0\r\n\r\n|
ports 80-85,2301,631,641,3128,5232,6000,8080,8888,9999,10000,10031,37435,49400
sslports 443,4443,8443
fallback GetRequest

match apollo-server m=^0000000001(?:3C|C0)0000$= p/Apollo Server database access/ cpe:/o/($P1)

---------
##############################NEXT PROBE##############################
Probe <Protocol> <Probe-Name> q|<send-data>|
rarity <num>

match|softmatch <resp-name> m<recv-pattern> [<cpe>]

*/

#[derive(Debug)]
pub enum Protocol {
    TCP,
    UDP
}

#[derive(Debug)]
enum MatchType {
    SoftMatch,
    Match
}

#[derive(Debug)]
struct MatchLineExpr {
    pattern: String,
    match_type: MatchType,
    name: String,
    cpe: CPE
}


impl Default for Protocol {
    fn default() -> Self {
        Protocol::TCP
    }
}


#[derive(Default, Debug)]
struct ProbeExpr {
    proto: Protocol,// TCP/UDP
    payload: String,
    rarity: u8,
    name: String,
    ports: Vec<PortInput>,
    exclude: Vec<PortInput>,
    tls_ports: Vec<PortInput>,
    patterns: Vec<MatchLineExpr>,
    fallback: Option<String>,

}


#[derive(Default, Debug)]
pub struct Match {
    name: String,
    cpe: CPE
}

#[derive(Debug)]
pub struct Probe {
    proto: Protocol,// TCP/UDP
    payload: Vec<u8>,
    rarity: u8,
    //name: String,
    ports: Vec<PortInput>,
    exclude: Vec<PortInput>,
    tls_ports: Vec<PortInput>,

    //so here we have a flat map where we'll do a quick match on, 
    //where we get a collection of indexes matched, we'll take those 
    lookup_set: AlignedSet,
}

impl Probe {
    fn matches<'a>(&'a self, input_buf: &[u8], out_buf: &mut Vec<&'a Match>) {
        self.lookup_set.match_response(input_buf, out_buf)
    }
}

struct ProbeEntry {
    inner: Probe,
    fallback: Option<String>
}

pub struct LinkedProbes(HashMap<String, ProbeEntry>);



const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

fn parse_txt_db<T: Read>(fd: &mut BufReader<T>) -> Result<LinkedProbes, Error> {
    
    let mut linker_buf = Vec::new();
    
    let mut line_buf = String::new();
    let mut entity = ProbeExpr::default();
    // parse line by line
    while fd.read_line(&mut line_buf)? > 0 {
        // if probe delimiter reached, attempt to make a `ProbeEntry` out of `ProbeExpr`
        if line_buf.contains(&DELIMITER) {
            if entity.name.len() > 0 && entity.payload.len() > 0 {
                linker_buf.push(construct_probe(entity));
                entity = ProbeExpr::default();
            }
        }

        line_buf.clear();
    }

    Ok(link_probes(&mut linker_buf))
}


fn link_probes(buf: &mut Vec<(String, Option<String>, Probe)>) -> LinkedProbes {
    let mut map = HashMap::new();
    for item in buf.drain(..) {
        let name = item.0;
        let entry = ProbeEntry {
            fallback: item.1,
            inner: item.2
        };

        if let Some(x) = map.insert(name, entry) {
            dbg!("got duplicate key in probe names: {}", x);
        }
    }
    
    
    map.iter().filter_map(|(k, v)| {
        // Drop anything that links to nothing
        if let Some(fallback) = v.fallback {
            if k.len() <= 2 || k.len() > 64 {
                return None;
            }

            if !map.contains_key(&fallback) {
                dbg!("fallback doesn't exist: {} -> {}", k, fallback);
                return None;
            }
            
            return Some((k, v));
        }

        return Some((k, v))
    });

    LinkedProbes(map)
}

#[derive(Debug)]
struct AlignedSet {
    set: RegexSet,
    map: Vec<Match>
}

impl AlignedSet {
    pub fn match_response<'a>(&'a self, input_buf: &[u8], out_buf: &mut Vec<&'a Match>) {
        let indexes: Vec<_> = self.set
            .matches(input_buf)
            .into_iter()
            .collect();
        
        for i in indexes {
            out_buf.push(self.map.get(i).unwrap());
        }

    }
}

fn align_regex_set(patterns: &[MatchLineExpr]) -> Result<AlignedSet, regex::Error> {
    // ALIGNED INDEXES
    // -- self.patterns
    // -- self.cpe_lookup
    
    let mut regex_buf = Vec::new();
    let mut mapping = Vec::new();

    for item in patterns {
        regex_buf.push(item.pattern);
        
        mapping.push(Match {
            name: item.name,
            cpe: item.cpe
        })
    }

    Ok(AlignedSet {
        set: RegexSet::new(regex_buf)?,
        map: mapping
    })
}

#[inline]
fn construct_probe(entity: ProbeExpr) -> (String, Option<String>, Probe) 
{
    let aset = align_regex_set(&entity.patterns).unwrap();
    let payload = construct_payload(&entity.payload);
    let probe = Probe {
        proto: entity.proto,
        rarity: entity.rarity,
        ports: entity.ports,
        exclude: entity.exclude,
        tls_ports: entity.tls_ports,
        lookup_set: aset,
        payload
    };
    (entity.name, entity.fallback, probe)
}


fn construct_payload(x: &str) -> Vec<u8> {
    unimplemented!()
}

fn assert_fallbacks() {

}



/*
    SF-Port21-TCP:V=3.40PVT16%D=9/6%Time=3F5A961C%r(NULL,3F,"220\x20stage\x20F
    SF:TP\x20server\x20\(Version\x202\.1WU\(1\)\+SCO-2\.6\.1\+-sec\)\x20ready\
    SF:.\r\n")%r(GenericLines,81,"220\x20stage\x20FTP\x20server\x20\(Version\x
    SF:202\.1WU\(1\)\+SCO-2\.6\.1\+-sec\)\x20ready\.\r\n500\x20'':\x20command\
    SF:x20not\x20understood\.\r\n500\x20'':\x20command\x20not\x20understood\.\
    SF:r\n");

For those who care, 

the information in the fingerprint above is port number (21), 
protocol (TCP), Nmap version (3.40PVT16), 
date (September 6), Unix time in hex, 
and a sequence of probe responses in the 
form r({<probename>}, {<responselength>}, "{<responsestring>}").

*/
struct ServiceFingerPrint {

}

// needs to take all the probes and link their fallbacks correctly
// fn linker<T: BufRead>(probes: Vec<ParsedProbe>) {

// }
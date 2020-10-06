use crate::model::PortInput;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read}
};
use regex::Regex;

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

// god i love null ptr optimization
struct CPE {
    part: String,
    vendor: String,
    product: String,
    OS: String,

    
    // info: String>,
    // table: Option<String>,
}

enum RegexCapture {
    SoftMatch(String, Option<CPE>),
    Match(String, Option<CPE>)
}

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

match|softmatch <name> m<recv-pattern> [<version-info>]

*/


enum Protocol {
    TCP,
    UDP,
}

struct ParsedProbe {
    proto: Protocol,// TCP/UDP
    name: String,
    rarity: u8,

    ports: Vec<PortInput>,
    sslports: Vec<PortInput>,
                                        
    response_names: HashMap<String, Vec<String, >>,
    //fallback: Option<Rc<RefCell<Probe>>>
}


struct PartialParsedProbe {
    proto: Option<Protocol>,// TCP/UDP
    rarity: Option<u8>,
    
    name: String,
    ports: Vec<PortInput>,
    sslports: Vec<PortInput>,                                    
    response_names: HashMap<String, Vec<Regex>>,
}



fn parse_file<T: Read>(fd: &mut BufReader<T>) -> Vec<ParsedProbe> {
    const DELIMITER: &'static str = "\
    ##############################\
    NEXT PROBE\
    ##############################";
    
    let mut line_buf = String::new();
    let mut result = Vec::new();
    let mut working_entity: ParsedProbe; 

    while let Ok(n) = fd.read_line(&mut line_buf) {
        if n == 0 || line_buf.contains(DELIMITER) { break }
        
        
        
        line_buf.clear()
    }
        result
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
fn linker<T: BufRead>(probes: Vec<ParsedProbe>) {

}
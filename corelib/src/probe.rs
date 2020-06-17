/*
(https://nmap.org/book/vscan-fileformat.html)
# boolean evaluation only

EVAL:  Table 7.1. versioninfo field formats and values
  p/vendorproductname/
  v/version/
  i/info/
  h/hostname/
  o/operatingsystem/
  d/devicetype/
  cpe:/cpename/[a]


FUNC:
  version <arg>             -- declare required version
  pausems|timeoutms <duration>
  match(*engine)            -- match response to regex - if no layer any
  softmatch                 -- match aggressively
 
sample:

  *version 1.0
  
  Probe TCP socks5 x05\x00\x00
  Probe UDP socks5 x04\x04\x02

  rarity 2
  ports 8888

  #                                        setup info
  match(*prce2) socks5 m|^\x05\x00\x01.?+| p/Socks5
  match backdoor m|\x00\x01|i p/Generic Kibuv worm/ i/BACKDOOR/ o/Windows/ cpe:/o:microsoft:windows/a

  
*/

use smallvec::SmallVec;
use std::collections::HashMap;

struct CPE {
    part: String,
    vendor: String,
    product: String,
    version: String,
    update: String,
    edition: String,
    language: String
}

// impl<'a, 'b> CPE<'a, 'b> {

//   fn parse(signature: &'a str) -> Self {
//     signature.split(":");
//   }

//   fn new(signature: &'a str) -> Self {
//     Self {
//       signature,
//       ..
//     }
//   }
// }

enum Port {
    SSLPort {
      number: u16,
      func: Box<Fn(Vec<u8>) -> Vec<u8>>
    },
    Port(u16),
    Range(u16, u16)
}


enum Layers {
  TCP,
  UDP,
  Unknown(String)
}

enum RE {
  Default
}

struct Entry {
    protocol_name: String,
    map: HashMap<Layers, (SmallVec<[u8; 512]>, Vec<(RE, String)>)>,
    ports: SmallVec<[Port; 32]>,

    rarity: u8,                         // Negative bias             
    cpe: CPE                            // Common platform enumeration
}

/*
                                                                                                                                       
##############################NEXT PROBE##############################                                                                 
# LinuxSampler Control Protocol
# https://www.linuxsampler.org/api/draft-linuxsampler-protocol.html 
Probe TCP LSCP q|GET SERVER INFO\r\n|
rarity 9
ports 8888

match lscp m|^DESCRIPTION: LinuxSampler - modular, streaming capable sampler\r\nVERSION: ([\d.]+)\r\nPROTOCOL_VERSION: ([\d.]+)\r\n| p/
LinuxSampler/ v/$1/ i/LSCP $2/ cpe:/a:linuxsampler:linuxsampler:$1/ 


                                                                                                          
##############################NEXT PROBE##############################                                                                 
# LinuxSampler Control Protocol
# https://www.linuxsampler.org/api/draft-linuxsampler-protocol.html 
Probe <LAYERS> <ANNOUNCE>
rarity 9
ports 8888

<OPERATORS>

match lscp m|^DESCRIPTION: LinuxSampler - modular, streaming capable sampler\r\nVERSION: ([\d.]+)\r\nPROTOCOL_VERSION: ([\d.]+)\r\n| p/
LinuxSampler/ v/$1/ i/LSCP $2/ cpe:/a:linuxsampler:linuxsampler:$1/ 





*/
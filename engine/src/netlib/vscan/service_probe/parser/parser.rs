use crate::netlib::vscan::common::Protocol;
use super::super::common::CPE;

use crate::model::PortInput;

use std::io::{Read, BufReader};

#[derive(Debug, Copy, Clone)]
pub enum Directive {
    SoftMatch,
    Match
}


#[derive(Debug)]
pub struct MatchLineExpr {
    pub pattern: String,
    pub match_type: Directive,
    pub name: String,
    pub cpe: CPE
}

#[derive(Default, Debug)]
pub struct ProbeExpr {
    pub proto: Protocol, // TCP/UDP
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



// fn align_regex_set(patterns: &[MatchLineExpr]) -> Result<AlignedSet, regex::Error> {
//     // align two buffers so that RegexSet's index correlates with
//     // 
//     // -- self.patterns
//     // -- self.cpe_lookup
    
//     let mut regex_buf = Vec::new();
//     let mut mapping = Vec::new();

//     for item in patterns {
//         regex_buf.push(item.pattern.clone());
        
//         mapping.push(Match {
//             name: item.name.clone(),
//             cpe: item.cpe.clone()
//         })
//     }

//     Ok(AlignedSet {
//         patterns: RegexSet::new(regex_buf)?,
//         map: mapping
//     })
// }

// const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

// fn parse_txt_db<T: Read>(fd: &mut BufReader<T>, intensity: u8) -> Result<LinkedProbes, Error> {
//     let mut linker_buf = Vec::new();    
//     let mut line_buf = String::new();
//     let mut entity = ProbeExpr::default();
//     // parse line by line
//     while fd.read_line(&mut line_buf)? > 0 {
//         // if probe delimiter reached, attempt to make a `ProbeEntry` out of `ProbeExpr`
//         if line_buf.contains(&DELIMITER) {
//             if entity.name.len() > 0 && entity.payload.len() > 0 {
//                 linker_buf.push(construct_probe(entity));
//                 entity = ProbeExpr::default();
//             }
//         }
//         line_buf.clear();
//     }
//     Ok(LinkedProbes::construct(linker_buf, intensity))
// }

// #[inline]
// fn construct_probe(entity: ProbeExpr) -> Probe 
// {
//     let aset = align_regex_set(&entity.patterns).unwrap();
//     let payload = construct_payload(&entity.payload);
//     Probe {
//         proto: entity.proto,
//         rarity: entity.rarity,
//         load_ord: entity.load_ord,
//         ports: entity.ports,
//         exclude: entity.exclude,
//         tls_ports: entity.tls_ports,
//         lookup_set: aset,
//         name: entity.name,
//         fallback: entity.fallback,
//         payload
//     }
// }
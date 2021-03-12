use super::parser::MatchLineExpr;

use regex::bytes::RegexSet;

use crate::error::Error;

// match http-proxy-ctrl m|^WWWOFFLE Server Status\n-*\nVersion *: (\d.*)\n| p/WWWOFFLE proxy control/ v/$1/ cpe:


/// This data structure is used for matching regex patterns expressed inside `nmap-service-probes`.
/// It allows us to store regex patterns inside a set, and their respective partner as 
/// 
/// this grouping is aligned so that indexes in `self.patterns` also correlate to `self.map`
/// where information about the response's data capture
#[derive(Debug)]
pub struct AlignedSet {
    patterns: RegexSet,
    map: Vec<MatchLineExpr>
}

impl AlignedSet {
    pub fn match_response<'a>(&'a self, input_buf: &[u8], out_buf: &mut Vec<&'a MatchLineExpr>) {
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

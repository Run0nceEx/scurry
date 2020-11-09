use super::{
    parser::{MatchLineExpr, Directive},
    common::CPE
};

use regex::bytes::RegexSet;


#[derive(Debug)]
pub struct Match {
    directive: Directive,
    name: String,
    cpe: CPE
}

impl Match {
    pub fn new(cpe: CPE, name: String, directive: Directive) -> Self {
        Self {
            cpe,
            name,
            directive
        }
    }
}

/// a representation of a group of match statements found in nmap-server-probes
/// this grouping is aligned so that indexes in `self.patterns` also correlate to `self.map`
/// where information about the response's data capture
#[derive(Debug)]
pub struct AlignedSet {
    patterns: RegexSet,
    map: Vec<Match>
}

impl AlignedSet {
    pub fn match_response<'a>(&'a self, input_buf: &[u8], out_buf: &mut Vec<&'a Match>) {
        let indexes: Vec<_> = self.patterns
            .matches(input_buf)
            .into_iter()
            .collect();
        
        for i in indexes {
            out_buf.push(self.map.get(i).unwrap());
        }
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
            mapping.push(Match::new(
                item.cpe.clone(),
                item.name.clone(),
                item.match_type,
            ));
        }
        
        Ok(AlignedSet {
            patterns: RegexSet::new(regex_buf)?,
            map: mapping
        })
    }
}

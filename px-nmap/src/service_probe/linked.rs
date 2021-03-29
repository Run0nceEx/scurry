
use regex::bytes::RegexSet;
use px_core::model::PortInput;
use super::{
    parser::model::{ProbeExpr, Protocol},
    parser::MatchLineExpr,
};
use crate::error::Error;

use std::{
    cmp::Ordering,
    collections::HashMap
};
use super::hex::construct_payload;
use regex::Regex;

#[derive(Debug)]
struct Link {
    proto: Protocol,// TCP/UDP
    payload: Vec<u8>,
    name: String,
    
    ports: Vec<PortInput>,
    exclude: Vec<PortInput>,
    tls_ports: Vec<PortInput>,

    //so here we have a flat map where we'll do a quick match on, 
    //where we get a collection of indexes matched, we'll take those 
    fallback: String,
    lookup_set: AlignedSet,
}

impl Link {
    fn matches<'a>(&'a self, input_buf: &[u8], out_buf: &mut Vec<&'a MatchLineExpr>) {
        self.lookup_set.match_response(input_buf, out_buf)
    }
}

#[derive(Debug)]
pub struct ChainedProbes {
    inner: Vec<Link>,
    name_map: HashMap<String, usize>
}

/// immutable collection of probe & trigger combinations
/// once created, its contents shouldn't be modified
// [x] rarity order + load order
// [x] all enteries with fallbacks do exist, or go to Null 
impl ChainedProbes {
    #[inline]
    fn inner_new(x: ProbeExpr) -> Result<Link, Error> {
        let mut this = Link {
            proto: x.proto,
            // interpret bytes TODO
            payload: {
                let mut buffer = Vec::new();
                construct_payload(&x.payload, &mut buffer)?;
                buffer
            },
            name: x.name,
            ports: x.ports,
            exclude: x.exclude,
            tls_ports: x.tls_ports,
            fallback: x.fallback.unwrap(),
            lookup_set: AlignedSet::new(x.matches).unwrap()
        };
        
        this.tls_ports.shrink_to_fit();
        this.ports.shrink_to_fit();
        this.exclude.shrink_to_fit();
        this.name.shrink_to_fit();
        this.payload.shrink_to_fit();
        this.fallback.shrink_to_fit();
    
        Ok(this)
    }
    
    #[inline]
    fn deduplicate_probes(mut last_state: (Option<String>, Vec<ProbeExpr>), probe: ProbeExpr) -> (Option<String>, Vec<ProbeExpr>) {
        if let Some(ref last_name) = last_state.0 {
            if probe.name == *last_name {
                //TODO(ADAM)
                //eprintln!("duplicate probe name found: {}", &probe.name);
            }
            last_state.0 = Some(probe.name.clone());
            last_state.1.push(probe);

        }
        else {
            // assume its the first iteration
            last_state.0 = Some(probe.name);
        }
        last_state
    }

    pub fn new(mut buf: Vec<ProbeExpr>) -> Result<Self, Error> {
        // sort by name
        buf.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());
        
        // look for duplicates, and de-duplicate
        let mut dedup = buf.drain(..)
            .fold(
                (None::<String>, Vec::new()), Self::deduplicate_probes
            ).1;
        drop(buf);

        // okay, now we need to do a fallback link check.
        // to ensure they actually go to something

        // copy all the names into this buffer
        let name_buf: Vec<String> = dedup
            .iter()
            .map(|probe| probe.name.clone())
            .collect();
        
        // use the buffer to see if anything shows up that doesn't exist
        let mut linked_probes: Vec<_> = dedup.drain(..).map(|mut probe| {
            if let Some(fallback) = &mut probe.fallback {

                // map to no fallback if it doesn't exist
                if !name_buf.contains(&fallback.to_string()) {
                    probe.fallback = None;
                }
            }
            probe
        }).collect();
        drop(name_buf);

        // now that we're de-duped and all unknown fallbacks are set to None,
        // we will set them to NULL (the probe), but before we can do that
        // we have to find NULL (again, the probe.)

        // sort buffer by rarity, and then by load order
        linked_probes.sort_by(|a, b| {
            // so this little condition should
            // put NULL as index 0
            if a.name == "NULL" {
                return Ordering::Greater
            }

            else if b.name == "NULL" {
                return Ordering::Less
            }

            // in all other cases, we'll sort on rarity, and then if equal, then load order
            let cmp = a.rarity.partial_cmp(&b.rarity).unwrap();
            match cmp {
                Ordering::Equal => return a.load_ord.partial_cmp(&b.load_ord).unwrap(),
                _ => return cmp
            }
        });
        
        // cut off after intensity is met
        // let mut probes: Vec<_> = linked.drain(..)
        //     .take_while(|probe| probe.rarity <= max_intensity)
        //     .collect();
        
        let mut name_map = HashMap::new();
        for (i, mut probe) in linked_probes.iter_mut().enumerate() {
            //reset None to NULL probe
            if let None = probe.fallback {
                probe.fallback = Some("NULL".to_string());
            }
            name_map.insert(probe.name.clone(), i);
                //.unwrap_none()
            
        }
        
        let mut links = Vec::with_capacity(linked_probes.len());
        for item in linked_probes {
            links.push(Self::inner_new(item)?)
        }

        // as far as we're concerned, 
        // this is now a flat list of probes that are ordered
        // first from rarity, then load order
        Ok(Self {
            inner: links,
            name_map
        })
    }

    pub fn null(&self) -> &Link {
        self.inner.get(0).unwrap()
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
    pub fn new(patterns: Vec<MatchLineExpr>) -> Result<AlignedSet, regex::Error> {
        // align two buffers so that RegexSet's index correlates with
        // 
        // -- self.patterns
        // -- self.cpe_lookup
            
        let mut regex_buf = Vec::new();
        let mut mapping = Vec::new();
        
        for item in patterns {
            regex_buf.push(item.pattern.schematic.clone());
            mapping.push(item);
        }

        regex_buf.shrink_to_fit(); 
        mapping.shrink_to_fit();
        
        Ok(AlignedSet {
            patterns: RegexSet::new(regex_buf)?,
            map: mapping
        })
    }

    fn iter<'s>(&'s self) -> AlignedSetIter<'s> {
        AlignedSetIter {
            idx: 0,
            set: self
        }
    }
}

struct AlignedSetIter<'set> {
    idx: usize,
    set: &'set AlignedSet
}

impl<'s> Iterator for AlignedSetIter<'s> {
    type Item=(&'s str, &'s MatchLineExpr);

    fn next(&mut self) -> Option<Self::Item> {
        match self.set.map.iter().nth(self.idx) {
            Some(match_expr) => {
                let pattern = self.set
                    .patterns
                    .patterns()[self.idx]
                    .as_str();
                
                self.idx += 1;
                Some((pattern, match_expr))
            }
            None => None
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use super::super::parser::parse;

    #[tokio::test]
    async fn align_set_behaves() {
        let mut buf = Vec::new();
        parse("/usr/share/nmap/nmap-service-probes", &mut buf).await.unwrap();
        let probes = ChainedProbes::new(buf).unwrap();
        
        //eprintln!("{:#?}", probes);
        //assert!(false)
    }
}

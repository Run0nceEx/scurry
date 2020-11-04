// parses "nmap-services"
use std::{
    collections::HashMap,
    io::{Read, BufReader, BufRead},
    str::FromStr
};
use crate::error::Error;
use super::super::common::Protocol;

#[derive(Clone, Debug)]
struct MapItem {
    protocol: Protocol,
    name: String,
    freq: f32
}

#[derive(Debug)]
struct PortMap(HashMap<u16, Vec<MapItem>>);

impl PortMap {
    pub fn parse_from_fd<T: Read>(fd: &mut T) -> Result<Self, Error> {
        let mut map: HashMap<u16, Vec<MapItem>> = HashMap::new();
        let mut reader = BufReader::new(fd);
        let mut buf = String::new();

        while let Ok(n) = reader.read_line(&mut buf) {
            if n == 0 { break }
            else if buf.starts_with("#") { continue }
            else if buf.contains("#") {
                buf = buf.split("#")
                    .nth(0)
                    .unwrap()
                    .to_string();
            }

            buf = buf.trim().to_string();

            let (seg, freq) = {
                let mut split = buf.split("\t");
                let main_seg = split.next().unwrap();
                let frequency = split.next().unwrap();
                (main_seg, frequency)
            };
            
            let (port, item) = {
                let mut split = seg.split(" ");
                let name = split.next().unwrap();
                
                let port_parse_str = split.next().unwrap();
                let mut port_split = port_parse_str.split("/");
                let port = port_split.next().unwrap().parse::<u16>()?;
                let protocol = Protocol::from_str(port_split.next().unwrap())?;

                (port, MapItem {
                    protocol,
                    name: name.to_string(),
                    freq: freq.parse()?
                })
            };
            if map.contains_key(&port) {
                let buf = map.get_mut(&port).unwrap();
                buf.push(item);
            }
            else {
                map.insert(port, vec![item]);
            }

            buf.clear();
        }

        Ok(Self(map))
    }
}
// parses "nmap-services"
use std::{
    collections::HashMap,
    str::FromStr
};
use crate::error::{Error, ErrorKind, FileLocation};
use tokio::{
    io::{self, BufReader, AsyncBufReadExt},
    fs::File
};

use std::path::Path;
use crate::service_probe::model::Protocol;

#[derive(Clone, Debug)]
struct MapItem {
    protocol: Protocol,
    name: String,
    freq: f32
}

#[derive(Debug)]
struct PortMap(HashMap<u16, Vec<MapItem>>);

impl PortMap {
    pub async fn parse_from_fd(path: &Path) -> Result<Self, ErrorKind> {
        let fd = File::open(path).await?;

        let mut reader = BufReader::new(fd);
        let mut map: HashMap<u16, Vec<MapItem>> = HashMap::new();
        let mut buf = String::new();
        
        let mut line_counter: u32 = 1;
        
        while reader.read_line(&mut buf).await? > 0 {
            if buf.starts_with("#") { continue }
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

            line_counter += 1;
            buf.clear();
        }

        Ok(Self(map))
    }
}
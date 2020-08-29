use serde::Serialize;
use crate::libcore::model::Host;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
enum Format {
    Stdout,
    Json
}

impl Format {
    pub fn fmt(&self, data: &Host) -> Result<String, super::error::Error> {
        match self {
            Format::Stdout => {
                let mut service_info = data.services
                    .iter()
                    .map(|s| format!("\t{}\t{}", s.port, s.state))
                    .collect::<Vec<_>>()[..]
                    .join("\n");
                
                Ok(format!("{}\n{}", data.addr, service_info))
            },
            Format::Json => Ok(serde_json::to_string(data)?)
        }
    }
}



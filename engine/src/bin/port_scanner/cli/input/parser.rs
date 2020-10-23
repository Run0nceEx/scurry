use std::{
    net::{IpAddr, SocketAddr},
    path::{PathBuf, Path}
};

use serde::Serialize;
use netcore::model::PortInput;
use cidr_utils::cidr::IpCidr;
use crate::cli::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressInput {
    Pair(SocketAddr),
    Singleton(IpAddr),
    CIDR(IpCidr),
    File(PathBuf),

    // TheWorld - special 
}


pub fn address_parser(src: &str) -> Result<AddressInput, Error> {    
    if Path::new(src).exists() {
        // File
        return Ok(AddressInput::File(PathBuf::from(src)))
    }
    
    else if src.contains("/") {
        //cidr
        return Ok(AddressInput::CIDR(IpCidr::from_str(src)?))
    }

    else {
        // IpAddr
        match src.parse() {
            // IpAddr
            Ok(x) => Ok(AddressInput::Singleton(x)),
            // SocketAddr
            Err(_) => Ok(AddressInput::Pair(src.parse()?))
        }
    }
}




impl std::str::FromStr for ScanMethod {
    type Err = Error;

    fn from_str(src: &str) -> Result<ScanMethod, Self::Err> {
        let opt = match src {
            "open" => ScanMethod::Open,
            "socks5" => ScanMethod::Socks5,
            _ => return Err(Error::CliError("unrecognized scan method".to_string()))
        };

        Ok(opt)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScanMethod {
    Open,
    Socks5,
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Serialize)]
pub enum Format {
    Stdout,
    Json,
    Stream
}

impl std::str::FromStr for Format {
    type Err = Error;
    
    fn from_str(x: &str) -> Result<Format, Self::Err> {
        let r = match x {
            "json" => Format::Json,
            "stream" => Format::Stream,
            "stdout" | "default" => Format::Stdout,
            
            _ => return Err(Error::CliError("Unknown format".to_string()))
        };
        Ok(r)
    }
}


// #[test]
// fn name() {

//     use cidr_utils::cidr::IpCidr;
//     let cidr = IpCidr::from_str("192.168.51.100/24").unwrap();
    
//     for x in cidr.iter_as_ip_addr() {
//         println!("{}", x);
//     }

//     assert_eq!(0, 1)
// }

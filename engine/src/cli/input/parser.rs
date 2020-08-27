use std::{
    net::{IpAddr},
    ops::Range,
    path::{PathBuf, Path}
};

use cidr_utils::cidr::IpCidr;
use crate::cli::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressInput {
    Singleton(IpAddr),
    CIDR(IpCidr),
    File(PathBuf)
}

pub fn address_parser(src: &str) -> Result<AddressInput, Error> {    
    if Path::new(src).exists() {
        return Ok(AddressInput::File(PathBuf::from(src)))
    }
    
    else if src.contains("/") {
        return Ok(AddressInput::CIDR(IpCidr::from_str(src)?))
    }
    
    else if (src.contains(".") && src.matches('.').count() == 4)
         || (src.contains(":") && src.matches(':').count() > 1)
    {
        return Ok(AddressInput::Singleton(src.parse()?))
    }

    Err(Error::CliError("unrecognized ip address format".to_string()))
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortInput {
    Singleton(u16),
    Range(Range<u16>)
}

impl std::convert::TryFrom<&str> for PortInput {
    type Error = Error;

    /// Accepts '80-443', '80', '0-10'
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        port_parser(value)
    }
}


pub fn port_parser(src: &str) -> Result<PortInput, Error> {
    let data: Vec<&str> = src.split("-").collect();
    let mut bottom = data.get(0).unwrap().parse::<u16>()?;

    if data.len() > 2 {
        return Err(Error::CliError("port ranges (-p 20-25) can only be in groups of 2 (not 20-25-30)".to_string()))
    }
    
    if data.len() == 2 {
        let mut top = data.get(1).unwrap().parse::<u16>()?;
        if bottom >= top {
            std::mem::swap(&mut bottom, &mut top);
        }
        return Ok(PortInput::Range(bottom..top))
    }

    Ok(PortInput::Singleton(bottom))
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScanMethod {
    Open,
    Socks5,
}

pub fn method_parser(src: &str) -> Result<ScanMethod, Error> {    
    match src {
        "open" => Ok(ScanMethod::Open),
        "socks5" => Ok(ScanMethod::Socks5),
        _ => Err(Error::CliError("unrecognized scan method".to_string()))
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

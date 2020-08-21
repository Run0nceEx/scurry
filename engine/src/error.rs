use super::libcore::error::Error as LibError;

#[derive(Debug)]
pub enum Error {
    CoreError(LibError),
    
    IO(std::io::Error),   
    CliError(String),
    IntParseError(std::num::ParseIntError),
    AddrParseError(std::net::AddrParseError),
    AddrParseErrorCIDR(cidr_utils::cidr::IpCidrError)
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Self::IO(x)
    }
}

impl From<cidr_utils::cidr::IpCidrError> for Error {
    fn from(x: cidr_utils::cidr::IpCidrError) -> Self {
        Self::AddrParseErrorCIDR(x)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(x: std::net::AddrParseError) -> Self {
        Self::AddrParseError(x)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(x: std::num::ParseIntError) -> Self {
        Self::IntParseError(x)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

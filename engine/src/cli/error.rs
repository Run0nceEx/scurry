use crate::libcore::error::Error as LibError;

#[derive(Debug)]
enum OutputError {
    Json(serde_json::Error),
}

#[derive(Debug)]
enum InputError {
    IntParseError(std::num::ParseIntError),
    AddrParseError(std::net::AddrParseError),
    AddrParseErrorCIDR(cidr_utils::cidr::IpCidrError)
}

#[derive(Debug)]
pub enum Error {
    CoreError(LibError),
    
    CliError(String),
    CliInputError(InputError),
    CliOutputError(OutputError),
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Self::CoreError(LibError::IO(x))
    }
}

impl From<cidr_utils::cidr::IpCidrError> for Error {
    fn from(x: cidr_utils::cidr::IpCidrError) -> Self {
        Self::CliInputError(InputError::AddrParseErrorCIDR(x))
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(x: std::net::AddrParseError) -> Self {
        Self::CliInputError(InputError::AddrParseError(x))
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(x: std::num::ParseIntError) -> Self {
        Self::CliInputError(InputError::IntParseError(x))
    }
}

impl From<serde_json::Error> for Error {
    fn from(x: serde_json::Error) -> Self {
        Self::CliOutputError(OutputError::Json(x))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

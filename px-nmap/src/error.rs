use std::path::PathBuf;

use super::service_probe::parser::model::Token;

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    ExpectedToken(Token),
    IO(tokio::io::Error),
    PxCore(px_core::error::Error),
    Bincode(Box<bincode::ErrorKind>)
}
#[derive(Debug, Clone)]
pub struct FileLocation {
    pub path: PathBuf,
    pub column: usize,
    pub row: usize
}

impl From<tokio::io::Error> for Error {
    fn from(e: tokio::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<px_core::error::Error> for Error {
    fn from(e: px_core::error::Error) -> Self {
        Self::PxCore(e)
    }
}

impl From<bincode::ErrorKind> for Error {
    fn from(e: bincode::ErrorKind) -> Self {
        Self::Bincode(Box::new(e))
    }
}

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        Self::Bincode(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(x: std::num::ParseIntError) -> Self {
        Self::ParseError(x.to_string())
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(x: std::num::ParseFloatError) -> Self {
        Self::ParseError(x.to_string())
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self)
    }
}

impl std::error::Error for Error {}
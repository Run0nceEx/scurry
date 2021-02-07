use tokio::time::error::Error as TimeError;

// #[derive(Debug)]
// pub struct TimeError;

#[derive(Debug)]
pub enum Error {
    ParseError(String),
    TimeCacheError(TimeError),
    IO(std::io::Error),
    RangeError,
//    ParseErr(ParseErr)
}

// use super::netlib::parsers::nmap::Error as ParseErr;
// impl From<ParseErr> for Error {
//     fn from(x: ParseErr) -> Self {
//         if let ParseErr::IO(err) = x {
//             return Error::IO(err)
//         }
//         Error::ParseErr(x)        
//     }
// }


impl From<std::num::ParseIntError> for Error {
    fn from(x: std::num::ParseIntError) -> Self {
        //kind of hacky but works
        Self::ParseError(x.to_string())
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(x: std::num::ParseFloatError) -> Self {
        //kind of hacky but works
        Self::ParseError(x.to_string())
    }
}


impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Self::IO(x)
    }
}

impl From<TimeError> for Error {
    fn from(x: TimeError) -> Self {
        Self::TimeCacheError(x)
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl std::error::Error for Error {}
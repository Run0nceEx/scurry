use tokio::time::Error as TimeError;

#[derive(Debug)]
pub enum Error {
    TimeCacheError(TimeError),
    IO(std::io::Error),
    RangeError,
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
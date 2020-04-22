use tokio::prelude::*;
use tokio::io;

pub enum Error {
    io(io::Error),
    http(httparse::Error)
}

impl From<io::Error> for Error {
    fn from(x: io::Error) -> Error {
        Error::io(x)
    }
}

impl From<httparse::Error> for Error {
    fn from(x: httparse::Error) -> Error {
        Error::http(x)
    }
}

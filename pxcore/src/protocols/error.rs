use tokio::prelude::*;

pub enum Error {
    io(io::Error),
    socks(tokio_socks::Error),
    http(http_types::Error)
}

impl From<io::Error> for Error {
    fn from(x: io::Error) -> Error {
        Error::io(x)
    }
}

impl From<tokio_socks::Error> for Error {
    fn from(x: tokio_socks::Error) -> Error {
        Error::socks(x)
    }
}

impl From<http_types::Error> for Error {
    fn from(x: http_types::Error) -> Error {
        Error::http(x)
    }
}

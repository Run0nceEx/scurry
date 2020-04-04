
pub mod chains;

use tokio::prelude::*;
use tokio::net::TcpStream;
use std::future::Future;
use super::error::Error;


pub trait ProxyChain {
    //takes self and read/write to forward next proxy
    fn chain(&mut self, payload: &[u8]) -> Future<Output=Result<usize, Error>>;
}

// Protocol determination
pub trait FromTcpStream<T>: Sized + AsyncRead + AsyncWrite {
    fn from_tcpstream(stream: TcpStream, state: T)  -> Self;
}

// back into raw/inner
pub trait IntoTcpStream: Sized {
    fn into_tcpstream(self) -> TcpStream;
}

use tokio_socks::tcp::Socks5Stream;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::prelude::*;
use crate::proxy::core::*;
use crate::proxy::error::Error;
use http_types::{Method, Request as HttpRequest, Url};
use async_h1::client::{Encoder};

#[repr(u8)]
#[derive(Clone, Copy)]
enum SockCommand {
    Connect = 0x01,
    Bind = 0x02,
    #[allow(dead_code)]
    Associate = 0x03,
    #[cfg(feature = "tor")]
    TorResolve = 0xF0,
    #[cfg(feature = "tor")]
    TorResolvePtr = 0xF1,
}

//////
// SocksState

pub struct SocksAuth {
    user:  Vec<u8>,
    password: Vec<u8>,
    command: SockCommand,
    target: SocketAddr
}

pub struct Socks {
    command: SockCommand,
    target: SocketAddr
}

impl Default for Socks {
    fn default() -> Self {
        Socks {
            command: SockCommand::Connect,
            target: "0.0.0.0".parse().expect("sdfd")
        }
    }
}

impl IntoTcpStream for Socks5Stream {
    fn into_tcpstream(self) -> TcpStream {
        self.into_inner()
    }
}

impl FromTcpStream<SocksAuth> for Socks5Stream 
{
    fn from_tcpstream(x: TcpStream, state: SocksAuth) -> Socks5Stream {
        unimplemented!()
    }
}


// impl ProxyProtocol<SocksAuth> for Socks5Stream {

//     fn is_protocol(&mut self) -> Result<(), Error> {
//         let buf = &mut [0;513];
//         self.read(buf).await?;
//         unimplemented!()
//     }

    
//     //takes self and read/write to forward next proxy
//     fn negotiate(&mut self, payload: &[u8]) -> Result<usize, Error> {
//         unimplemented!()
//     }
// }

//////
// no auth Socks5Stream

impl FromTcpStream<Socks> for Socks5Stream {
    fn from_tcpstream(x: TcpStream, state: Socks) -> Socks5Stream {
        unimplemented!()
    }
}

// #[async_trait]
// impl ProxyProtocol<Socks> for Socks5Stream {
    
//     async fn is_protocol(&mut self) -> Result<(), Error> {
//         let buf = &mut [0;513];
//         self.read(buf).await?;
//         unimplemented!()
//     }
    
//     //takes self and read/write to forward next proxy
//     async fn negotiate(&mut self, payload: &[u8]) -> Result<usize, Error> {
//         unimplemented!()
//     }
// }


//////
// http

impl FromTcpStream<HttpRequest> for TcpStream {
    fn from_tcpstream(mut x: TcpStream, state: HttpRequest) -> TcpStream {
        x
    }
}

// #[async_trait]
// impl ProxyProtocol<HttpRequest> for TcpStream {
    
//     async fn is_protocol(&mut self) -> Result<(), Error> {
//         let buf = &mut [0;513];
//         self.read(buf).await?;
//         unimplemented!()
//     }
    
//     //takes self and read/write to forward next proxy
//     async fn negotiate(&mut self, payload: &[u8]) -> Result<usize, Error> {
//         unimplemented!()
//     }
// }


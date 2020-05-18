use std::net::SocketAddr;
use std::error::Error;
use tokio::net::TcpStream;

pub struct ErrorKind(String);


#[async_trait::async_trait]
pub trait Negotiate: Sized + 'static {

    async fn negotiate<T, C>(&self, addr: SocketAddr, protocol: T) -> Result<bool, ErrorKind>
    where 
        T: Identifier<Self> + Send + Sync,
        C: Connector<Self> + Send + Sync,
        Self: Send
    {
        protocol.detect(C::init_connect(addr).await?).await
    }

    async fn noop<C>(&self, addr: SocketAddr) -> Result<(), ErrorKind>
    where
        C: Connector<Self> + Send
    {
        C::init_connect(addr).await?;
        Ok(())
    }
}

//Blanket expression for any type that has Connector<T> + Identifier<T>
impl<T> Negotiate for T 
where 
    T: Connector<T> + Identifier<T> + Send + Sync + 'static {}


#[async_trait::async_trait]
/// Object used to represent the communication object spawned from `init_connect`
/// and is used in `Indentifier`
pub trait Connector<C: Negotiate> {
    /// Constructor
    async fn init_connect(addr: SocketAddr) -> Result<C, ErrorKind>;
}


#[async_trait::async_trait]
/// Identifies protocols
pub trait Identifier<C: Negotiate + 'static>
{
    /// exec detection
    async fn detect(&self, con: C) -> Result<bool, ErrorKind>;
}


impl Negotiate for TcpStream {}

#[async_trait::async_trait]
impl Connector<TcpStream> for TcpStream {
    async fn init_connect(addr: SocketAddr) -> Result<TcpStream, ErrorKind> {
        Ok(TcpStream::connect(addr).await?)
    }
}

struct Socks5;

#[async_trait::async_trait]
impl Identifier<TcpStream> for Socks5 {
  async fn detect(&self, con: TcpStream) -> Result<bool, ErrorKind> {
      Ok(true)
  }
}


struct A(TcpStream);
impl A {
    async fn new(addr: SocketAddr) {
        let stream = TcpStream::connect(addr).await.unwrap();
        Socks5{}.detect(stream);
    }
}

impl<T> From<T> for ErrorKind where T: std::error::Error {
    fn from(x: T) -> ErrorKind {
        ErrorKind(x.to_string())
    }
}

// #[derive(Debug)]
// struct SocksScanner {
//     addr: SocketAddr
// }

// impl std::fmt::Display for SocksScanner {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Socks5")
//     }
// }

// #[async_trait::async_trait]
// impl Scannable for SocksScanner {
//     async fn scan(&self) -> bool {true}
// }

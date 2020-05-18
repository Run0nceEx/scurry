use std::net::SocketAddr;
use std::error::Error;
use tokio::net::TcpStream;

pub struct ErrorKind(String);


#[async_trait::async_trait]
pub trait Connection: Sized + 'static {

    async fn negotiate<T, C>(&self, addr: SocketAddr, protocol: T) -> Result<bool, ErrorKind>
    where 
        T: Identifier<Self> + Send + Sync,
        C: Connector<Self> + Send + Sync,
        Self: Send
    {
        protocol.detect(C::init_connect(addr).await?).await
    }
}

impl<T> Connection for T 
where 
    T: Connector<Self> + Identifier<Self> + Send + Sync + 'static {}


#[async_trait::async_trait]
/// Object used to represent the communication object spawned from `init_connect`
/// and is used in `Indentifier`
pub trait Connector<C: Connection> {
    /// Constructor
    async fn init_connect(addr: SocketAddr) -> Result<C, ErrorKind>;
}


#[async_trait::async_trait]
/// Identifies protocols
pub trait Identifier<C: Connection + 'static>
{
    /// exec detection
    async fn detect(&self, con: C) -> Result<bool, ErrorKind>;

    async fn noop_connect(&self, addr: SocketAddr) -> Result<(), ErrorKind> {
        TcpStream::connect(addr).await?;
        Ok(())
    }
}


#[async_trait::async_trait]
impl Connector<TcpStream> for TcpStream {
    async fn init_connect(addr: SocketAddr) -> Result<TcpStream, ErrorKind> {
        Ok(TcpStream::connect(addr).await?)
    }
}


impl Identifier<TcpStream> for 


impl Connection for TcpStream {}

struct A(TcpStream);
impl A {
    async fn new(addr: SocketAddr) {
        TcpStream::connect(addr).await;
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

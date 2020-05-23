use std::{
    net::SocketAddr,
    error::Error,
    collections::HashMap
};

use tokio::time::{Duration, timeout as timeout_future};


#[async_trait::async_trait]
pub trait Negotiate: Sized + 'static {

    async fn negotiate<T>(&self, addr: SocketAddr, timeout: Duration, protocol: T) -> Result<bool, Box<dyn Error>>
    where 
        T: Identifier<Self> + Send + Sync,
        Self: Connector + Send
    {   
        let mut stream = Self::init_connect(addr).await?;
        Ok(timeout_future(timeout, protocol.detect(&mut stream)).await??)
    }

    async fn noop(&self, addr: SocketAddr) -> Result<(), Box<dyn Error>>
    where
        Self: Connector + Send
    {
        Self::init_connect(addr).await?;
        Ok(())
    }
}


#[async_trait::async_trait]
/// This can be seen as a constructor/init for `C`
/// and is passed into `Indentifier`
pub trait Connector: Negotiate {
    /// Constructor
    async fn init_connect(addr: SocketAddr) -> Result<Self, Box<dyn Error>>;
}

#[async_trait::async_trait]
/// Identifies protocols
/// Takes whatever Connector<C> constructs
pub trait Identifier<C: Negotiate + 'static>
{
    /// exec detection
    async fn detect(&self, con: &mut C) -> Result<bool, Box<dyn Error>>;
}



// Blanket expression for any type that has Connector<T> + Identifier<T>
impl<T> Negotiate for T 
where 
    T: Connector + Identifier<T> + Send + Sync + 'static {}



pub use tokio::net::TcpStream;

impl Negotiate for TcpStream {}

#[async_trait::async_trait]
impl Connector for TcpStream {
    async fn init_connect(addr: SocketAddr) -> Result<TcpStream, Box<dyn Error>> {
        Ok(TcpStream::connect(addr).await?)
    }
}

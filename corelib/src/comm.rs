use std::{
    net::SocketAddr,
    error::Error,
    collections::HashMap
};

use tokio::time::{Duration, timeout as timeout_future};




// Blanket expression for any type that has Connector<T> + Identifier<T>
impl<T> Negotiate for T 
where 
    T: Connector + ProtocolIdentifier<T> + Send + Sync + 'static {}



pub use tokio::net::TcpStream;

impl Negotiate for TcpStream {}

#[async_trait::async_trait]
impl Connector for TcpStream {
    async fn init_connect(addr: SocketAddr) -> Result<TcpStream, Box<dyn Error>> {
        Ok(TcpStream::connect(addr).await?)
    }
}

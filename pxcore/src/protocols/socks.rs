use tokio::net::TcpStream;
use tokio::prelude::*;
use std::net::SocketAddr;

use super::{Scannable, Error};

struct Sock5 {}

#[async_trait::async_trait]
impl Scannable<TcpStream> for Sock5 {
    
    async fn connect(&self, addr: SocketAddr) -> Result<TcpStream, Error> {
        Ok(TcpStream::connect(addr).await?)
    }

    async fn scan(&self, addr: SocketAddr) -> Result<bool, Error> {
        let mut stream = self.connect(addr).await?;
        stream.write(&[0x05, 0x01, 0x00]).await?;
        
        let mut buf: [u8; 2] = [0; 2];
        stream.read(&mut buf).await?;
        
        Ok(buf[0] == 5 && buf[1] == 0)
    }
}




use tokio::net::TcpStream;
use tokio::prelude::*;
use crate::ProtocolIdentifier;

struct Socks5;

#[async_trait::async_trait]
impl ProtocolIdentifier<TcpStream> for Socks5 {
    async fn detect(&self, stream: &mut TcpStream) -> Result<bool, Box<std::error::Error>> {
        stream.write(&[0x05, 0x01, 0x00u8]).await?;
        
        let mut buf: [u8; 2] = [0; 2];
        stream.read(&mut buf).await?;
        
        Ok(buf[0] == 5 && buf[1] == 0)
    }
}




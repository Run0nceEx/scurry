use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;

use super::Error;

#[derive(Clone, Debug)]
struct Chain<L: Copy> {
    cnt_ptr: usize,
    pub links: Vec<L>,
    
    
    buf: Vec<u8>
}

impl<L: ChainLink + Copy> Chain<L> {
    
    pub fn flush(&mut self) {
        self.buf.clear();
    }

    pub async fn send<'a, 'b>(&'a mut self, stream: &'b mut TcpStream, payload: &[u8]) {
        for l in self.links.clone() {
            l.wrap(stream, &mut self.buf).await;
        }
        stream.write(payload).await;
    }

    pub async fn recv(&mut self, stream: &mut TcpStream, buf: &mut [u8]) -> std::io::Result<usize> {
        self.links.reverse();

        for l in self.links.clone() {
            l.unwrap(stream, &mut self.buf).await;
        }
        
        let size = stream.read(buf).await;
        self.links.reverse();

        size
    }

}



#[async_trait::async_trait]
pub trait ChainLink {
    async fn wrap(&self, stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), Error>;
    async fn unwrap(&self, stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), Error>;
}
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;

// todo

#[async_trait::async_trait]
pub trait ChainLink: Clone {
    /// Init - negotiate all the steps needed to pass on the payload
    async fn init(&self, stream: &mut TcpStream) -> Result<(), tokio::io::Error>;


    /// Prepare payload's padding
    /// finish any further negotiations
    async fn step(&self, stream: &mut TcpStream, buf: &mut [u8]) -> Result<(), tokio::io::Error>;

    // clean up any extra data from padding when recv'ed
    async fn clean(&self, buf: &mut [u8]) -> std::io::Result<()>;
}


#[derive(Clone, Debug)]
struct Chain<T: ChainLink> {
    pub links: Vec<T>,
    buf: Vec<u8>
}

impl<T: ChainLink> Chain<T> {
    pub fn flush(&mut self) {
        self.buf.clear();
    }

    pub async fn send(&mut self, stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
        for l in &self.links {
            l.init(stream).await?;
            l.step(stream, &mut self.buf).await?;
        }
        stream.write(payload).await?;
        Ok(())
    }

    pub async fn recv(&mut self, stream: &mut TcpStream, buf: &mut [u8]) -> std::io::Result<usize> {
        self.links.reverse();

        for l in &self.links {
            l.clean(&mut self.buf).await?;
        }
        
        let size = stream.read(buf).await;
        self.links.reverse();

        size
    }

}


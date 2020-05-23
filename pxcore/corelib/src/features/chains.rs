// todo
use super::comm::{Connector, Identifier, Negotiate as Scan};


#[async_trait::async_trait]
pub trait LinkConnect<C: Connector<C>> {
    fn introduce(stream: C) -> Result<(), Box<dyn Error>>;
}


#[async_trait::async_trait]
trait ChainProtocol<C>: LinkConnect<C> {
    fn relay(&mut self) -> Result<(), Box<dyn Error>>;
}

#[async_trait::async_trait]
/// Chain protocol support for a specific protocol `P`
pub trait ChainLink {

}




pub trait Link {
    fn step<C>(&mut self, constructor: C) where C: LinkConnect<C> + LinkTunnel<C> 
}



// pub trait ConnectorLink<S: Connector<S>, P: Identifier<S>> {
//     fn unravel<X: Identifier<X>, N: ConnectorLink<S, X>>(stream: S, protocol: P) -> Option<N> {}
// }



#[async_trait::async_trait]
pub trait ChainLink<C>: Clone {
    /// Init - negotiate all the steps needed to pass on the payload
    async fn init(&self, stream: &mut C) -> Result<(), tokio::io::Error> {

    }


    async fn step(&self, stream: &mut C, buf: &mut [u8]) -> Result<(), tokio::io::Error>;

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


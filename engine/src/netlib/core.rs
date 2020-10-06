// noticing we're only using connections we're going to attempt to make a more generic worker
use std::{
    net::SocketAddr,
    time::Duration
};

use crate::model::State;


#[async_trait::async_trait]
trait PeerPreview<I> where I: PeerInterface {
    async fn connect(self) -> Result<I, crate::error::Error>;
}


#[async_trait::async_trait]
// this is essentially tcp/udp 
// 
trait PeerInterface {
    fn peer(&self) -> SocketAddr;
    fn state(&self) -> State;
    fn set_timeout(&mut self, ttl: Duration);
    
    async fn read_iface(&mut self, buf: &mut Vec<u8>, amount: usize) -> Result<usize, std::io::Error>;
    async fn write_iface(&mut self, buf: &[u8]) -> Result<usize, std::io::Error>;
    
}


trait UpgradeInterface<T>: PeerInterface
where
    Self: PeerInterface,
    T: PeerInterface
{
    fn wants_upgrade(&mut self) -> bool;
    fn upgrade(self) -> T;
}

#[async_trait::async_trait]
trait PeerOperations: PeerInterface {
    type Operation;
    
    async fn apply_operation(&mut self, op: Self::Operation);
}


#[async_trait::async_trait]
trait Protocol
{

    async fn probe<I>(self) -> bool
    where
        Self: PeerPreview<I> + Send + Sync + Sized,
        I: PeerInterface + Send + Sync
    
    {
        if let Ok(x) = self.connect().await {
            return true
        }

        false
    }


    async fn apply_operation<T>(self, op: T) -> Result<(), crate::error::Error>
    where
        Self: PeerInterface + PeerOperations<Operation=T> + Send + Sync + Sized,
        T: Send + Sync
    
    {
        self.apply_operation(op).await
    }

}





async fn negotiate_upgrade<I, U, E>(interface: I) -> Result<Option<U>, E>
where
    I: PeerInterface,
    U: PeerInterface,

    {

        unimplemented!()
    }

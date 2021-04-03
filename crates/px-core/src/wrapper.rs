// noticing we're only using connections we're going to attempt to make a more generic worker
use std::{
    net::SocketAddr,
    time::Duration
};

use crate::model::State;


#[async_trait::async_trait]
trait NetworkInterface {
    async fn read_iface(&mut self, buf: &mut Vec<u8>, amount: usize) -> Result<usize, std::io::Error>;
    async fn write_iface(&mut self, buf: &[u8]) -> Result<usize, std::io::Error>;
    
    fn peer(&self) -> SocketAddr;
    fn state(&self) -> State;

    fn set_timeout(&mut self, ttl: Duration);
}

trait Probe<T> {
    // T = probe data - Probe<Minecraft>, Probe<Ssh>
    async fn probe_service<I: NetworkInterface>(&mut self, iface: &mut I) -> Option<T>;
}


#[async_trait::async_trait]
trait Interact {
    type Interface: NetworkInterface;
    type Operation;
    
    async fn apply_operation<I>(&mut self, iface: &mut Self::Interface, op: Self::Operation) -> Result<T, Error>;
    
    fn wants_upgrade(&mut self, iface: Self::Interface) -> bool;   

    async fn upgrade<T>(&mut self, iface: &mut Self::Interface) -> Result<T, Error>
    where T: UpgradeConnection<Self::Interface, Self::Operation, T>
    {
        T::negotiate_upgrade(iface)
    }
}

trait UpgradeConnection<I, O, T> {
    async fn negotiate_upgrade(iface: &mut I) -> Result<T, Error>
    where T: Interact<Interface=I, Operation=O>;
}

trait Protocol 
{
    type Operation;
    type Interface: NetworkInterface;

    async fn probe<T>(&mut self, iface: &mut I) -> Option<T>
    where Self: Probe<T> {
        self.probe_service(iface).await
    }

    async fn interact<T>(&mut self, iface: &mut I, ) -> Result<T, Error>
    where Self: Interact {
        if self.wants_upgrade(&mut iface) {
            self.upgrade(&mut iface);
        }
        
        self.apply_operation(iface).await
    }
}


trait ConnectionWorker {
    type State;
    type Response;

    fn exec(state: &mut Self::State) -> Result<JobCtrl<Self::Response>, crate::error::Error>;
}




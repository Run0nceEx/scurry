use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::prelude::*;
use async_std::task;
use async_std::io;
use async_std::sync::Sender;

pub async fn negotiate<T, I, F, Fut>(rsrc: T, func: F, tx: Sender)
where
    T: Iterator<Item=I>,
    I: ToSocketAddrs + Send + Sync + 'static,
    <I as ToSocketAddrs>::Iter: Send + Sync,
    F: Fn(io::Result<TcpStream>) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output=io::Result<()>> + Send + Sync
{
    for addr in rsrc {
        task::spawn(async move {
            tx.send(
                func(
                    TcpStream::connect(addr).await
                ).await
            );
        });
    }
}

pub async fn scan_socks(s: io::Result<TcpStream>) {



}
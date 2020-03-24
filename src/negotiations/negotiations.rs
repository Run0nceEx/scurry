use async_std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use async_std::prelude::*;
use async_std::task;
use async_std::io;
use async_std::sync::{Sender};


pub enum Connection {
    Completed(io::Result<()>),
    Denied(io::Error),
    ParsingError(Option<io::Error>)
}

pub enum InputData<I> {
    Addr(SocketAddr),
    Raw(I)
}

pub async fn spawner<T, I, F, Fut>(rsrc: T, func: F, tx: Sender<(InputData<I>, Connection)>)
where
    T: Iterator<Item=I> + Send + Sync,
    I: ToSocketAddrs + Send + Sync + Clone + 'static,
    <I as ToSocketAddrs>::Iter: Send + Sync,
    F: Fn(TcpStream) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output=io::Result<()>> + Send + Sync
{
    for addr in rsrc {
        let ntx = tx.clone();
        task::spawn(async move {
            match addr.to_socket_addrs().await {
                Ok(mut s_addr) => {
                    
                    match s_addr.next() {
                        Some(socketaddr) => {
                            let pkg = match TcpStream::connect(socketaddr.clone()).await {
                                Ok(con) => {
                                    (InputData::Addr(socketaddr), Connection::Completed(func(con).await))
                                }
                                Err(e) => {
                                    (InputData::Raw(addr), Connection::Denied(e))
                                }
                            };
                            ntx.send(pkg).await;
                        }
                        
                        None => {
                            ntx.send((InputData::Raw(addr), Connection::ParsingError(None))).await;
                        } 
                    } 
                //
                }

                Err(e) => {
                    ntx.send((InputData::Raw(addr), Connection::ParsingError(Some(e)))).await;
                }
            }
        });
    }
}
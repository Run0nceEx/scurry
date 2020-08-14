use crate::{
    task::{SignalControl, CRON},
    error::Error,
};

use tokio::net::TcpStream;
use std::{net::SocketAddr, time::{Duration, Instant}};

#[derive(Debug, Clone)]
pub struct Job {
    pub addr: SocketAddr
}

impl Job {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: addr,
        }
    }
}

impl From<SocketAddr> for Job {
    fn from(s: SocketAddr) -> Self {
        Self::new(s)
    }
}


#[derive(Debug, Clone)]
pub enum PortState {
    Open(SocketAddr),
    Closed(SocketAddr)
}

#[derive(Debug)]
pub struct OpenPortJob;

#[async_trait::async_trait]
impl CRON for OpenPortJob
{
    type State = Job;
    type Response = PortState;

    async fn exec(state: &mut Job) -> Result<(SignalControl, Option<Self::Response>), Error>
    {
        match scan(state.addr).await {
            Ok(_) => Ok(
                (SignalControl::Success(true), Some(PortState::Open(state.addr)))
            ),

            Err(Error::IO(x)) => {
                match x.kind() {
                    std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionRefused 
                    | std::io::ErrorKind::TimedOut => return Ok((SignalControl::Success(false), Some(PortState::Closed(state.addr)))),

                    std::io::ErrorKind::Other => {
                        if let Some(error_code) = x.raw_os_error() {
                            match error_code {
                                101         // Network unreachable
                                | 113       // no route to host
                                | 92        // failed to bind to interface/protocol
                                | 24 =>     // too many file-discriptors open
                                    return Ok((SignalControl::Stash(Duration::from_secs(5)), None)),
                                
                                _ => {
                                        tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "Error Code: {}", error_code);
                                        return Ok((SignalControl::Success(false), Some(PortState::Closed(state.addr))))
                                } 
                            };
                        }
                        else {
                            tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "Error Code: [OTHER]");
                            return Ok((SignalControl::Retry, None))
                        }
                    }
                    
                    _ => {
                        tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "unmatched {:#?}", x);
                        return Ok((SignalControl::Retry, None))
                    }
                }                        
            },

            Err(e) => {
                tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "unmatched {:#?}", e);
                return Ok((SignalControl::Retry, None))
            }
        }
    }
}

async fn scan(addr: SocketAddr) -> Result<(), crate::error::Error> {
    TcpStream::connect(addr).await?;
    Ok(())
}


#[cfg(test)]
mod test {
    extern crate test;
    
    use super::*;
    use tokio::runtime::Runtime;
    use tokio::net::TcpListener;

    #[bench]
    fn tokio_connect(b: &mut test::Bencher) {
        let mut rt = Runtime::new().unwrap();
        let addr: SocketAddr = "127.0.0.1:20927".parse().unwrap();
        
        rt.spawn(async move {
            let mut listener = TcpListener::bind(addr).await.unwrap();
            loop {    
                let (con, _addr) = listener.accept().await.unwrap();
                drop(con);
            }
        });
        
        b.iter(|| rt.block_on(TcpStream::connect(addr)));
    }
}
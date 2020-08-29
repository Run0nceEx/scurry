use crate::libcore::{
    task::{SignalControl, CRON},
    error::Error,
    model::State,
};

use tokio::net::TcpStream;
use std::{
    net::SocketAddr, 
    //time::{Duration, Instant}
};

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

    async fn exec(state: &mut Job) -> Result<SignalControl<Self::Response>, Error>
    {
        match scan(state.addr).await {
            Ok(_) => Ok(SignalControl::Success(State::Open, PortState::Open(state.addr))),
            Err(Error::IO(x)) => Ok(super::handle_io_error(x, PortState::Closed(state.addr))),
            
            Err(e) => {
                tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "unmatched {:#?} [not io error]", e);
                return Ok(SignalControl::Retry)
            }
        }
    }
}

async fn scan(addr: SocketAddr) -> Result<(), Error> {
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
    /// Test how fast tokio's connect is.
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
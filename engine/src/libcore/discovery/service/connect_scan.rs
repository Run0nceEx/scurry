use crate::libcore::{
    pool::{SignalControl, CRON},
    error::Error,
    model::State,
};

use tokio::net::TcpStream;
use std::{net::SocketAddr};
use super::util::handle_io_error;

#[derive(Debug)]
pub struct OpenPortJob;

#[async_trait::async_trait]
impl CRON for OpenPortJob
{
    type State = SocketAddr;
    type Response = SocketAddr;

    async fn exec(state: &mut SocketAddr) -> Result<SignalControl<Self::Response>, Error>
    {
        match scan(*state).await {
            Ok(_) => Ok(SignalControl::Success(State::Open, *state)),
            Err(Error::IO(x)) => Ok(handle_io_error(x, *state)),
            
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
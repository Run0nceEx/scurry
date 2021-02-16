use px_core::{
    pool::{JobCtrl, CRON, JobErr},
    error::Error,
    model::State,
};

use tokio::net::TcpStream;
use std::net::SocketAddr;
use super::handle_io_error;


#[derive(Debug)]
pub struct TcpProbe;

#[async_trait::async_trait]
impl CRON for TcpProbe
{
    type State = SocketAddr;
    type Response = SocketAddr;

    async fn exec(state: &mut SocketAddr) -> Result<JobCtrl<Self::Response>, Error>
    {
        match scan(*state).await {
            Ok(_) => Ok(JobCtrl::Return(State::Open, *state)),

            Err(Error::IO(err)) => Ok(JobCtrl::Error(handle_io_error(err))),
            Err(e) => {
                eprintln!("unmatched error {:#?} [not io error]", e);
                Ok(JobCtrl::Error(JobErr::Other))
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
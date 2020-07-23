// This is a scanner thats concepts
// and resources are adopted from the nmap project
// theres probably 100 things im doing wrong, and that im missing (syn packets for ex)
// i do accept contributions, and they'll probably be on this file.
use crate::{
    schedule::{
        {SignalControl, CRON},
        sugar::Subscriber,
        meta::CronMeta,
    },
    error::Error,
};

use tokio::{
    net::TcpStream    
};

use std::net::SocketAddr;


#[derive(Debug, Clone)]
pub struct Job {
    addr: SocketAddr,
    ctr: u8,
    ctr_max: u8
}

impl Job {
    pub fn new(addr: SocketAddr, max: u8) -> Self {
        Self {
            addr: addr,
            ctr: 0,
            ctr_max: max
        }
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
            Ok(_)   => Ok(
                (SignalControl::Success, Some(PortState::Open(state.addr)))
            ),
            Err(_e) => Ok(
                (SignalControl::Success, Some(PortState::Closed(state.addr)))
            )
        }
    }

    fn name() -> String {
        let x = format!("{:?}", OpenPortJob);
        x
    }
}

async fn scan(addr: SocketAddr) -> Result<(), crate::error::Error> {
    //TODO Add timeout
    let mut connection = TcpStream::connect(addr).await?;
    
    Ok(())
}

pub struct PrintSub {
    ctr: u64,
}
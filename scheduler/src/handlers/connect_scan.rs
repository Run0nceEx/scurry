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

pub struct OpenPortJob;

#[async_trait::async_trait]
impl CRON for OpenPortJob
{
    type State = Job;
    type Response = PortState;

    async fn exec(state: Job) -> Result<SignalControl<(Option<Self::Response>, Self::State)>, Error>
    {
        match scan(state.addr).await {
            Ok(_)   => Ok(SignalControl::Success((Some(PortState::Open(state.addr)), state))),
            Err(_e) => Ok(SignalControl::Success((Some(PortState::Closed(state.addr)), state)))
        }
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

impl PrintSub {
    pub fn new() -> Self {
        Self {
            ctr: 0
        }
    }
}

#[async_trait::async_trait]
impl Subscriber<(Option<PortState>, Job)> for PrintSub {
    async fn handle(&mut self, meta: &CronMeta, data: &(Option<PortState>, Job)) -> Result<(), Error> {
        self.ctr += 1;

        let notify = [200, 1000, 10000, 50000, 100000, 150000, 200000];
        if notify.contains(&self.ctr) {
            println!("Reached {}", self.ctr);
        }
        
        // if self.ctr == 20000 {
        //   println!("Done");
        //   panic!("");
        // }
        
        Ok(())
    }
}

impl std::fmt::Debug for PrintSub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrintSub")
    }
}


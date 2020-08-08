use crate::{
    schedule::{
        {SignalControl, CRON},
        pool::Subscriber,
        meta::CronMeta,
    },
    error::Error,
};

use tokio::net::TcpStream;
use std::{net::SocketAddr, time::{Duration, Instant}};

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
                (SignalControl::Success(true), Some(PortState::Open(state.addr)))
            ),
            Err(_e) => Ok(
                (SignalControl::Success(false), Some(PortState::Closed(state.addr)))
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
    ts: Instant,
    ctr: u64,
}

impl PrintSub {
    pub fn new() -> Self {
        Self {
            ts: Instant::now(),
            ctr: 0
        }
    }
}

#[async_trait::async_trait]
impl Subscriber<PortState, Job> for PrintSub {
    async fn handle(&mut self, meta: &mut CronMeta, signal: &SignalControl, resp: &Option<PortState>, state: &mut Job) -> Result<SignalControl, Error> {
        self.ctr += 1;
        //dbg!(self.ctr);
        if self.ts.elapsed() >= Duration::from_secs(60) {
            tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Rate sample: [{}/min | {}/sec | {}/ms]", self.ctr, self.ctr/60, (self.ctr/60)/60);
            self.ts = Instant::now();
            self.ctr = 0;
        }
        
        Ok(*signal)
    }
}

impl std::fmt::Debug for PrintSub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrintSub")
    }
}


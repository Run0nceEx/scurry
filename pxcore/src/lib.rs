#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![feature(async_closure)]

mod protocols;
mod scheduler;
mod processors;




use async_trait::async_trait;
use scheduler::{Schedule, CRON};
use std::{
    time::{Duration, Instant},
    net::SocketAddr,
};

enum Error {}


#[derive(Debug, Clone)]
enum Action {
    NoopConnection,
    ProtocolScan(/* */),
}

struct Response {
    ttl: Duration,
    ts_started: Instant,
    ts_stopped: Option<Instant>,
    addr: SocketAddr,
    action: Action,
    result: Option<Result<(), Error>>
}

impl Response {
    fn new(ttl: Duration, addr: SocketAddr, action: Action) -> Self
    {
        Self {
            ttl,
            ts_started: Instant::now(),
            ts_stopped: None,
            addr,
            action,
            result: None
        } 
    }
}


struct Miner(Schedule<SocketAddr, Response>);

struct Mine {
    addr: SocketAddr,
    action: Action
}

#[async_trait]
impl CRON<Response> for Mine {
    async fn exec(self) -> Response {
        let mut resp = Response::new(
            self.ttl(),
            self.addr,
            self.action.clone()
        );
        
        match self.action {
            Action::NoopConnection => {

            }

            Action::ProtocolScan( ) => {

            }
        } 

        unimplemented!()
    }

    fn check(&self) -> bool {
        true
    }
}

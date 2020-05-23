use async_trait::async_trait;
use scheduler::{Schedule, CRON};
use std::{
    time::{Duration, Instant},
    net::SocketAddr,
};
use smallvec::SmallVec;

#[derive(Clone)]
enum Error {}

const MAX_RESCHEDULE: usize = 4;

#[derive(Clone)]
enum Action {
    NoopConnection,
    ProtocolScan,
    ReschdeludeOnFail(Box<Action>, Duration,)
}

#[derive(Clone)]
struct Response {
    ttl: Duration,
    ts_started: Instant,
    ts_stopped: Option<Instant>,
    addr: SocketAddr,
    result: Result<bool, Error>
}

impl Response {
    fn new(ttl: Duration, addr: SocketAddr) -> Self
    {
        Self {
            ttl,
            ts_started: Instant::now(),
            ts_stopped: None,
            addr,
            result: Ok(false)
        }
    }
}


struct Miner(Schedule<SocketAddr, Response>);

struct Mine {
    addr: SocketAddr,
    action: Action,
}

#[async_trait]
impl CRON<Response> for Mine {
    async fn exec(self) -> Response {
        let mut resp = Response::new(
            self.ttl(),
            self.addr
        );

        unimplemented!()
    }

    fn check(&self) -> bool {
        true
    }
}

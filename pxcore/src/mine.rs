
use async_trait::async_trait;
use super::scheduler::{Schedule, CRON};
use std::{
    time::{Duration, Instant},
    net::SocketAddr,
};
use smallvec::SmallVec;

enum Error {}

const MAX_RESCHEDULE: usize = 3;

#[derive(Debug, Clone)]
enum Action<T> {
    NoopConnection,
    ProtocolScan(/* */),
    ReschdeludeOnFail(T, Duration, SmallVec<[Response<T>; MAX_RESCHEDULE]>)
}

#[derive(Debug, Clone)]
struct Response<T> {
    ttl: Duration,
    ts_started: Instant,
    ts_stopped: Option<Instant>,
    addr: SocketAddr,
    action: Action<T>,
    result: T
}

impl<'a> Response<Result<(), Error>> {
    fn new(ttl: Duration, addr: SocketAddr, action: Action<Result<(), Error>>) -> Self
    {
        Self {
            ttl,
            ts_started: Instant::now(),
            ts_stopped: None,
            addr,
            action,
            result: Ok(())
        }
    }
}


struct Miner<'a>(Schedule<SocketAddr, Response<'a, Result<(), Error>>>);

struct Mine {
    addr: SocketAddr,
    action: Action::ReschdeludeOnFail(Action::ProtocolScan(), Duration::from_secs(1.0, 0u32))
}

#[async_trait]
impl<'a, T> CRON<Response<Result<(), Error>>> for Mine {
    async fn exec(self) -> Response<Result<(), Error>> {
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

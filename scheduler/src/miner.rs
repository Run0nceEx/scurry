use corelib::{ProtocolIdentifier, Connector, Negotiate};

use super::schedule::{CronControls, CRON};
use tokio::{
    time::{Instant, Duration},
    net::TcpStream
};
use std::net::SocketAddr;
use smallvec::SmallVec;

const MAX_OBJECTS: usize = 24;
const MEASUREMENTS: usize = 5;

#[derive(Clone, Copy)]
pub struct MinedMeta {
    ts: Instant,
    duration: Option<Duration>,
}

impl MinedMeta {
    fn new() -> Self {
        Self {
            ts: Instant::now(),
            duration: None
        }
    }
}

#[derive(Clone)]
pub struct Mined {
    meta: MinedMeta,
    addr: SocketAddr,
    protocol: Option<String>,
    noop: bool,
}

impl Mined {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            meta: MinedMeta::new(),
            protocol: None,
            noop: false
        }
    }
}


struct MinerJob<C, P> {
    addr: SocketAddr,
    connector: std::marker::PhantomData<C>,
    scanners: Vec<Box<P>>,
    index: usize
}

#[async_trait::async_trait]
impl<C, P> CRON<CronControls<Mined>> for MinerJob<C, P> 
where 
    C: Connector + Send, // Constant
    P: ProtocolIdentifier<C> + Send + Sync // Dynamic     
{
    async fn exec(&mut self) -> CronControls<Mined> {
        let mut resp = Mined::new(self.addr);

        let mut stream = match C::init_connect(self.addr).await {
            Ok(stream) => stream,
            Err(e) => return CronControls::Reschedule(Duration::from_secs(360))
        };

        let scanner = &self.scanners[self.index];
        // Result<T, E> behaves like Result<&T> until unwrapped
        // so we may need some special result type internally that impls Clone + Send for
        // schedules - currently undetermined
        let res = scanner.detect(&mut stream).await.unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            false
        });

        resp.meta.duration = Some(resp.meta.ts.elapsed());
        
        let ret = match res {
            true => {
                CronControls::Success(resp)
            },

            _ => {
                CronControls::Reschedule(Duration::from_secs(3))
            }
        };
        
        CronControls::Drop
    }
}

#[derive(Clone)]
struct SpeedTest {
    results: [Duration; MEASUREMENTS],
    ctr: usize
}

struct SpeedTestJob {
    addr: SocketAddr,
    res: SpeedTest,
    tts: Duration
}

#[async_trait::async_trait]
impl CRON<CronControls<SpeedTest>> for SpeedTestJob {
    async fn exec(&mut self) -> CronControls<SpeedTest> {
        if MEASUREMENTS > self.res.ctr {
            
            let now = Instant::now();
            let result = TcpStream::connect(self.addr).await;
            let time_taken = now.elapsed();

            self.res.results[self.res.ctr] = time_taken;
            self.res.ctr += 1;
            
            return CronControls::Reschedule(self.tts)
        }

        CronControls::Success(self.res.clone())
    }
}

use super::{CRON, Scannable};
use super::schedule::CronControls;
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
    fn new(addr: SocketAddr) -> Self 
    {
        Self {
            addr,
            meta: MinedMeta::new(),
            protocol: None,
            noop: false
        }
    }
}


struct MinerJob {
    addr: SocketAddr,
    protos: SmallVec<[Box<dyn Scannable>; 24]>,
}

#[async_trait::async_trait]
impl CRON<CronControls<Mined>> for MinerJob {
    async fn exec(&mut self) -> CronControls<Mined> {
        let mut resp = Mined::new(self.addr);

        if let Some(protocol) = self.protos.pop() {
            let res = protocol.scan().await;
            resp.meta.duration = Some(resp.meta.ts.elapsed());
            
            let ret = match res {
                Ok(_) => {
                    resp.protocol = format!("{}", protocol);
                    CronControls::Success(resp)
                
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    CronControls::Reschedule(Duration::from_secs(3))
                }
            };

            return ret
        }
        
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
            TcpStream::connect(self.addr).await;
            let time_taken = now.elapsed();

            self.res.results[self.res.ctr] = time_taken;
            self.res.ctr += 1;
            
            return CronControls::Reschedule(self.tts)
        }

        CronControls::Success(self.res.clone())
    }
}
use crate::utils::{DiskRepr, IntoDiskRepr, FromDiskRepr, Mode};
use super::{CRON, PostProcessor, Scannable};

use tokio::{
    sync::{mpsc},
    time::timeout,
    task::JoinHandle,
};

use std::{
    time::{Duration, Instant},
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll}
};

use core::future::Future;
use serde::{Serialize, Deserialize};

use smallvec::{SmallVec};

const STACK_ALLOC_MAX: usize = 128;


/// Runs Post Processors against `receiver<R>`, Executed in order of the `post_hooks` (0..)
/// ```txt
/// -------------------------------------
///                    1st          2nd
///   post_hooks = [ processor -> processor ] -> Some(Released)
///                             ^ or drop
/// -------------------------------------
/// ```
pub async fn post_process<R>(rx: &mut mpsc::Receiver<R>, post_hooks: &[R]) -> Option<R> 
where 
    R: PostProcessor<R> + Copy
{
    if let Some(mut item) = rx.recv().await {
        for hook in post_hooks {
            match hook.process(item).await {
                Some(new_item) => {
                    item = new_item;
                }
                None => return None
            }
        }
        return Some(item)
    }
    None
}

pub struct Handles<J>(Vec<JoinHandle<Option<J>>>);

enum HandlesState<J> {
    Push(J),
    Pop,
    Pending,
    Error(Box<dyn std::error::Error>)
}

impl<J> Handles<J> {
    #[inline]
    fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    fn push(&mut self, item: JoinHandle<Option<J>>) {
        self.0.push(item)
    }

    /// Like `join` except non-blocking
    /// Seek for Resolved threads - remove resolved, keep unresolved until next pass
    async fn partial_join(&mut self) -> SmallVec<[J; STACK_ALLOC_MAX]> {

        let mut i: usize = 0;
        let mut indexes: SmallVec<[usize; STACK_ALLOC_MAX]> = SmallVec::new();
        
        let mut jobs: SmallVec<[J; STACK_ALLOC_MAX]> = SmallVec::new();

        for mut handle in &mut self.0 {
            let resp = futures::future::poll_fn(|cx: &mut Context| {
                match Pin::new(&mut handle).poll(cx) {
                    Poll::Ready(Ok(Some(job))) => Poll::Ready(HandlesState::Push(job)),
                    Poll::Ready(Ok(None)) => Poll::Ready(HandlesState::Pop),
                    Poll::Ready(Err(e)) => Poll::Ready(HandlesState::Error(Box::new(e))),
                    Poll::Pending => Poll::Ready(HandlesState::Pending)
                }
            }).await;
            
            match resp {
                HandlesState::Push(job) => {
                    jobs.push(job);
                    indexes.push(i);
                },
                HandlesState::Pop => indexes.push(i),
                HandlesState::Pending => {},
                HandlesState::Error(e) => {
                    // trace::log!("shit {}", e);   
                    indexes.push(i)
                }
            };

            i += 1;
        }

        remove_indexes(&mut self.0, &indexes[..]);
        jobs
    }
}

fn remove_indexes<T>(src: &mut Vec<T>, indexes: &[usize]) {
    let mut balancer: usize = 0;
    
    for rm_i in indexes {
        let i = rm_i - balancer;
        balancer += 1;
        src.remove(i);
    }
}


pub enum CronReturnSender {
    Success(Mined, Vec<Mined>),
    Failure(Vec<Mined>)
}

pub struct Schedule<Job>
{
    val_tx: mpsc::Sender<CronReturnSender>,
    commands: Vec<Job>,
    handles: Handles<Job>
}



pub enum ScheduleConstructor {
    Inherit(mpsc::Sender<Mined>),
    New
}

impl<T> Schedule<T> {
    fn append(&mut self, jobs: &[T]) where T: Clone {
        if jobs.len() > 0 {
            self.commands.extend(jobs.iter().map(|x| x.clone()))
        }
    }

    fn new(opts: ScheduleConstructor) -> (Self, mpsc::Receiver<CronReturnSender>) {
        let (vtx, vrx) = mpsc::channel(1024);
        
        let schedule = Self {
            val_tx: vtx,
            commands: Vec::with_capacity(1024),
            handles: Handles::new()
        };

        (schedule, vrx)
    }
}

struct ErrorKind {
    addr: SocketAddr,
    error: Box<std::error::Error>,
    proto: Box<Scannable>
}


// impl std::fmt::Display for ErrorKind {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[{}] {:?}", self.addr, self.error)
//     }
// } 
// impl std::error::Error for ErrorKind {}


pub enum CronReturnInternal {
    Success(Mined, Vec<Mined>),
    Failure(Vec<Mined>),
    Reschedule
}

impl<T> Schedule<T>
where
    T: CRON<CronReturnInternal> + Sync + Send + 'static + Clone + PartialEq,
{   
    /// Run tasks
    /// returns kill indexes
    fn run(&mut self) {
        let commands = self.commands.clone();

        for command in commands {
            if command.check() {       
                let mut vtx = self.val_tx.clone();
                if let Some(mut job) = self.commands.remove_item(&command) {

                    tokio::spawn(async move { 
                        // the only reason we can do this is bc async runtime is smol
                        // i tink :kek:

                        let og_schedule_max = job.max_reschedule();
                        let mut reschedule_cnt: usize = 0;
                        //let mut responses: Vec<Mined> = Vec::with_capacity(og_schedule_max+1);

                        while og_schedule_max > reschedule_cnt {
                            match timeout(job.ttl(), job.exec()).await {
                                Ok(val) => {
                                    let resp = match val {
                                        CronReturnInternal::Success(suc, attempts) => Some(CronReturnSender::Success(suc, attempts)),
                                        CronReturnInternal::Failure(attemps) => Some(CronReturnSender::Failure(attemps)),
                                        CronReturnInternal::Reschedule => None
                                    };

                                    if let Some(r) = resp {
                                        if let Err(e) = vtx.send(r).await {
                                            eprintln!("Failed to send back in Job: {}", e)
                                        }
                                    }

                                    // else {
                                    //     tokio::timer::sleep();
                                    // }
                                    
                                }
                                Err(e) => {
                                    eprintln!("{}", e);
                                    reschedule_cnt += 1;
                                    if job.reschedule() == false {
                                        break
                                    }
                                }
                            }
                        }
                    });

                }
            }

        }
    }
}


impl<'a, T: Serialize + Deserialize<'a>> DiskRepr for Schedule<T>
where T: IntoDiskRepr<T> + FromDiskRepr<'a, T> {}


#[derive(Copy, Clone)]
enum Error {}


#[derive(Clone, Copy)]
pub struct Mined {
    ttl: Duration,
    ts_started: Instant,
    ts_stopped: Option<Duration>,
    addr: SocketAddr,
}

impl Mined {
    fn new(addr: SocketAddr, ttl: Duration) -> Self {
        Self {
            ttl,
            addr,
            ts_started: Instant::now(),
            ts_stopped: None,
        }
    }
}

#[derive(Clone)]
enum Action {
    NoopConnection,
    ProtocolScan,
    //ReschdeludeOnFail(Box<Action>, Duration,)
}

struct MinerJob {
    addr: SocketAddr,
    protocol: Box<dyn Scannable>,


}

#[async_trait::async_trait]
impl CRON<Mined> for MinerJob {
    async fn exec(&mut self) -> Mined {
        let mut resp = Mined::new(self.addr, self.ttl());
        self.protocol.scan().await;
        resp.ts_stopped = Some(resp.ts_started.elapsed());

        resp
    }

    fn check(&self) -> bool {
        true
    }
}






// struct MinerSchedule<T>
// where 
//     T: CRON<Mined> + Sync + Send + Clone + 'static
// {
//     schedule: Schedule<T, Mined>,
//     receiver: mpsc::Receiver<Mined>,
//     scanners: Vec<Box<dyn Scannable>>,
// }




// // enum MineScheulde {
// //     Http(MinerSchedule<HttpHandler, Mined>),
// //     Socks5(MinerSchedule<SocksHandler, Mined>),
// // }

// impl<T> MinerSchedule<T>
// where 
//   T: CRON<Mined> + Scannable + Sync + Send + Clone + 'static + PartialEq {
  
//   #[inline]
//   fn new(scanners: Vec<Box<dyn Scannable>>, opt: ScheduleConstructor<Mined>) -> Self {
    
//     let (schedule, rx) = Schedule::new(opt);
    
//     Self {
//         schedule,
//         receiver: rx,
//         scanners: scanners
//     }
//   }

//   #[inline]
//   fn push(&mut self, jobs: &[T]) {
//       self.schedule.append(jobs);
//   }

//   #[inline]
//   fn run(&mut self) {
//     self.schedule.run();
//   }
// }

// #[derive(Serialize, Deserialize)]
// pub struct JobsSerialized<T>(Vec<T>);
// impl<T> DiskRepr for JobsSerialized<T> {}

// /// Partially serialize into raw bytes, 
// impl<'a, T, R> IntoDiskRepr<JobsSerialized<T>> for Schedule<T, R>
// where T: Serialize + Deserialize<'a> {
//     fn into_raw_repr(self) -> JobsSerialized<T> {
//         JobsSerialized(self.commands.into_iter().map(|x| x.clone()).collect())
//     }
// }

// /// Partially Deserialization raw bytes into original
// impl<'a, T, R> FromDiskRepr<'a, JobsSerialized<T>> for Schedule<T, R> where T: Deserialize<'a> {
//     fn from_raw_repr(&mut self, buf: &'a [u8], input: Mode) -> Result<(), Box<std::error::Error>> {
//         self.commands = bincode::deserialize::<JobsSerialized<T>>(buf)?.0
//             .into_iter()
//             .map(|x| Arc::new(x))
//             .collect();
//         Ok(())
//     }
// }
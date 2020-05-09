use crate::utils::{DiskRepr, IntoDiskRepr, FromDiskRepr, Mode};
use super::{CRON, PostProcessor, Scannable};
use std::future::Future;
use serde::{Serialize, Deserialize};

use tokio::{
    sync::mpsc,
    time::{timeout, Instant, Duration, DelayQueue, Error as TimeError},
    task::JoinHandle,
    stream::StreamExt
};

use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    collections::HashMap,
};

use smallvec::SmallVec;

const STACK_ALLOC_MAX: usize = 128;
const RESCHEDULES_ALLOW_MAX: usize = 128;

/// Runs Post Processors against `receiver<R>`, Executed in order of the `post_hooks` (0..)
/// ```txt
/// -------------------------------------
///                    1st          2nd
///   post_hooks = [ processor -> processor ] -> Some(Released)
///                             ^ or drop
/// -------------------------------------
/// ```
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
    /// Seeks for Resolved threads - remove resolved, keep unresolved until next pass
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
    Success(Mined),
    Failure(Error)
}

#[derive(Clone, Copy)]
pub struct CronMeta {
    ts: Instant,
    id: uuid::Uuid,
    tts: Duration, // time to sleep
    ttl: Duration // timeout duration/time to live
    // exec_duration: Option<Duration>,
}

impl CronMeta {
    fn new() -> Self {

        Self {
            ts: Instant::now(),
            id: uuid::Uuid::new_v4(),
            tts: Duration::from_secs(2),
            ttl: Duration::from_secs(2)
            //exec_duration: None
        }
    }
}

pub struct Schedule<Job> {
    val_tx: mpsc::Sender<CronReturnSender>,
    pending: HashMap<uuid::Uuid, (CronMeta, Job)>, // collection of pending jobs
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    handles: Handles<(CronMeta, Job)>,             // pending future handles
}


impl<T> Schedule<T> {
    pub fn insert(&mut self, job: T, meta: CronMeta) where T: Clone {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.pending.insert(meta.id, (CronMeta::new(), job));
    
    }

    pub fn new() -> (Self, mpsc::Receiver<CronReturnSender>) {
        let (vtx, vrx) = mpsc::channel(1024);
        
        let schedule = Self {
            val_tx: vtx,
            pending: HashMap::new(),
            timer: DelayQueue::new(),
            handles: Handles::new(),

        };

        (schedule, vrx)
    }

    /// Release jobs that ready to be fired
    async fn fire(&mut self) -> Result<SmallVec<[(CronMeta, T); 1024]>, TimeError> {
        let mut jobs: SmallVec<[(CronMeta, T); 1024]> = SmallVec::new();

        while let Some(res) = self.timer.next().await {
            let entry = res?;

            if let Some((meta, job)) = self.pending.remove(entry.get_ref()) {
                jobs.push((meta, job));
            }
        }

        Ok(jobs)
    }

    #[inline]
    async fn join(&mut self) where T: Clone {
        for (meta, job) in self.handles.partial_join().await {
            self.insert(job, meta)
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum Error {
    Something
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        unimplemented!()
    }
}


impl std::error::Error for Error {}

pub enum CronReturnInternal {
    Success(Mined),
    Failure(Error),
    Reschedule
}

impl<T> Schedule<T>
where
    T: CRON<CronReturnInternal> + Sync + Send + 'static + Clone + PartialEq,
{   
    
    /// Run tasks and collect
    pub async fn run(&mut self) {
        self.join().await;
        let jobs = self.fire().await.expect("iye matey, we're out of rum.");

        for (meta, mut job) in jobs {
            let mut vtx = self.val_tx.clone();

            let handle = tokio::spawn(async move { 
                match timeout(job.ttl(), job.exec()).await {
                    Ok(val) => {
                        let resp = match val {
                            CronReturnInternal::Reschedule => return Some((meta, job)),
                            CronReturnInternal::Success(success) => CronReturnSender::Success(success),
                            CronReturnInternal::Failure(e) => CronReturnSender::Failure(e)
                        };
                        
                        if let Err(e) = vtx.send(resp).await {
                            eprintln!("Failed to send back in Job: {}", e)
                        }
                    }

                    Err(e) => eprintln!("{}", e)
                }

                return None
            });
            
            self.handles.push(handle);       
        }
    }
}

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

#[derive(Clone, Copy)]
pub struct Mined {
    meta: MinedMeta,
    addr: SocketAddr,
}

impl Mined {
    fn new(addr: SocketAddr, ttl: Duration) -> Self {
        Self {
            addr,
            meta: MinedMeta::new()
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
    protocol: Vec<Box<dyn Scannable>>,
    index: usize,
    fire_ts: Instant

}

#[async_trait::async_trait]
impl CRON<CronReturnInternal> for MinerJob {
    async fn exec(&mut self) -> CronReturnInternal {
        let mut resp = Mined::new(self.addr, self.ttl());

        if let Some(scanner) = self.protocol.get(self.index) {
            let res = scanner.scan().await;
            resp.meta.duration = Some(resp.meta.ts.elapsed());
            
            if res {
                return CronReturnInternal::Success(resp);
            }

            else {
                self.index += 1;
                //self.fire_ts = Duration::from_secs(360);
                return CronReturnInternal::Reschedule
            }
        }

        CronReturnInternal::Failure(Error::Something)
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


impl<'a, T: Serialize + Deserialize<'a>> DiskRepr for Schedule<T>
where T: IntoDiskRepr<T> + FromDiskRepr<'a, T> {}


// #[derive(Serialize, Deserialize)]
// pub struct JobsSerialized<T>(Vec<T>);
// impl<T> DiskRepr for JobsSerialized<T> {}

// /// Partially serialize into raw bytes, 
// impl<'a, T, R> IntoDiskRepr<JobsSerialized<T>> for Schedule<T, R>
// where T: Serialize + Deserialize<'a> {
//     fn into_raw_repr(self) -> JobsSerialized<T> {
//         JobsSerialized(self.jobs.into_iter().map(|x| x.clone()).collect())
//     }
// }

// /// Partially Deserialization raw bytes into original
// impl<'a, T, R> FromDiskRepr<'a, JobsSerialized<T>> for Schedule<T, R> where T: Deserialize<'a> {
//     fn from_raw_repr(&mut self, buf: &'a [u8], input: Mode) -> Result<(), Box<std::error::Error>> {
//         self.jobs = bincode::deserialize::<JobsSerialized<T>>(buf)?.0
//             .into_iter()
//             .map(|x| Arc::new(x))
//             .collect();
//         Ok(())
//     }
// }

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

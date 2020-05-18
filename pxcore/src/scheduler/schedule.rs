use super::{CRON, PostProcessor};
use std::future::Future;

use tokio::{
    sync::mpsc,
    time::{timeout, Instant, Duration, DelayQueue, Error as TimeError},
    task::JoinHandle,
    stream::StreamExt
};

use std::{
    pin::Pin,
    task::{Context, Poll},
    collections::HashMap,
};

use smallvec::SmallVec;

const STACK_ALLOC_MAX: usize = 256;
const MAX_RESCHEDULES: usize = 256;

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

#[derive(Clone, Copy)]
pub struct CronMeta {
    id: uuid::Uuid,
    created: Instant,
    tts: Duration, // time to sleep
    ttl: Duration, // time to live
    ctr: usize,
}

impl CronMeta {
    fn new(timeout: Duration, fire_at: Duration) -> Self {

        Self {
            id: uuid::Uuid::new_v4(),
            created: Instant::now(),
            tts: fire_at,
            ttl: timeout,
            ctr: 0,
        }
    }
}



pub struct Schedule<Job, R> {
    //CronReturnSender<R, E>
    val_tx: mpsc::Sender<R>,
    pending: HashMap<uuid::Uuid, (CronMeta, Job)>, // collection of pending jobs
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    handles: Handles<(CronMeta, Job)>,             // pending future handles
}


impl<T, R> Schedule<T, R> {
    pub fn insert(&mut self, job: T, meta: CronMeta) where T: Clone {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.pending.insert(meta.id, (meta, job));
    
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
    
    /// partially collect returned values from thread,
    /// like `tokio::task::JoinHandle.join` but non-blocking
    async fn join(&mut self) where T: Clone {
        for (mut meta, job) in self.handles.partial_join().await {
            if meta.ctr >= MAX_RESCHEDULES {
                eprintln!("[{}] exceed reschedule limit", meta.id);
                continue
            }

            meta.ctr += 1;
            self.insert(job, meta)
        }
    }
    
    #[inline]
    pub fn new() -> (Self, mpsc::Receiver<R>) {
        let (vtx, vrx) = mpsc::channel(1024);
        
        let schedule = Self {
            val_tx: vtx,
            pending: HashMap::new(),
            timer: DelayQueue::new(),
            handles: Handles::new(),

        };

        (schedule, vrx)
    }
}


pub enum CronControls<R> {
    Reschedule(Duration),
    Success(R),
    Drop,
} 

impl<T, R> Schedule<T, R>
where
    T: CRON<CronControls<R>> + Sync + Send + Clone + 'static,
    R: Send + 'static
{   
    
    /// Run tasks and collect
    pub async fn run(&mut self) {
        self.join().await;
        let jobs = self.fire().await.expect("Polling schedule failed.");

        for (mut meta, mut job) in jobs {
            let mut vtx = self.val_tx.clone();
            
            let handle = tokio::spawn(async move { 
                match timeout(meta.ttl, job.exec()).await {
                    Ok(ctrl) => {
                        match ctrl {
                            CronControls::Reschedule(ttl) => {
                                meta.ttl = ttl;
                                return Some((meta, job))
                            }

                            CronControls::Success(resp) => {
                                //todo(adam) add handler for processing meta data
                                if let Err(e) = vtx.send(resp).await {
                                    eprintln!("Failed to send back in Job: {}", e)
                                }
                                return None
                            }

                            CronControls::Drop => return None
                        };
                    }
                    Err(e) => eprintln!("Timed out: {}", e)
                }
                return None
            });

            self.handles.push(handle);  
        }
    }
}




// #[derive(Copy, Clone, Debug)]
// pub enum Error {
//     Something
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         unimplemented!()
//     }
// }

// impl std::error::Error for Error {}




//------------------------------------------------------

// impl<'a, T: Serialize + Deserialize<'a>> DiskRepr for Schedule<T>
// where T: IntoDiskRepr<T> + FromDiskRepr<'a, T> {}


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

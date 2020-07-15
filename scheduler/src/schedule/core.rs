#![allow(dead_code)]
use super::CRON;
use crate::error::Error;

use tokio::{
    sync::mpsc,
    time::{timeout, Instant, Duration, DelayQueue, Error as TimeError},
    stream::StreamExt,
    task::JoinHandle,
};

use std::{
    collections::HashMap,
    pin::Pin,
    task::{Poll, Context},
    future::Future
};

use futures::future::poll_fn;
use tracing::{event, Level};
use smallvec::SmallVec;

const CHUNK_SIZE: usize = 256;

#[derive(Copy, Clone)]
pub enum ScheduleControls<R> {
    /// Operations went according to plan, 
    /// and requesting to be reschedule again
    Reschedule(Duration),

    /// Operations failed and would like to attemp again without a specified time
    Retry,

    /// Operation Succeeded and given value
    Success(R),

    /// Operation was nullified either because of no result, or unreported error
    Drop(R),

    Fuck,
} 

#[derive(Clone, Copy)]
pub struct CronMeta {
    pub id: uuid::Uuid,
    pub created: Instant,
    pub tts: Duration, // time to sleep
    pub ttl: Duration, // time to live
    pub ctr: usize,    // operation counter
    pub max_ctr: usize, // fail/retry counter
}


impl CronMeta {
    pub fn new(timeout: Duration, fire_in: Duration, max_retry: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created: Instant::now(),
            tts: fire_in,
            ttl: timeout,
            ctr: 0,
            max_ctr: max_retry,
        }
    }
}

#[derive(Debug)]
enum HandlesState<J> {
    Push(J),
    Pop,
    Drop,
    Pending,
    Error(Box<dyn std::error::Error>)
}

#[derive(Debug)]
pub struct Handle<R> {
    pub uuid: uuid::Uuid,
    pub inner: JoinHandle<R>,
}

/// partially "join" threads to get completed results in manager,
/// 
async fn partial_join<R, S>(handles: &mut Vec<Handle<(ScheduleControls<R>, CronMeta, S)>>, reschedules: &mut Vec<(ScheduleControls<R>, CronMeta, S)>) {
    let mut keys: Vec<usize> = Vec::with_capacity(20000);
    
    let mut ctr = 0;
    
    for handle in handles.iter_mut() {
        let resp = poll_fn(|cx: &mut Context| {
            match Pin::new(&mut handle.inner).poll(cx) {
                Poll::Pending => Poll::Ready(HandlesState::Pending),
                Poll::Ready(Ok(resp)) => Poll::Ready(HandlesState::Push(resp)),
                Poll::Ready(Err(e)) => Poll::Ready(HandlesState::Error(Box::new(e)))
            }
        }).await;
        
        match resp {
            HandlesState::Push(result) => {
                reschedules.push(result);
                keys.push(ctr);
            }
            
            HandlesState::Error(e) => eprintln!("FUCK {}", e),
            _ => {}
        }
        
        
        ctr += 1;
    }

    for (i, x) in keys.iter().enumerate() {
        println!("removing handle {}",x - i);
        println!("x: {}, i: {}",x , i);
        //remove and balance index for completed handles
        handles.remove(x - i);
    }
}

/// Release jobs that ready to be fired
/// expected to be
async fn release_due<S>(
    timer: &mut DelayQueue<uuid::Uuid>,
    pending: &mut HashMap<uuid::Uuid, (CronMeta, S)>,
    job_buf: &mut Vec<(CronMeta, S)>) -> Result<(), TimeError>
{    
    while let Some(res) = timer.next().await {
        let entry = res?;
        
        if let Some((meta, state)) = pending.remove(entry.get_ref()) {
            job_buf.push((meta, state));
        }
    }
    Ok(())
}

pub struct ThreadController<J, R, S>
where 
    J: CRON<Response=ScheduleControls<R>, State=S>,
    R: Send + Clone + Sync + 'static
{
    
    _job: std::marker::PhantomData<J>,
    handles: Vec<Handle<(ScheduleControls<R>, CronMeta, S)>>,
    tx: mpsc::Sender<R>,
}

impl<J, R, S> ThreadController<J, R, S> 
where 
    J: CRON<Response=ScheduleControls<R>, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Sync + Clone + 'static
{
    #[inline]
    pub fn new() -> (Self, mpsc::Receiver<R>) {
        let (vtx, vrx) = mpsc::channel(1024);
        
        let tctrl = ThreadController {
            _job: std::marker::PhantomData::<J>,
            tx: vtx,
            handles: Vec::new(),

        };

        (tctrl, vrx)
    }

    pub fn fire(&mut self, meta: CronMeta, state: S) {
        let mut vtx = self.tx.clone();

        let handle = tokio::spawn(async move {
            tracing::event!(target: "Schedule Thread", Level::INFO, "Firing job {}", meta.id);
            let prev_state = state.clone();

            let (state, ctrl) = match timeout(meta.ttl, J::exec(state)).await {
                Ok((ctrl, state)) => (state, ctrl),
                
                Err(e) => {
                    //eprintln!("Error: {}", e);
                    return (ScheduleControls::Retry, meta, prev_state);
                }
            };
            
            tracing::event!(target: "Schedule Thread", Level::INFO, "Completed job {}", meta.id);
            
            match ctrl {
                ScheduleControls::Success(resp) => {
                    let resp: R = resp;
                    if let Err(e) = vtx.send(resp.clone()).await {
                        eprintln!("{}", e)
                    }
                    return (ScheduleControls::Success(resp), meta, state)
                }
                _ => return (ctrl, meta, state)    
            }
        });

        let hldr = Handle {
            inner: handle,
            uuid: meta.id
        };

        self.handles.push(hldr);
    }

    /// partially collect returned values from thread,
    /// like `tokio::task::JoinHandle.join` but non-blocking
    async fn join(&mut self, buf: &mut Vec<(CronMeta, S)>)
    {   
        let mut tmp: Vec<(ScheduleControls<R>, CronMeta, S)> = Vec::new();

        partial_join(&mut self.handles, &mut tmp).await;
        
        for (ctrl, mut meta, state) in tmp {
            let mut vtx = self.tx.clone();
            
            match ctrl {
                ScheduleControls::Reschedule(_) | ScheduleControls::Retry => {     
                    println!("reschedule");
                    if let ScheduleControls::Reschedule(tts) = ctrl {
                        meta.tts = tts;
                    }


                    if meta.ctr <= meta.max_ctr {
                        meta.ctr += 1;
                        buf.push((meta, state))
                    }
                }
                ScheduleControls::Drop(r) => {
                    
                    println!("dropping whatever ")
                },
                ScheduleControls::Fuck => {
                    println!("we errors")
                },
                ScheduleControls::Success(r) => {
                    if let Err(e) = vtx.send(r.clone()).await {
                        eprintln!("{}", e)
                    }
                }
            }

            // let mut ctr = 0;
            // for x in &self.handles {
            //     ctr += 1;
            //     if x.uuid == meta.id {
            //         break
            //     }
            // }
            
            // self.handles.remove(ctr);
        }
    }
}


pub struct Schedule<J, R, S>
where 
    J: CRON<Response=ScheduleControls<R>, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Clone + Sync
{
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    threads: ThreadController<J, R, S>,
    bank: HashMap<uuid::Uuid, (CronMeta, S)>,      // collection of pending jobs
    job_buf: Vec<(CronMeta, S)>
}


impl<J, R, S> Schedule<J, R, S> 
where 
    J: CRON<Response=ScheduleControls<R>, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Clone + Sync + 'static
{
    pub fn insert(&mut self, meta: CronMeta, state: S) {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.bank.insert(meta.id, (meta, state));
    
    }
    
    #[inline]
    pub fn new() -> (Self, mpsc::Receiver<R>) {
        let (tctrl, vrx) = ThreadController::new();
        
        let schedule = Self {
            threads: tctrl,
            bank: HashMap::new(),
            timer: DelayQueue::new(),
            job_buf: Vec::with_capacity(2048),
        };

        (schedule, vrx)
    }

    /// Run tasks and collect
    pub async fn spawn_ready(&mut self) -> Result<(), Error> 
    where 
        R: Send + 'static + Clone + Sync
    {   
        release_due(&mut self.timer, &mut self.bank, &mut self.job_buf).await?;
        self.threads.join(&mut self.job_buf).await;
        
        for (meta, state) in self.job_buf.drain(..) {
            self.threads.fire(meta, state);
        }

        Ok(())
    }

    /// Run tasks and collect
    pub fn handles(&self) -> usize 
    {   
        self.threads.handles.len()
    }
}

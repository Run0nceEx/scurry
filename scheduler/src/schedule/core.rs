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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Clone, Copy)]
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

fn spawn_worker<J, R, S>(mut vtx: mpsc::Sender<(ScheduleControls<R>, CronMeta, S)>, meta: CronMeta, state: S)
where 
    J: CRON<Response=ScheduleControls<R>, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Sync + Clone + 'static
{
    tokio::spawn(async move {
        tracing::event!(target: "Schedule Thread", Level::INFO, "Firing job {}", meta.id);
        let prev_state = state.clone();

        let (state, ctrl) = match timeout(meta.ttl, J::exec(state)).await {
            Ok((ctrl, state)) => (state, ctrl),
            
            Err(e) => {
                (prev_state, ScheduleControls::Retry)
            }
        };
        
        tracing::event!(target: "Schedule Thread", Level::INFO, "Completed job {}", meta.id);
        vtx.send((ctrl, meta, state)).await;
    });
}



pub struct Schedule<J, R, S>
where 
    J: CRON<Response=ScheduleControls<R>, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Clone + Sync
{
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    tx: mpsc::Sender<(ScheduleControls<R>, CronMeta, S)>,
    bank: HashMap<uuid::Uuid, (CronMeta, S)>,      // collection of pending jobs
    job_buf: Vec<(CronMeta, S)>,
    _job: std::marker::PhantomData<J>
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
    pub fn new() -> (Self, mpsc::Receiver<(ScheduleControls<R>, CronMeta, S)>) {
        let (tx, rx) = mpsc::channel(100000);
        
        let schedule = Self {
            tx: tx,
            bank: HashMap::new(),
            timer: DelayQueue::new(),
            job_buf: Vec::with_capacity(2048),
            _job: std::marker::PhantomData
        };

        (schedule, rx)
    }

    /// Run tasks and collect
    pub async fn spawn_ready(&mut self) -> Result<(), Error> 
    where 
        R: Send + 'static + Clone + Sync
    {   
        release_due(&mut self.timer, &mut self.bank, &mut self.job_buf).await?;
        
        for (meta, state) in self.job_buf.drain(..) {
            spawn_worker::<J, R, S>(self.tx.clone(), meta, state);
        }

        Ok(())
    }
}

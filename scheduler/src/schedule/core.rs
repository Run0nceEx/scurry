use super::CRON;
use crate::error::Error;
use super::proc::*;

use tokio::{
    sync::mpsc,
    time::{timeout, Instant, Duration, DelayQueue, Error as TimeError},
    stream::StreamExt
};

use std::{
    collections::HashMap,
};

const STACK_ALLOC_MAX: usize = 256;
const MAX_RESCHEDULES: usize = 256;
const CHUNK_SIZE: usize = 256;


#[derive(Clone)]
pub struct CronMeta<Job, R> {
    id: uuid::Uuid,
    created: Instant,
    tts: Duration, // time to sleep
    ttl: Duration, // time to live
    ctr: usize,    // operation counter
    retry_ctr: usize, // fail/retry counter
    state: Job,
    last_ctrl: Option<CronControls<R>>
}


impl<J, R> CronMeta<J, R> where J: Default {
    pub fn new(timeout: Duration, fire_at: Duration) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created: Instant::now(),
            tts: fire_at,
            ttl: timeout,
            ctr: 0,
            retry_ctr: 0,
            state: J::default(),
            last_ctrl: None
        }
    }

    pub fn update_state(&mut self, ctrl: CronControls<R>) {
        self.last_ctrl = Some(ctrl);
    }
}

pub struct Schedule<Job, R> {
    val_tx: mpsc::Sender<R>,
    pending: HashMap<uuid::Uuid, CronMeta<Job, R>>, // collection of pending jobs
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    handles: Handles<CronMeta<Job, R>>,             // pending future handles
}


impl<T, R> Schedule<T, R> {
    pub fn insert(&mut self, meta: CronMeta<T, R>) {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.pending.insert(meta.id, meta);
    
    }

    /// Release jobs that ready to be fired
    async fn fire(&mut self) -> Result<Vec<CronMeta<T, R>>, TimeError> {
        let mut jobs: Vec<CronMeta<T, R>> = Vec::with_capacity(CHUNK_SIZE);

        while let Some(res) = self.timer.next().await {
            let entry = res?;

            if let Some(meta) = self.pending.remove(entry.get_ref()) {
                jobs.push(meta);
            }
        }

        Ok(jobs)
    }
   
    /// partially collect returned values from thread,
    /// like `tokio::task::JoinHandle.join` but non-blocking
    async fn join(&mut self) {
        let mut job_buf = Vec::with_capacity(256);
        self.handles.partial_join(&mut job_buf).await;

        for mut meta in job_buf {
            if meta.ctr >= MAX_RESCHEDULES {
                eprintln!("[{}] exceed reschedule limit", meta.id);
                continue
            }

            meta.ctr += 1;
            self.insert(meta)
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

#[derive(Clone)]
pub enum CronControls<R> {
    /// Operations went according to plan, 
    /// and requesting to be reschedule again
    Reschedule(Duration),

    /// Operations failed and would like to attemp again
    Retry(Duration),

    /// Operation Succeeded and given value
    Success(R),

    /// Operation was nullified either because of no result, or unreported error
    Drop,
} 

impl<T, R> Schedule<T, R>
where
    T: CRON<CronControls<R>> + Sync + Send + Clone + 'static,
    R: Send + 'static + Clone
{   
    
    /// Run tasks and collect
    pub async fn run(&mut self) -> Result<(), Error> {
        self.join().await;             // Partial join results
        let jobs = self.fire().await?; // Ready to fire

        for meta in jobs {
            let vtx = self.val_tx.clone();
            
            let handle = tokio::spawn(async move {
                return handle_proc(meta, vtx).await; 
            });

            self.handles.push(handle);
        }

        Ok(())
    }
}


async fn handle_proc<J, R>(mut meta: CronMeta<J, R>, mut vtx: mpsc::Sender<R>) -> Option<CronMeta<J, R>>
where 
    J: CRON<CronControls<R>> + Clone,
    R: Clone
{   
    
    let ctrl = match timeout(meta.ttl, meta.state.exec()).await {
        Ok(ctrl) => ctrl,
        Err(e) => {
            eprintln!("Error: {}", e);
            return None
        }
    };
    
    meta.last_ctrl = Some(ctrl.clone());

    match ctrl {
        CronControls::Reschedule(tts) | CronControls::Retry(tts) => {
            meta.tts = tts;
            return Some(meta)
        }

        CronControls::Success(resp) => {
            //todo(adam) add handler for processing meta data
            if let Err(e) = vtx.send(resp).await {
                eprintln!("Failed to send back in Job: {}", e)
            }
            return None
        }

        CronControls::Drop => return None,

    }
    
}
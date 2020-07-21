#![allow(dead_code)]
use super::{CRON, SignalControl};
use super::meta::CronMeta;

use crate::error::Error;

use tokio::{
    sync::mpsc,
    time::{DelayQueue, Error as TimeError},
    stream::StreamExt,
};

use std::{
    collections::HashMap,
};

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


pub struct Schedule<J, R, S>
where 
    J: CRON<Response=R, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Clone + Sync
{
    pub tx: mpsc::Sender<(CronMeta, SignalControl<R>, S)>,
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    bank: HashMap<uuid::Uuid, (CronMeta, S)>,      // collection of pending jobs

    _job: std::marker::PhantomData<J>
}


impl<J, R, S> Schedule<J, R, S> 
where 
    J: CRON<Response=R, State=S>,
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
    pub fn new(channel_size: usize) -> (Self, mpsc::Receiver<(CronMeta, SignalControl<R>, S)>) {
        let (tx, rx) = mpsc::channel(channel_size);
        
        let schedule = Self {
            tx: tx,
            bank: HashMap::new(),
            timer: DelayQueue::new(),
            _job: std::marker::PhantomData
        };

        (schedule, rx)
    }

    /// Release tasks from Timer
    pub async fn release_ready(&mut self, reschedule_jobs: &mut Vec<(CronMeta, S)>) -> Result<(), Error> 
    where 
        R: Send + 'static + Clone + Sync
    {   
        release_due(&mut self.timer, &mut self.bank, reschedule_jobs).await?;

        Ok(())
    }
}

use super::meta::CronMeta;

use crate::error::Error;

use tokio::{
    sync::mpsc,
    time::DelayQueue,
    stream::StreamExt,
};

use std::{
    collections::HashMap,
    time::Duration
};

/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error>;

    fn name() -> String;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SignalControl {
    /// Drop memory, and give a boolean to tell if we connected 
    Success(bool), // Boolean to signify to the scheduler if we connected to the target or not
    
    /// and requesting to be reschedule again
    Reschedule(Duration),

    /// Operations failed and would like to attemp again, 
    /// it will sleep again for whatever it's time to sleep paramenter was set to. (tts)
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,
}

pub struct Schedule<J, R, S>
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync,
    S: Send + Sync
{
    pub tx: mpsc::Sender<(CronMeta, SignalControl, Option<R>, S)>,
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    bank: HashMap<uuid::Uuid, (CronMeta, S)>,      // collection of pending jobs

    _job: std::marker::PhantomData<J>
}


impl<J, R, S> Schedule<J, R, S> 
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync,
    S: Send + Sync
{
    pub fn insert(&mut self, meta: CronMeta, state: S) {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.bank.insert(meta.id, (meta, state));
    }
    
    #[inline]
    pub fn new(channel_size: usize) -> (Self, mpsc::Receiver<(CronMeta, SignalControl, Option<R>, S)>) {
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
    /// If `max` is 0, no limit is occured
    pub async fn release_ready(&mut self, reschedule_jobs: &mut Vec<(CronMeta, S)>) -> Result<(), Error> 
    {
        while let Some(res) = self.timer.next().await {
            if let Some((meta, state)) = self.bank.remove(res?.get_ref()) {
                reschedule_jobs.push((meta, state));
            }
        }
        Ok(())
    }
}




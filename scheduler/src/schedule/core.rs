#![allow(dead_code)]
use super::CRON;
use super::sig::{SignalControl};
use super::meta::CronMeta;

use crate::error::Error;

use tokio::{
    sync::mpsc,
    time::{timeout, DelayQueue, Error as TimeError},
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

fn spawn_worker<J, R, S>(
    mut vtx: mpsc::Sender<(CronMeta, SignalControl<(Option<R>, S)>)>,
    meta: CronMeta,
    state: S
) where 
    J: CRON<Response=R, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Sync + Clone + 'static
{
    tokio::spawn(async move {
        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Firing job {}", meta.id);

        let prev_state = state.clone();

        let ctrl = match timeout(meta.ttl, J::exec(state)).await {
            Ok(Ok(ctrl)) => ctrl,
            
            Err(e)   => {
                if meta.ctr >= meta.max_ctr {
                    println!("timeout error? {}", e );
                    SignalControl::Retry((None, prev_state))
                }
                else {
                    SignalControl::Drop((None, prev_state))
                }
            }

            //Something other than a timeout error
            Ok(Err(e)) => {
                eprintln!("Error in Job: {}", e);   
                SignalControl::Fuck
            }
        };
        
        let id = meta.id;
        
        if let Err(e) = vtx.send((meta, ctrl)).await {
            eprintln!("channel error: {}", e)
        }
        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Completed job {}", id);
    });
}



pub struct Schedule<J, R, S>
where 
    J: CRON<Response=R, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Clone + Sync
{
    tx: mpsc::Sender<(CronMeta, SignalControl<(Option<R>, S)>)>,
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    bank: HashMap<uuid::Uuid, (CronMeta, S)>,      // collection of pending jobs
    job_buf: Vec<(CronMeta, S)>,

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
    pub fn new(channel_size: usize) -> (Self, mpsc::Receiver<(CronMeta, SignalControl<(Option<R>, S)>)>) {
        let (tx, rx) = mpsc::channel(channel_size);
        
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

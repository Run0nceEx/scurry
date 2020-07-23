/*
Public interface for schedules

*/

use super::{
    core::{Schedule},
    meta::CronMeta,
    CRON, SignalControl
};
use tokio::sync::mpsc;
use smallvec::SmallVec;
use crate::error::Error;


#[async_trait::async_trait]
pub trait Subscriber<R, S>: std::fmt::Debug {
    async fn handle(&mut self, 
        meta: &mut CronMeta,
        signal: &mut SignalControl,
        data: &Option<R>,
        state: &mut S
    ) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait MetaSubscriber: std::fmt::Debug {
    async fn handle(&mut self, meta: &mut CronMeta, signal: &mut SignalControl) -> Result<(), Error>;
}

struct Listener<R, S>
{
    channel: mpsc::Receiver<(CronMeta, SignalControl, Option<R>, S)>,
    subscribers: SmallVec<[Box<dyn Subscriber<R, S>>; 8]>,
    meta_subscribers: SmallVec<[Box<dyn MetaSubscriber>; 8]>
}


impl<R, S> Listener<R, S>
{
    pub fn new(channel: mpsc::Receiver<(CronMeta, SignalControl, Option<R>, S)>) -> Self {
        Self {
            channel,
            subscribers: SmallVec::new(),
            meta_subscribers: SmallVec::new()
        }
    }

    pub async fn process(&mut self, rescheduled_jobs: &mut Vec<(CronMeta, S)>) where R: std::fmt::Debug {
        process_subscribers(
            &mut self.channel,
            &mut self.subscribers[..],
            &mut self.meta_subscribers[..],
            rescheduled_jobs
        ).await;
    }


    pub fn add_sub<T>(&mut self, sub: T ) where T: Subscriber<R, S> + 'static {
        self.subscribers.push(Box::new(sub))
    }


    pub fn add_meta_sub<T>(&mut self, sub: T ) where T: MetaSubscriber + 'static {
        self.meta_subscribers.push(Box::new(sub))
    }
}

async fn process_subscribers<R, S>(
    channel: &mut  mpsc::Receiver<(CronMeta, SignalControl, Option<R>, S)>,
    subs: &mut [Box<dyn Subscriber<R, S>>],
    meta_subs: &mut [Box<dyn MetaSubscriber>],
    ret_buf: &mut Vec<(CronMeta, S)>
){
    if let Some((mut meta, mut ctrl, resp, mut state)) = channel.recv().await {
        
        for meta_hdlr in meta_subs.iter_mut() {
            if let Err(e) = meta_hdlr.handle(&mut meta, &mut ctrl).await {
                eprintln!("Error while handling [{:?}] {}", meta_hdlr, e);    
            }
        }
        
        for hdlr in subs.iter_mut() {
            if let Err(e) = hdlr.handle(&mut meta, &mut ctrl, &resp, &mut state).await {
                eprintln!("Error while handling [{:?}] {}", hdlr, e);
            }
        }
        
        match ctrl {
            SignalControl::Reschedule(tts) => {
                if meta.ctr <= meta.max_ctr {
                    meta.tts = tts;
                    meta.ctr += 1;
                    ret_buf.push((meta, state))
                }
            },
                
            SignalControl::Retry => {
                if meta.ctr <= meta.max_ctr {
                    meta.ctr += 1;
                    ret_buf.push((meta, state))
                }
            },
                
            SignalControl::Drop | SignalControl::Success | SignalControl::Fuck => {}
        }
    }
}


pub struct ScheduledJobPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Clone + Sync + 'static,
	S: Send + Clone + Sync,
{
	schedule: Schedule<J, R, S>,
	listener: Listener<R, S>,
}


impl<J, R, S> ScheduledJobPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Clone + Sync + 'static + std::fmt::Debug,
	S: Send + Clone + Sync + 'static + std::fmt::Debug
{   

	pub fn new(channel_size: usize) -> Self {
		let (schedule, rx) = Schedule::new(channel_size);
		Self {
			schedule: schedule,
			listener: Listener::new(rx)
		}
	}

	pub fn subscribe<T>(&mut self, sub: T)
	where 
		T: Subscriber<R, S> + 'static
	{
		self.listener.add_sub(sub);
	}

	pub fn insert(&mut self, job: S,  timeout: std::time::Duration, fire_in: std::time::Duration, max_retry: usize) {
        let meta = CronMeta::new(timeout, fire_in, max_retry);
        self.schedule.insert(meta, job);
    }
    
    /// syntacially this function is called `process_reschedules` but it does do more
    /// It processes all the data the comes across the channel including reschedule
    pub async fn process_reschedules(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        self.listener.process(rescheduled_buf).await;
        //println!("")
    }

    /// Fires jobs that are ready
    /// If the max counter is 0 then no limiter will be set
    pub async fn release_ready(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) -> Result<(), Error> {
        self.schedule.release_ready(rescheduled_buf).await?;
        Ok(())
    }

    pub fn fire_jobs(&self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        for (meta, state) in rescheduled_buf.drain(..) {
            spawn_worker::<J, R, S>(self.schedule.tx.clone(), meta, state)
        }
    }
}



fn spawn_worker<J, R, S>(
    mut vtx: mpsc::Sender<(CronMeta, SignalControl, Option<R>, S)>,
    meta: CronMeta,
    mut state: S
) where 
    J: CRON<Response=R, State=S>,
    R: Send + Clone + Sync + 'static,
    S: Send + Sync + Clone + 'static
{
    tokio::spawn(async move {
        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Firing job {}", meta.id);

        let (sig, resp) = match tokio::time::timeout(meta.ttl, J::exec(&mut state)).await {
            
            Ok(Ok((sig, resp))) => (sig, resp),

            Err(_) | Ok(Err(_)) => {
                if meta.ctr >= meta.max_ctr {
                    (SignalControl::Retry, None)
                }
                else {
                    (SignalControl::Drop, None)
                }
            }
        };
        
        let id = meta.id;
        
        if let Err(e) = vtx.send((meta, sig, resp, state)).await {
            eprintln!("channel error: {}", e)
        }

        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Completed job {}", id);
    });
}

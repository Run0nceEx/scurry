/*
Public interface for schedules

*/

use super::{
    core::{Schedule},
    meta::CronMeta,
    sig::*,
    CRON,
};
use tokio::sync::mpsc;
use smallvec::SmallVec;
use crate::error::Error;


#[async_trait::async_trait]
pub trait Subscriber<T>: std::fmt::Debug {
    async fn handle(&mut self, meta: &CronMeta, data: &T) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait MetaSubscriber: std::fmt::Debug {
    async fn handle(&mut self, data: &CronMeta) -> Result<(), Error>;
}

struct Listener<T>
{
    channel: mpsc::Receiver<(CronMeta, SignalControl<T>)>,
    subscribers: SmallVec<[Box<dyn Subscriber<T>>; 8]>,
    meta_subscribers: SmallVec<[Box<dyn MetaSubscriber>; 8]>
}


impl<R> Listener<R>
{
    pub fn new(channel: mpsc::Receiver<(CronMeta, SignalControl<R>)>) -> Self {
        Self {
            channel,
            subscribers: SmallVec::new(),
            meta_subscribers: SmallVec::new()
        }
    }

    pub async fn process(&mut self, rescheduled_jobs: &mut Vec<(CronMeta, R)>) where R: std::fmt::Debug {
        process_subscribers(
            &mut self.channel,
            &mut self.subscribers[..],
            &mut self.meta_subscribers[..],
            rescheduled_jobs
        ).await;
    }


    pub fn add_sub<T>(&mut self, sub: T ) where T: Subscriber<R> + 'static {
        self.subscribers.push(Box::new(sub))
    }


    pub fn add_meta_sub<T>(&mut self, sub: T ) where T: MetaSubscriber + 'static {
        self.meta_subscribers.push(Box::new(sub))
    }
}

async fn process_subscribers<T>(
    channel: &mut mpsc::Receiver<(CronMeta, SignalControl<T>)>,
    subs: &mut [Box<dyn Subscriber<T>>],
    meta_subs: &mut [Box<dyn MetaSubscriber>],
    ret_buf: &mut Vec<(CronMeta, T)>
){
    if let Some((mut meta, ctrl)) = channel.recv().await {
        match ctrl {
            SignalControl::Success(data) => {
                for meta_hdlr in meta_subs.iter_mut() {
                    if let Err(e) = meta_hdlr.handle(&meta).await {
                        eprintln!("Error while handling [{:?}] {}", meta_hdlr, e);    
                    }
                }
                
                for hdlr in subs.iter_mut() {
                    if let Err(e) = hdlr.handle(&meta, &data).await {
                        eprintln!("Error while handling [{:?}] {}", hdlr, e);
                    }
                }
            }

            SignalControl::Reschedule(state, tts) => {
                if meta.ctr <= meta.max_ctr {
                    meta.tts = tts;
                    meta.ctr += 1;
                    ret_buf.push((meta, state))
                }
            },
            
            SignalControl::Retry(state) => {
                if meta.ctr <= meta.max_ctr {
                    meta.ctr += 1;
                    ret_buf.push((meta, state))
                }
            },
            
            SignalControl::Drop(state) => {
                println!("dropping");
            },
            SignalControl::Fuck => {}
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
	listener: Listener<(Option<R>, S)>,
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
		T: Subscriber<(Option<R>, S)> + 'static
	{
		self.listener.add_sub(sub);
	}

	pub fn insert(&mut self, job: S,  timeout: std::time::Duration, fire_in: std::time::Duration, max_retry: usize) {
        let meta = CronMeta::new(timeout, fire_in, max_retry);
        self.schedule.insert(meta, job);
    }
    
    /// Processes the Events produced from jobs against the subscribers
    pub async fn process_events(&mut self) {

        let mut rescheduled_buf: Vec<(CronMeta, (Option<R>, S))> = Vec::new();

        self.listener.process(&mut rescheduled_buf).await;

        for (meta, (_response, state)) in rescheduled_buf.iter() {
            println!("redoing [{}]", meta.id);
            self.schedule.insert(meta.clone(), state.clone())
        }
    }

    /// Fires jobs that are ready
    pub async fn process_jobs(&mut self) -> Result<(), Error> {
        self.schedule.spawn_ready().await?;
        Ok(())
    }
}

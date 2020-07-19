/*
Public interface for schedules

*/

use super::{
    core::{CronMeta, Schedule, ScheduleControls},
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

struct Listener<R>
{
    channel: mpsc::Receiver<(CronMeta, R)>,
    subscribers: SmallVec<[Box<dyn Subscriber<R>>; 8]>,
    meta_subscribers: SmallVec<[Box<dyn MetaSubscriber>; 8]>
}


impl<R> Listener<R>
{
    pub fn new(channel: mpsc::Receiver<(CronMeta, R)>) -> Self {
        Self {
            channel,
            subscribers: SmallVec::new(),
            meta_subscribers: SmallVec::new()
        }
    }

    pub async fn process(&mut self) where R: std::fmt::Debug {
        process_subscribers(&mut self.channel, &mut self.subscribers[..], &mut self.meta_subscribers[..]).await;
    }


    pub fn add_sub<T>(&mut self, sub: T ) where T: Subscriber<R> + 'static {
        self.subscribers.push(Box::new(sub))
    }


    pub fn add_meta_sub<T>(&mut self, sub: T ) where T: MetaSubscriber + 'static {
        self.meta_subscribers.push(Box::new(sub))
    }
}

async fn process_subscribers<R>(
    channel: &mut mpsc::Receiver<(CronMeta, R)>,
    subs: &mut [Box<dyn Subscriber<R>>],
    meta_subs: &mut [Box<dyn MetaSubscriber>])
{
    if let Some((meta, data)) = channel.recv().await {
        for y in meta_subs.iter_mut() {
            if let Err(e) = y.handle(&meta).await {
                eprintln!("Error while handling [{:?}-{:?}] {}", x, y, e, );    
            }
        }
        
        for x in subs {
            if let Err(e) = x.handle(&meta, &data).await {
                eprintln!("Error while handling [{:?}] {}", x, e);
            }
        }
    }
}


pub struct ScheduledJobPool<J, R, S>
where
	J: CRON<Response = ScheduleControls<R>, State = S>,
	R: Send + Clone + Sync + 'static,
	S: Send + Clone + Sync,
{
	schedule: Schedule<J, R, S>,
	listener: Listener<(ScheduleControls<R>, S)>,
}


impl<J, R, S> ScheduledJobPool<J, R, S>
where
	J: CRON<Response = ScheduleControls<R>, State = S>,
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
		T: Subscriber<(ScheduleControls<R>, S)> + 'static
	{
		self.listener.add_sub(sub);
	}

	pub fn insert(&mut self, job: S,  timeout: std::time::Duration, fire_in: std::time::Duration, max_retry: usize) {
        let meta = CronMeta::new(timeout, fire_in, max_retry);
        self.schedule.insert(meta, job);
    }
    
    /// Processes the Events produced from jobs against the subscribers
    pub async fn process_events(&mut self) {
        self.listener.process().await;
    }

    /// Fires jobs that are ready
    pub async fn process_jobs(&mut self) -> Result<(), Error> {
        self.schedule.spawn_ready().await?;
        Ok(())
    }
}

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

pub type SubscribeBuf<R> = SmallVec<[Box<dyn Subscriber<R>>; 16]>;

#[async_trait::async_trait]
pub trait Subscriber<R> where Self: std::fmt::Display {
    async fn handle(&mut self, data: &R) -> Result<(), Error>;
}


struct Listener<R>
{
    channel: mpsc::Receiver<R>,
    subscribers: SubscribeBuf<R>
}


impl<R> Listener<R>
{
    pub fn new(channel: mpsc::Receiver<R>) -> Self {
        Self {
            channel,
            subscribers: SmallVec::new()
        }
    }

    pub async fn process(&mut self) where R: std::fmt::Debug {
        process_subscribers(&mut self.channel, &mut self.subscribers[..]).await;
    }


    pub fn add_sub<T>(&mut self, sub: T ) where T: Subscriber<R> + 'static {
        self.subscribers.push(Box::new(sub))
    }
}

async fn process_subscribers<R: std::fmt::Debug>(channel: &mut mpsc::Receiver<R>, subs: &mut [Box<dyn Subscriber<R>>])
{
    if let Some(data) = channel.recv().await {
        //println!("retrieved: {:?}", data);
        for x in subs {
            if let Err(e) = x.handle(&data).await {
                eprintln!("Error while processing subscriber ({}): {}", x, e);
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
	listener: Listener<(ScheduleControls<R>, CronMeta, S)>,
}


impl<J, R, S> ScheduledJobPool<J, R, S>
where
	J: CRON<Response = ScheduleControls<R>, State = S>,
	R: Send + Clone + Sync + 'static + std::fmt::Debug,
	S: Send + Clone + Sync + 'static + std::fmt::Debug
{
	pub fn new() -> Self {
		let (schedule, rx) = Schedule::new();
		Self {
			schedule: schedule,
			listener: Listener::new(rx)
		}
	}

	pub fn subscribe<T>(&mut self, sub: T)
	where 
		T: Subscriber<(ScheduleControls<R>, CronMeta, S)> + 'static
	{   
        //println!("adding sub");
		self.listener.add_sub(sub);
	}


	pub fn insert(&mut self, job: S,  timeout: std::time::Duration, fire_in: std::time::Duration, max_retry: usize) {
        //println!("adding job");
        let meta = CronMeta::new(timeout, fire_in, max_retry);
        self.schedule.insert(meta, job);
    }
    
    /// Processes the Events produced from jobs against the subscribers
    pub async fn process_events(&mut self) where R: std::fmt::Debug {
        self.listener.process().await;
        //println!("processing events");
    }

    /// Fires jobs that are ready
    pub async fn process_jobs(&mut self) -> Result<(), Error> {
        //println!("running jobs");
        self.schedule.spawn_ready().await?;
        Ok(())
    }
}
use super::{
    core::{Schedule, CRON, SignalControl},
    meta::CronMeta,
};

use tokio::sync::mpsc;
use smallvec::SmallVec;
use std::time::{Instant, Duration};

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


pub struct CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync,
	S: Send + Sync,
{
	schedule: Schedule<J, R, S>,
   
    channel: mpsc::Receiver<(CronMeta, SignalControl, Option<R>, S)>,
    subscribers: SmallVec<[Box<dyn Subscriber<R, S>>; 8]>,
    meta_subscribers: SmallVec<[Box<dyn MetaSubscriber>; 8]>,
    job_cnt: usize
}


impl<J, R, S> CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + 'static + std::fmt::Debug,
	S: Send + Sync + 'static + std::fmt::Debug
{   

	pub fn new(channel_size: usize) -> Self {
		let (schedule, rx) = Schedule::new(channel_size);
		Self {
			schedule: schedule,
            channel: rx,
            subscribers: SmallVec::new(),
            meta_subscribers: SmallVec::new(),
            job_cnt: 0
		}
    }
    
    #[inline]
    pub fn job_count(&self) -> usize {
        self.job_cnt
    }

	pub fn subscribe<T>(&mut self, sub: T)
	where 
		T: Subscriber<R, S> + 'static
	{
		self.subscribers.push(Box::new(sub));
    }
    
    pub fn subscribe_meta_handler<T>(&mut self, sub: T)
	where 
		T: MetaSubscriber + 'static
	{
        self.meta_subscribers.push(Box::new(sub));
	}

	pub fn insert(&mut self, job: S,  timeout: Duration, fire_in: Duration, max_retry: usize) {
        let meta = CronMeta::new(timeout, fire_in, J::name(), max_retry);
        self.schedule.insert(meta, job);
    }

    /// syntacially this function is called `process_reschedules` but it does do more
    /// It processes all the data the comes across the channel including reschedule
    pub async fn process_reschedules(&mut self, rescheduled_jobs: &mut Vec<(CronMeta, S)>) {
        
        if let Some((mut meta, mut ctrl, resp, mut state)) = self.channel.recv().await {
            for meta_hdlr in self.meta_subscribers.iter_mut() {
                if let Err(e) = meta_hdlr.handle(&mut meta, &mut ctrl).await {
                    eprintln!("Error while handling meta data [{:?}] {}", meta_hdlr, e);    
                }
            }
            
            for hdlr in self.subscribers.iter_mut() {
                if let Err(e) = hdlr.handle(&mut meta, &mut ctrl, &resp, &mut state).await {
                    eprintln!("Error while handling [{:?}] {}", hdlr, e);
                }
            }

            meta.ctr += 1;
            self.job_cnt -= 1;

            match ctrl {
                SignalControl::Reschedule(tts) => {
                    if meta.ctr <= meta.max_ctr {
                        meta.tts = tts;
                        self.schedule.insert(meta, state);
                    }
                },
                    
                SignalControl::Retry => {
                    if meta.ctr <= meta.max_ctr {
                        self.schedule.insert(meta, state);
                    }
                },

                SignalControl::RetryNow => {
                    if meta.ctr <= meta.max_ctr {
                        rescheduled_jobs.push((meta, state))
                    }
                },

                SignalControl::Drop | SignalControl::Success(_) | SignalControl::Fuck => {}
            }
        }
    }

    /// Fires jobs that are ready
    pub async fn release_ready(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) -> Result<(), Error> {
        self.schedule.release_ready(rescheduled_buf).await?;
        Ok(())
    }

    pub fn fire_jobs(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        self.job_cnt += 1;
        for (meta, state) in rescheduled_buf.drain(..) {
            spawn_worker::<J, R, S>(self.schedule.tx.clone(), meta, state)
        }
    }
}



fn spawn_worker<J, R, S>(
    mut vtx: mpsc::Sender<(CronMeta, SignalControl, Option<R>, S)>,
    mut meta: CronMeta,
    mut state: S)
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + 'static,
    S: Send + Sync + 'static
{
    tokio::spawn(async move {
        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Firing job {}", meta.id);
        let now = Instant::now();

        let (sig, resp) = match tokio::time::timeout(meta.ttl, J::exec(&mut state)).await {
            Ok(Ok((sig, resp))) => (sig, resp),

            Err(_) | Ok(Err(_)) => {
                if meta.max_ctr >= meta.ctr {
                    (SignalControl::Retry, None)
                }
                else {
                    (SignalControl::Drop, None)
                }
            }
        };

        meta.durations.push(now.elapsed());
        let id = meta.id;

        if let Err(e) = vtx.send((meta, sig, resp, state)).await {
            eprintln!("channel error: {}", e)
        }

        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Completed job {}", id);
    });
}

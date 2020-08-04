use super::{
    core::{Schedule, CRON, SignalControl},
    meta::CronMeta,
};

use tokio::{
    sync::mpsc,
    time::timeout
};

use smallvec::SmallVec;
use std::time::{Instant, Duration};

use crate::error::Error;

#[async_trait::async_trait]
pub trait Subscriber<R, S>: std::fmt::Debug {
    async fn handle(&mut self, 
        meta: &mut CronMeta,
        signal: &SignalControl,
        data: &Option<R>,
        state: &mut S
    ) -> Result<SignalControl, Error>;
}

#[async_trait::async_trait]
pub trait MetaSubscriber: std::fmt::Debug {
    async fn handle(&mut self, meta: &mut CronMeta, signal: &SignalControl) -> Result<SignalControl, Error>;
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

    async fn process_subscribers(&mut self, meta: &mut CronMeta, ctrl: SignalControl, state: &mut S, response: &Option<R>) -> SignalControl { 
        let mut nctrl = ctrl;

        for meta_hdlr in self.meta_subscribers.iter_mut() {
            match meta_hdlr.handle(meta, &nctrl).await {
                Ok(sig) => nctrl = sig,
                Err(e) => {
                    eprintln!("Error while handling meta data [{:?}] {}", meta_hdlr, e);    
                }
            }
        }
        
        for hdlr in self.subscribers.iter_mut() {
            match hdlr.handle(meta, &nctrl, response, state).await {
                Ok(sig) => nctrl = sig,
                Err(e) => {
                    eprintln!("Error while handling meta data [{:?}] {}", hdlr, e);    
                }
            }
        }

        nctrl
    }


    /// syntacially this function is called `process_reschedules` but it does do more
    /// It processes all the data the comes across the channel including reschedule
    pub async fn process_reschedules(&mut self) -> Option<(CronMeta, Option<R>, S)> {
        const RECV_TIMEOUT: f32 = 0.0025;

        match timeout(Duration::from_secs_f32(RECV_TIMEOUT), self.channel.recv()).await {
            Ok(Some((mut meta, mut ctrl, resp, mut state))) => {
                ctrl = self.process_subscribers(&mut meta, ctrl, &mut state, &resp).await;

                if meta.ctr > meta.max_ctr {
                    ctrl = SignalControl::Drop; 
                }
                
                self.job_cnt -= 1;
                meta.ctr += 1;
                
                match ctrl {
                    SignalControl::Reschedule(tts) => {
                        meta.tts = tts;
                        self.schedule.insert(meta, state);
                    },

                    SignalControl::Retry => self.schedule.insert(meta, state),

                    SignalControl::Drop | SignalControl::Success(_) => return Some((meta, resp, state))
                }
            
            }
            Ok(None) => eprintln!("Channel is shutting down - i think"),
            Err(e) => eprintln!("{}", e)
        }
        
        None
    }

    /// Fires jobs that are ready
    pub async fn release_ready(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) -> Result<(), Error> {
        self.schedule.release_ready(rescheduled_buf).await?;
        Ok(())
    }

    pub fn fire_jobs(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        for (meta, state) in rescheduled_buf.drain(..) {
            self.job_cnt += 1;
            spawn_worker::<J, R, S>(self.schedule.tx.clone(), meta, state, FailOpt::Retry)
        }
    }
}


pub enum FailOpt {
    // Uses reschedule
    Reschedule(Duration),
    
    // Uses Retry
    Retry,
}

fn spawn_worker<J, R, S>(
    mut vtx: mpsc::Sender<(CronMeta, SignalControl, Option<R>, S)>,
    mut meta: CronMeta,
    mut state: S, 
    opt: FailOpt)
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + 'static,
    S: Send + Sync + 'static
{
    tokio::spawn(async move {
        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Firing job {}", meta.id);
        let now = Instant::now();

        let (sig, resp) = match timeout(meta.ttl, J::exec(&mut state)).await {
            Ok(Ok((sig, resp))) => (sig, resp),

            Err(_) | Ok(Err(_)) => {
                let sig = match opt {
                        FailOpt::Reschedule(dur) => SignalControl::Reschedule(dur),
                        FailOpt::Retry => SignalControl::Retry,
                };
                (sig, None)
            }
        };
        meta.durations.push(now.elapsed());
        
        let id = meta.id;

        if let Err(e) = vtx.send((meta, sig, resp, state)).await {
            panic!("{}", e);    
        }

        tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Completed job {}", id);
    });
}

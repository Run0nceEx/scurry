use crate::schedule::{
    CRON, SignalControl,
    meta::CronMeta,
};

use tokio::{
    sync::mpsc,
    time::timeout
};

use smallvec::SmallVec;
use std::time::{Instant, Duration};

use crate::error::Error;

use tokio::{
    time::DelayQueue,
    stream::StreamExt,
};

use std::{
    collections::HashMap,
};

use std::sync::{Arc, Mutex};

use evc::OperationCache;



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



#[derive(Clone, Debug, Default)]
pub struct EVec<T>(pub Vec<T>);


impl<T> EVec<T> {
    pub fn with_compacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operation<T> {
    Push(T),
    Remove(usize),
    Clear,
}

impl<T> OperationCache for EVec<T> where T: Clone {
    type Operation = Operation<T>;

    fn apply_operation(&mut self, operation: Self::Operation) {
        match operation {
            Operation::Push(value) => self.0.push(value),
            Operation::Remove(index) => { self.0.remove(index); },
            Operation::Clear => self.0.clear(),
        }
    }
}


pub struct Schedule<J, R, S>
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + Clone,
    S: Send + Sync + Clone
{
    pub tx: Arc<Mutex<evc::WriteHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>>>,
    timer: DelayQueue<uuid::Uuid>,                 // timer for jobs
    pub bank: HashMap<uuid::Uuid, (CronMeta, S)>,      // collection of pending jobs

    _job: std::marker::PhantomData<J>
}


impl<J, R, S> Schedule<J, R, S> 
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + Clone,
    S: Send + Sync + Clone
{
    pub fn insert(&mut self, meta: CronMeta, state: S) {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.bank.insert(meta.id, (meta, state));
    }
    
    #[inline]
    pub fn new(channel_size: usize) -> (Self, evc::ReadHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>) {
        //let (tx, rx) = mpsc::channel(channel_size);
        let (tx, rx) = evc::new(EVec::with_compacity(channel_size));

        let schedule = Self {
            tx: Arc::new(Mutex::new(tx)),
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


pub struct CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + Clone,
	S: Send + Sync + Clone,
{
	pub schedule: Schedule<J, R, S>,
    channel: evc::ReadHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>,
    subscribers: SmallVec<[Box<dyn Subscriber<R, S>>; 8]>,
    meta_subscribers: SmallVec<[Box<dyn MetaSubscriber>; 8]>,
}


impl<J, R, S> CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + 'static + std::fmt::Debug + Clone,
	S: Send + Sync + 'static + std::fmt::Debug + Clone
{   

	pub fn new(channel_size: usize) -> Self {
		let (schedule, rx) = Schedule::new(channel_size);
		Self {
			schedule: schedule,
            channel: rx,
            subscribers: SmallVec::new(),
            meta_subscribers: SmallVec::new(),

		}
    }

    #[inline]
	pub fn subscribe<T>(&mut self, sub: T)
	where 
		T: Subscriber<R, S> + 'static
	{
		self.subscribers.push(Box::new(sub));
    }
    
    #[inline]
    pub fn subscribe_meta_handler<T>(&mut self, sub: T)
	where 
		T: MetaSubscriber + 'static
	{
        self.meta_subscribers.push(Box::new(sub));
	}

    #[inline]
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

    #[inline]
    pub fn flush(&mut self) {
        let mut lock = self.schedule.tx.lock().unwrap();
        lock.write(Operation::Clear);
        lock.refresh();
    }

    /// syntacially this function is called `process_reschedules` but it does do more
    /// It processes all the data the comes across the channel including reschedule
    pub async fn process_reschedules(&mut self, results: &mut Vec<(CronMeta, Option<R>, S)>) {
        self.flush();

        let preprocess_buf = self.channel.read().0.clone();

        for (mut meta, mut ctrl, resp, mut state) in preprocess_buf {
            ctrl = self.process_subscribers(&mut meta, ctrl, &mut state, &resp).await;

            if meta.ctr > meta.max_ctr {
                tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Killing Job {}", meta.id);
                ctrl = SignalControl::Drop; 
            }
            meta.ctr += 1;
            
            match ctrl {
                SignalControl::Reschedule(tts) => {
                    meta.tts = tts;
                    self.schedule.insert(meta, state);
                },
                SignalControl::Retry => self.schedule.insert(meta, state),
                SignalControl::Drop | SignalControl::Success(_) => results.push((meta, resp.clone(), state))
            }
        }
    }

    /// Get jobs that are ready
    pub async fn release_ready(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) -> Result<(), Error> {
        self.schedule.release_ready(rescheduled_buf).await?;
        Ok(())
    }
    
    pub fn fire_jobs(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        for (meta, state) in rescheduled_buf.drain(..) {
            spawn_worker::<J, R, S>(self.schedule.tx.clone(), meta, state)
        }
    }
}


fn spawn_worker<J, R, S>(
    mut vtx: Arc<Mutex<evc::WriteHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>>>,
    mut meta: CronMeta,
    mut state: S, 
)
where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + 'static + Clone,
    S: Send + Sync + 'static + Clone
{
    tokio::spawn(async move {
        tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Firing job {}", meta.id);
        let now = Instant::now();

        let (sig, resp) = match timeout(meta.ttl, J::exec(&mut state)).await {
            Ok(Ok((sig, resp))) => (sig, resp),

            Err(_) | Ok(Err(_)) => {
                tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "job timed-out {}", meta.id);
                (SignalControl::Retry, None)
            }
        };
        meta.durations.push(now.elapsed());
        
        let id = meta.id;

        let mut lock = vtx.lock().unwrap();
        lock.write(Operation::Push((meta, sig ,resp, state)));

        tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Completed job {}", id);
    });
}

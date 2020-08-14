use super::{
    CRON, SignalControl,
    meta::CronMeta,
};

use tokio::{
    time::timeout,
    stream::{StreamExt, Stream}
};

use evc::OperationCache;

use std::sync::{Arc, Mutex};
use std::task::{Poll, Context};
use std::pin::Pin;

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

    #[allow(dead_code)]
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




pub struct CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + Clone,
    S: Send + Sync + Clone,
{
    tx: Arc<Mutex<evc::WriteHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>>>,
    rx: evc::ReadHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>,
    throttle: usize, // max amount of job_count()


    _job: std::marker::PhantomData<J>,
}

impl<J, R, S> CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + 'static + std::fmt::Debug + Clone,
	S: Send + Sync + 'static + std::fmt::Debug + Clone
{
	pub fn new(channel_size: usize, throttle: usize) -> Self {		
        let (tx, rx) = evc::new(EVec::with_compacity(channel_size));
        
        Self {
            throttle,
            rx,
            tx: Arc::new(Mutex::new(tx)),
            _job: std::marker::PhantomData
		}
    }

    #[inline]
    pub fn job_count(&self) -> usize {
        std::sync::Arc::strong_count(&self.tx)
    }
    
    pub fn fire_jobs(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        let top = {
            if self.throttle > 0 {
                if self.job_count() < self.throttle {
                    let mut i = self.throttle-self.job_count();
                    
                    if i >= rescheduled_buf.len() {
                        i = rescheduled_buf.len()
                    } 
                    //tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Firing {} jobs", i);
                    
                    i
                }
                else {
                    0
                }
            }

            else {
                rescheduled_buf.len()
            }
        };

        for (meta, state) in rescheduled_buf.drain(..top) {
            spawn_worker::<J, R, S>(self.tx.clone(), meta, state)
        }
    }
}


impl<J, R, S> Stream for CronPool<J, R, S> 
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + Clone + 'static,
    S: Send + Sync + Clone + 'static
{
    type Item = Vec<(CronMeta, SignalControl, Option<R>, S)>;
    
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut lock = self.tx.lock().unwrap();
        lock.refresh();

        let jobs = &self.rx.read().0;
        if jobs.len() > 0 {
            lock.write(Operation::Clear);
            return Poll::Ready(Some(jobs.clone()));
        }
        else {
            return Poll::Ready(None);
        }
    }
}


fn spawn_worker<J, R, S>(
    vtx: Arc<Mutex<evc::WriteHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>>>,
    mut meta: CronMeta,
    mut state: S, 
) where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + 'static + Clone,
    S: Send + Sync + 'static + Clone
{
    tokio::spawn(async move {
        loop {
            tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Firing job {}", meta.id);
            let now = std::time::Instant::now();

            let (mut sig, resp) = match timeout(meta.ttl, J::exec(&mut state)).await {
                Ok(Ok((sig, resp))) => (sig, resp),

                Err(_) | Ok(Err(_)) => {
                    tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "job timed-out {}", meta.id);
                    (SignalControl::Retry, None)
                }
            };

            //meta.record_elapsed(now);
            
            if meta.ctr >= meta.max_ctr {
                sig = SignalControl::Drop; 
            }

            meta.ctr += 1;

            match sig {
                SignalControl::Retry => {},
                _ => {
                    tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Completed job {}", meta.id);
                    let mut lock = vtx.lock().unwrap();
                    lock.write(Operation::Push((meta, sig ,resp, state)));
                    break;
                }, 
            }
        }
    });
}

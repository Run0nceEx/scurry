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
    
    ret_tx: Arc<Mutex<evc::WriteHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>>>,
    ret_rx: evc::ReadHandle<EVec<(CronMeta, SignalControl, Option<R>, S)>>,
    _job: std::marker::PhantomData<J>
}


impl<J, R, S> CronPool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + 'static + std::fmt::Debug + Clone,
	S: Send + Sync + 'static + std::fmt::Debug + Clone
{   

	pub fn new(channel_size: usize) -> Self {
		
        let (ret_tx, ret_rx) = evc::new(EVec::with_compacity(channel_size));
        
        Self {
            ret_rx,
            ret_tx: Arc::new(Mutex::new(ret_tx)),
            _job: std::marker::PhantomData
		}
    }

    #[inline]
    pub fn job_count(&self) -> usize {
        std::sync::Arc::strong_count(&self.ret_tx)
    }
    
    pub fn fire_jobs(&mut self, rescheduled_buf: &mut Vec<(CronMeta, S)>) {
        for (meta, state) in rescheduled_buf.drain(..) {
            spawn_worker::<J, R, S>(self.ret_tx.clone(), meta, state)
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
        let mut lock = self.ret_tx.lock().unwrap();
        lock.refresh();
        drop(lock);
        
        let mut jobs = {
          let lock = &self.ret_rx.read().0;
          if lock.len() == 0 {
            return Poll::Ready(None);
          }
          lock.clone()
        };
        
        let mut lock = self.ret_tx.lock().unwrap();
        lock.write(Operation::Clear);
        drop(lock);
        
        // I'd really like to have a reference it return a referenced slice from `ret_rx`
        // but then we can't filter the results
        // if anyone comes up for a solution to this
        // please merge
        let item = jobs.drain(..).filter_map(|(mut meta, mut ctrl, resp, state)| {
            if meta.ctr > meta.max_ctr {
                tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Killing Job {}", meta.id);
                ctrl = SignalControl::Drop; 
            }

            meta.ctr += 1;
            eprintln!("{}", meta.ctr);
            
            match ctrl {
                SignalControl::Retry => {
                    spawn_worker::<J, R, S>(self.ret_tx.clone(), meta, state);
                    None
                },

                SignalControl::Drop | SignalControl::Success(_) => Some((meta, ctrl, resp, state))
            }
        }).collect::<Vec<_>>();

        if item.len() > 0 {
            return Poll::Ready(Some(item));    
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
        tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Firing job {}", meta.id);
        let now = std::time::Instant::now();

        let (sig, resp) = match timeout(meta.ttl, J::exec(&mut state)).await {
            Ok(Ok((sig, resp))) => (sig, resp),

            Err(_) | Ok(Err(_)) => {
                tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "job timed-out {}", meta.id);
                (SignalControl::Retry, None)
            }
        };

        meta.record_elapsed(now);
        
        tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Completed job {}", meta.id);
        
        let mut lock = vtx.lock().unwrap();
        lock.write(Operation::Push((meta, sig ,resp, state)));
    });
}

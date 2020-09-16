use super::CRON;

use tokio::{
    time::timeout,
    stream::Stream
};

use evc::OperationCache;
use crate::libcore::model::State as NetState;

use std::{
    sync::{Arc, Mutex},
    task::{Poll, Context},
    pin::Pin,
    net::SocketAddr
};


use crate::libcore::util::Boundary;
use crate::cli::input::combine::Feeder;

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

#[derive(Debug, Clone)]
pub enum JobErr {
    IO(std::io::ErrorKind),
    Errno(i32),
    TaskFailed,
    Other
}


#[derive(Debug, Clone)]
pub enum JobCtrl<R> {
    Return(NetState, R),
    Error(JobErr),
}

pub struct Worker<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + Clone,
    S: Send + Sync + Clone,
{
    tx: Arc<Mutex<evc::WriteHandle<EVec<(JobCtrl<R>, S)>>>>,
    rx: evc::ReadHandle<EVec<(JobCtrl<R>, S)>>,

    ttl: std::time::Duration, // time to live for each job
    
    throttle: Boundary, // max amount of job_count()
    _job: std::marker::PhantomData<J>,
}

impl<J, R, S> Worker<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + 'static + std::fmt::Debug + Clone,
	S: Send + Sync + 'static + std::fmt::Debug + Clone
{
	pub fn new(throttle: Boundary, ttl: std::time::Duration) -> Self {
        const ALLOC_SIZE: usize = 16384;

        let compacity = {
            if let Boundary::Limited(n) = throttle {
                if n > ALLOC_SIZE { n+1 }
                else { ALLOC_SIZE }
            }
            else { ALLOC_SIZE }
        };

        let (tx, rx) = evc::new(EVec::with_compacity(compacity));

        let instance = Self {
            throttle,
            rx,
            ttl,
            tx: Arc::new(Mutex::new(tx)),
            _job: std::marker::PhantomData
        };
        
        instance
    }


    fn calc_new_spawns(&self, buf_len: usize) -> usize {
        let limit = match self.throttle {
            Boundary::Limited(limit) => limit, 
            Boundary::Unlimited => return buf_len
        };

        if limit >= self.job_count() {
            let mut spawn_count = limit-self.job_count();
            if spawn_count > buf_len {
                spawn_count = buf_len
            }

            return spawn_count
        }

        return 0 
    }

    #[inline]
    pub fn job_count(&self) -> usize {
        std::sync::Arc::strong_count(&self.tx)
    }
    
    #[inline]
    pub fn throttled(&self) -> Boundary {
        self.throttle
    }

    
    pub fn fire_jobs(&mut self, buf: &mut Vec<S>) -> usize {
        let mut amount = 0;
        for state in buf.drain(..) {
            amount += 1;
            spawn_worker::<J, R, S>(self.tx.clone(), state, self.ttl)
        }
        amount
    }

    pub fn throttle_fire(&mut self, buf: &mut Vec<S>) -> usize 
    {
        let new_spawn_count = self.calc_new_spawns(buf.len());

        if new_spawn_count > 0 {
            for state in buf.drain(..new_spawn_count) {
                spawn_worker::<J, R, S>(self.tx.clone(), state, self.ttl)
            }
        }

        new_spawn_count
    }

    pub fn throttle_feed_fire<'a>(&mut self, buf: &mut Vec<S>, feed: &mut Feeder<'a>) -> usize
    where S: From<SocketAddr>
    {
        let mut sock_buf = Vec::new();
        let amount = feed.generate_chunk(&mut sock_buf, self.calc_new_spawns(buf.len()));

        buf.extend(sock_buf.drain(..).map(|x| x.into()));
        
        self.throttle_fire(buf)
    }

    pub fn flush(&mut self) -> Vec<(JobCtrl<R>, S)> {
        let mut lock = self.tx.lock().unwrap();
        let mut buffer = self.rx.read().0.clone();
        lock.refresh();
        buffer.extend(self.rx.read().0.clone());
        lock.write(Operation::Clear);
        lock.refresh();
        buffer
    }
}


impl<J, R, S> Stream for Worker<J, R, S> 
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + Clone + 'static,
    S: Send + Sync + Clone + 'static
{
    type Item = Vec<(JobCtrl<R>, S)>;
    
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut lock = self.tx.lock().unwrap();
        lock.refresh();
        let jobs = &self.rx.read().0;
        if jobs.len() > 0 {
            // this is safe because operations
            // aren't reflected until `lock.refresh()`
            // is executed
            lock.write(Operation::Clear);
            return Poll::Ready(Some(jobs.clone()));
        }
        else {
            return Poll::Ready(None);
        }
    }
}


fn spawn_worker<J, R, S>(
    vtx: Arc<Mutex<evc::WriteHandle<EVec<(JobCtrl<R>, S)>>>>,
    mut state: S,
    ttl: std::time::Duration
) where 
    J: CRON<Response=R, State=S>,
    R: Send + Sync + 'static + Clone,
    S: Send + Sync + 'static + Clone
{
    tokio::spawn(async move {        
        let sig = match timeout(ttl, J::exec(&mut state)).await {
            Ok(Ok(sig)) => sig,
            Err(_) => JobCtrl::Error(JobErr::IO(std::io::ErrorKind::TimedOut)),
            
            Ok(Err(err)) => {
                eprintln!("task failed: {}", err);
                JobCtrl::Error(JobErr::TaskFailed)
            }
        };

        let mut lock = vtx.lock().unwrap();
        lock.write(Operation::Push((sig, state)));
    });
}

use crate::libcore::error::Error;
use super::{
    meta::CronMeta,
    stash::Stash,
    pool::CronPool
};
use tokio::stream::StreamExt;
use smallvec::SmallVec;

/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SignalControl {
    /// Drop memory, and give a boolean to tell if we connected 
    Success(bool), // Boolean to signify to the scheduler if we connected to the target or not
    
    /// Operations failed and would like to attemp again, 
    /// it will sleep again for whatever it's time to sleep paramenter was set to. (tts)
    Retry,

    /// Operation was nullified either because of no result, or unreported error
    Drop,

    Stash(std::time::Duration)
}


pub struct Pool<J, R, S>
where
	J: CRON<Response = R, State = S>,
	R: Send + Sync + Clone + std::fmt::Debug,
    S: Send + Sync + Clone + std::fmt::Debug,

{
    pub pool: CronPool<J, R, S>,
    timer: std::time::Instant,
    queued: Vec<(CronMeta, S)>,
    stash: Stash<S>,

    work_buf: Vec<(CronMeta, SignalControl, Option<R>, S)>
}

impl<J, R, S> Pool<J, R, S> 
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,
{   
    #[inline]
    pub fn new(pool: CronPool<J, R, S>) -> Self {
        Self {
            pool,
            timer: std::time::Instant::now(),
            queued: Vec::new(),
            stash: Stash::new(),

            work_buf: Vec::new()
        }
    }

    pub async fn tick<'a>(&'a mut self) -> &'a [(CronMeta, SignalControl, Option<R>, S)] {
        self.work_buf.clear();
        self.stash.release(&mut self.queued).await;
        
		if self.queued.len() > 0 {
			self.pool.fire_jobs(&mut self.queued);
		}

		if self.timer.elapsed() >= std::time::Duration::from_secs(5) {			
			tracing::event!(
				target: "Pool", tracing::Level::DEBUG, "Job count is [{}/{}] jobs",
				self.pool.job_count(), self.queued.len(),
			);

			self.timer = std::time::Instant::now();
        }

        while let Some(mut chunk) = self.pool.next().await {
            let processed_chunk: SmallVec<[(CronMeta, SignalControl, Option<R>, S); 64]> = chunk.drain(..)
                .filter_map(|(meta, ctrl, resp, state)| {
                    match ctrl {
                        SignalControl::Stash(duration) => {
                            self.stash.insert(meta, state, &duration);
                            None
                        },
                        _ => Some((meta, ctrl, resp, state))
                    }
                })
                .collect();
            
            self.work_buf.extend(processed_chunk);
        }

        &self.work_buf
    }

    #[inline]
    pub fn is_working(&self) -> bool {
        self.pool.job_count()-1 > 0 && self.stash.amount() > 0 
    }

    #[inline]
    pub fn mut_buffer<'a>(&'a mut self) -> &'a mut Vec<(CronMeta, S)> {
        &mut self.queued
    }
    
    #[inline]
    pub fn buffer<'a>(&'a self) -> &'a Vec<(CronMeta, S)> {
        &self.queued
    }
}


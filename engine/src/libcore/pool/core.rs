use crate::libcore::error::Error;
use super::{
    meta::CronMeta,
    stash::Stash,
    pool::CronPool
};
use tokio::stream::{StreamExt, Stream};
use smallvec::SmallVec;
use super::sig::SignalControl;

/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<SignalControl<Self::Response>, Error>;
}

pub type WorkBuf<R, S> = SmallVec<[(CronMeta, SignalControl<R>, S); 64]>;

pub struct Pool<J, R, S>
where
    J: CRON<Response = R, State = S>,
    
    // Stream<Item=&WorkBuf<R, S>>
    
    R: Send + Sync + Clone + std::fmt::Debug,
    S: Send + Sync + Clone + std::fmt::Debug,

{
    pub pool: CronPool<J, R, S>,
    timer: std::time::Instant,
    queued: Vec<(CronMeta, S)>,
    stash: Stash<S>,

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
        }
    }

    pub async fn tick(&mut self) -> WorkBuf<R, S> {
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

        let mut ret_buf: WorkBuf<R, S> = SmallVec::new();

        while let Some(mut chunk) = self.pool.next().await {
            ret_buf.extend(chunk.drain(..)
                .filter_map(|(meta, ctrl, state)| {
                    match ctrl {
                        SignalControl::Stash(duration) => {
                            self.stash.insert(meta, state, &duration);
                            None
                        },
                        _ => Some((meta, ctrl, state))
                    }
                })
            );
        }

        ret_buf
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


use crate::libcore::error::Error;
use super::{
    stash::Stash,
    worker::{Worker, JobCtrl, JobErr},
};

use tokio::stream::StreamExt;
use std::fmt::Debug;


/// Used in scheduler (Command run on)
#[async_trait::async_trait]
pub trait CRON: std::fmt::Debug {
    type State;
    type Response;

    /// Run function, and then append to parent if more jobs are needed
    async fn exec(state: &mut Self::State) -> Result<JobCtrl<Self::Response>, Error>;
}

pub struct Pool<J, R, S>
where
    J: CRON<Response = R, State = S>,
    R: Send + Sync + Clone + Debug,
    S: Send + Sync + Clone + Debug,

{
    pub pool: Worker<J, R, S>,
    timer: std::time::Instant,
    stash: Stash<S>,

}

impl<J, R, S> Pool<J, R, S> 
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,
{   
    #[inline]
    pub fn new(pool: Worker<J, R, S>) -> Self {
        Self {
            pool,
            timer: std::time::Instant::now(),
            stash: Stash::new(),
        }
    }

    pub async fn tick(&mut self, queued: &mut Vec<S>) -> Vec<(JobCtrl<R>, S)> {
        const RESCHEDULE: u64 = 5;

        self.stash.release(queued).await;
        println!("HALLO 2 | queued-len {}", queued.len());
        
        if queued.len() > 0 {
            self.pool.fire_jobs(queued);
		}

		if self.timer.elapsed() >= std::time::Duration::from_secs(5) {			
			tracing::event!(
				target: "Pool", tracing::Level::DEBUG, "Job count is [{}/{}] jobs",
				self.pool.job_count(), queued.len(),
			);

			self.timer = std::time::Instant::now();
        }

        let mut ret_buf = Vec::new();
        

        while let Some(mut chunk) = self.pool.next().await {
            println!("CHUNK: {:?}", chunk);
            
            ret_buf.extend(chunk.drain(..)
                // if resource (nic) is blocked,
                // stash and remove from results
                // so the entries may be retried
                .filter_map(|(ctrl, state)| {
                    match ctrl {
                        JobCtrl::Error(io_error) => {
                            let stash_flag = match io_error {
                                JobErr::Errno(i) => is_resource_blocked(i),
                                
                                JobErr::IO(_) 
                                | JobErr::Other 
                                | JobErr::TaskFailed => false,
                            };
                            if stash_flag {
                                self.stash.insert(
                                    state, 
                                    &std::time::Duration::from_secs(RESCHEDULE)
                                );
                                None
                            }
                            else {
                                Some((JobCtrl::Error(io_error), state))
                            }
                        },
                        _ => Some((ctrl, state))
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
}

fn is_resource_blocked(errno: i32) -> bool {
    match errno {
        101         // Network unreachable
        | 113       // no route to host
        | 92        // failed to bind to interface/protocol
        | 24        // too many file-discriptors open
        => true,
        _ => false
    }
}
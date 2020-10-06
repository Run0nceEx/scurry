use super::{
    stash::Stash,
    worker::{Worker, JobCtrl, JobErr},
};

use tokio::stream::StreamExt;
use std::fmt::Debug;
use super::CRON;
use crate::cli::input::combine::Feeder;


pub struct Pool<J, R, S>
where
    J: CRON<Response = R, State = S>,
    R: Send + Sync + Clone + Debug,
    S: Send + Sync + Clone + Debug,

{
    pub pool: Worker<J, R, S>,
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
            stash: Stash::new(),
        }
    }

    #[inline]
    pub fn is_working(&self) -> bool {
        self.pool.job_count() > 1 && self.stash.amount() > 0  
    }

    #[inline]
    pub fn job_count(&self) -> usize {
        self.pool.job_count()
    }

    #[inline]
    pub fn flush_stash(&mut self, buf: &mut Vec<S>) -> usize {
        self.stash.flush(buf)
    }

    #[inline]
    pub fn flush_channel(&mut self) -> Vec<(JobCtrl<R>, S)> {
        self.pool.flush()
    }

    pub async fn fire_from_feeder<'a>(&mut self, queued: &mut Vec<S>, feed: &mut Feeder<'a>) -> usize
    where S: From<std::net::SocketAddr>
    {

        let mut sock_buf = Vec::with_capacity(4001);
        feed.generate_chunk(&mut sock_buf, 4000);
        
        queued.extend(sock_buf.drain(..).map(|x| x.into()));

        let alloc_amt = self.pool.calc_new_spawns(queued.len());
        
        if alloc_amt > 0 {
            let release_amt = self.stash.release(queued).await;       
            let feed_amt;

            if release_amt >= alloc_amt {
                feed_amt = 0
            }

            else {
                feed_amt = alloc_amt - release_amt
            }
            
            if feed_amt > 0 && !feed.is_done() {
                feed.generate_chunk(&mut sock_buf, feed_amt);
                queued.extend(sock_buf.drain(..).map(|x| x.into()));
            }
            
            return self.pool.spawn(queued)
        }
        0
    }

    pub async fn tick(&mut self, queued: &mut Vec<S>) -> Vec<(JobCtrl<R>, S)> {
        const RESCHEDULE: u64 = 5;

        self.stash.release(queued).await;
        
        if queued.len() > 0 {
            self.pool.spawn(queued);
		}

        let mut ret_buf = Vec::new();
        
        while let Some(mut chunk) = self.pool.next().await {
            ret_buf.extend(chunk.drain(..)
                // if resource (nic) is blocked,
                // stash and remove from results
                // so the entries may be retried
                .filter_map(|(ctrl, state)| {
                    if should_stash(&ctrl) {
                        self.stash.insert(
                            state, 
                            &std::time::Duration::from_secs(RESCHEDULE)
                        );
                        None
                    }
                    else { Some((ctrl, state)) }
                }));
        }

        ret_buf
    }

}

fn should_stash<T>(ctrl: &JobCtrl<T>) -> bool {
    match ctrl {
        JobCtrl::Error(JobErr::Errno(i)) => is_resource_blocked(*i),
        _ => false
    }
}


fn is_resource_blocked(errno: i32) -> bool {
    match errno {
        101         // Network unreachable
        // | 113       // no route to host
        | 92        // failed to bind to interface/protocol
        | 24        // too many file-discriptors open
        => true,
        _ => false
    }
}

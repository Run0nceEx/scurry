/*
Mocked up pools for testing and benching.

Most of this stuff is repetative fluff
*/
extern crate test;
use test::Bencher;

use tokio::runtime::Runtime;

use super::{
    Worker,
    JobCtrl,
    core::CRON,
};

use crate::libcore::{
    error::Error,
    model::State as NetState
};

use tokio::stream::StreamExt;
use std::time::Duration;

pub const JOB_CNT: usize = 100;
pub const POOLSIZE: usize = 16_384;

#[cfg(test)]
pub mod noop {
    use super::*;

    pub type NoOpPool<S, R> = Worker<Handler<S, R>, R, S>;
    pub type Pool = NoOpPool<State, Response>;

    impl<S, R> NoOpPool<S, R>
    where 
        S: Send + Sync + Clone + Default + std::fmt::Debug + 'static,
        R: Send + Sync + Clone + Default + std::fmt::Debug + 'static
    {
        pub fn default_test() -> Self {
            Worker::new(POOLSIZE, 0, std::time::Duration::from_secs(5))
        }
    }

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct State;

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct Response;

    #[derive(Debug)]
    pub struct Handler<S, R> {
        _state:     std::marker::PhantomData<S>,
        _response:  std::marker::PhantomData<R>,
    }

    impl<S, R> Handler<S, R> {
        pub fn new() -> Self {
            Self {
                _state:     std::marker::PhantomData,
                _response:  std::marker::PhantomData,
            }
        }
    }

    impl<S, R> Default for Handler<S, R> {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait::async_trait]
    impl<S, R> CRON for Handler<S, R> 
    where
        S: Send + Sync + Default + std::fmt::Debug,
        R: Send + Sync + Default + std::fmt::Debug
    {
        type State = S;
        type Response = R;

        /// Run function, and then append to parent if more jobs are needed
        async fn exec(_state: &mut Self::State) -> Result<JobCtrl<Self::Response>, Error> {
            Ok(JobCtrl::Return(NetState::Closed, R::default()))
        }
    }

}

#[test]
fn pool_single_in_single_out() {
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    buf.push(
        noop::State
    );

    rt.block_on(async move {
        let mut pool = noop::Pool::default_test();
        pool.fire_jobs(&mut buf);
        
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
        let rbuf = pool.next().await.unwrap();
        assert_eq!(rbuf.len(), 1);
    });
}



#[test]
fn pool_job_count_accurate() {
    let mut rt = Runtime::new().unwrap();

    let mut buf = vec![noop::State; 1];

    rt.block_on(async move {
        let mut pool = noop::Pool::default_test();
        
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), 2);
        
        tokio::time::delay_for(std::time::Duration::from_secs(2)).await;
        
        assert_eq!(pool.next().await.unwrap().len(), 1);
        assert_eq!(pool.job_count(), 1);
    });
}


#[test]
fn all_in_all_out() {
    let mut rt = Runtime::new().unwrap();
    
    let mut buf = vec![noop::State; JOB_CNT];
    
    rt.block_on(async move {
        let mut pool = noop::Pool::default_test();

        assert_eq!(buf.len(), JOB_CNT);
        pool.fire_jobs(&mut buf);
        
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
        
        assert_eq!(JOB_CNT, pool.next().await.unwrap().len());
    });
}

//assert all tasks do eventually timeout
#[test]
fn all_timeout() {
    use noop as mock;

    const DELAY_SECS: u64 = 3;

    #[derive(Debug)]
    struct Handler;

    #[async_trait::async_trait]
    impl CRON for Handler {
        type State = mock::State;
        type Response = mock::Response;

        async fn exec(_state: &mut Self::State) -> Result<JobCtrl<Self::Response>, Error> {
            tokio::time::delay_for(Duration::from_secs(DELAY_SECS)).await;
            Ok(JobCtrl::Return(NetState::Closed, mock::Response))
        }
    }
    
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();
    
    let mut pool: Worker<Handler, mock::Response, mock::State> = Worker::new(
        POOLSIZE,
        0, 
        std::time::Duration::from_secs(DELAY_SECS-1)
    );

    for _ in 0..JOB_CNT {
        buf.push(
            noop::State
        );
    }

    rt.block_on(async move {
        assert_eq!(buf.len(), JOB_CNT);
        // Fire and retrieve once
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), JOB_CNT+1);
        assert_eq!(buf.len(), 0);
        
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
        
        while let Some(data) = pool.next().await {
            for (sig, _state) in data {
                match sig {
                    JobCtrl::Return(_sig, _resp) => assert_eq!(1, 0),
                    _ => {}
                }
            }
        }
        
        assert_eq!(pool.job_count(), 1);
       
    });
}



#[bench]
fn evpool_poll_noop_bench(b: &mut Bencher) {
    let mut rt = Runtime::new().unwrap();
    let mut buf = vec![noop::State; JOB_CNT];

    let mut pool = noop::Pool::new(POOLSIZE, 0, std::time::Duration::from_secs(5));
    rt.block_on(async {
        pool.fire_jobs(&mut buf)
    });

    b.iter(|| rt.block_on(pool.next()));
}

#[bench]
fn evpool_poll_addition_bench(b: &mut Bencher) {
    #[derive(Debug, Default, Clone)]
    pub struct State {
        a: u16,
        b: u16
    };

    #[derive(Debug, Default)]
    pub struct Handler {
        count: usize,
    }

    #[async_trait::async_trait]
    impl CRON for Handler 
    {
        type State = State;
        type Response = usize;

        /// Run function, and then append to parent if more jobs are needed
        async fn exec(state: &mut Self::State) -> Result<JobCtrl<Self::Response>, Error> {
            Ok(JobCtrl::Return(NetState::Closed, (state.a as u32 + state.b as u32) as usize))
        }
    }

    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();
    
    for _ in 0..JOB_CNT {
        buf.push(State {
            a: rand::random(),
            b: rand::random()
        });
    }

    let mut pool: Worker<Handler, usize, State> = Worker::new(POOLSIZE, 0, std::time::Duration::from_secs(5));
    
    rt.block_on(async {    
        pool.fire_jobs(&mut buf);
    });

    b.iter(|| rt.block_on(pool.next()));
}
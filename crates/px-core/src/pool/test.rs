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
    CRON,
};

use crate::{
    error::Error,
    model::State as NetState,
    util::Boundary
};

use tokio_stream::StreamExt;
use std::time::Duration;

pub const JOB_CNT: usize = 100;

#[cfg(test)]
pub mod noop {
    use super::*;

    pub type NoOpPool<S, R> = Worker<Handler<S, R>, R, S>;
    pub type NopWorker = NoOpPool<State, Response>;

    impl<S, R> NoOpPool<S, R>
    where 
        S: Send + Sync + Clone + Default + std::fmt::Debug + 'static,
        R: Send + Sync + Clone + Default + std::fmt::Debug + 'static
    {
        pub fn default_test() -> Self {
            Worker::new(Boundary::Unlimited, std::time::Duration::from_secs(5))
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
fn worker_single_in_single_out() {
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    buf.push(
        noop::State
    );

    rt.block_on(async move {
        let mut pool = noop::NopWorker::default_test();
        pool.spawn(&mut buf);
        
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let rbuf = pool.next().await.unwrap();
        assert_eq!(rbuf.len(), 1);
    });
}



#[test]
fn worker_job_count_accurate() {
    let mut rt = Runtime::new().unwrap();

    let mut buf = vec![noop::State; 1];

    rt.block_on(async move {
        let mut pool = noop::NopWorker::default_test();
        
        pool.spawn(&mut buf);
        assert_eq!(pool.job_count(), 2);
        
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        assert_eq!(pool.next().await.unwrap().len(), 1);
        assert_eq!(pool.job_count(), 1);
    });
}


#[test]
fn worker_all_in_all_out() {
    let mut rt = Runtime::new().unwrap();
    
    let mut buf = vec![noop::State; JOB_CNT];
    
    rt.block_on(async move {
        let mut pool = noop::NopWorker::default_test();

        assert_eq!(buf.len(), JOB_CNT);
        pool.spawn(&mut buf);
        
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
        assert_eq!(JOB_CNT, pool.next().await.unwrap().len());
    });
}

//assert all tasks do eventually timeout
#[test]
fn worker_all_timeout() {
    use noop as mock;

    const DELAY_SECS: u64 = 3;

    #[derive(Debug)]
    struct Handler;

    #[async_trait::async_trait]
    impl CRON for Handler {
        type State = mock::State;
        type Response = mock::Response;

        async fn exec(_state: &mut Self::State) -> Result<JobCtrl<Self::Response>, Error> {
            tokio::time::sleep(Duration::from_secs(DELAY_SECS)).await;
            Ok(JobCtrl::Return(NetState::Closed, mock::Response))
        }
    }
    
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();
    
    let mut pool: Worker<Handler, mock::Response, mock::State> = Worker::new(
        Boundary::Unlimited,
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
        pool.spawn(&mut buf);
        assert_eq!(pool.job_count(), JOB_CNT+1);
        assert_eq!(buf.len(), 0);
        
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        
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


use crate::pool::Pool;

#[test]
fn pool_all_in_all_out() {
    let mut rt = Runtime::new().unwrap();
    let worker: noop::NopWorker = Worker::new(Boundary::Unlimited, std::time::Duration::from_secs(5));
    let mut buf = vec![noop::State; 3];
    let mut pool = Pool::new(worker);


    let results = rt.block_on(async move {
        for i in 0..2 {
            let res = pool.tick(&mut buf).await;
            
            if i == 0 {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            else if i == 1 {
                return res
            }
            
        }

        unreachable!()

    });

    assert_eq!(results.len(), 3);
}


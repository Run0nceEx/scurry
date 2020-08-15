/*
Mocked up pools for testing and benching.

Most of this stuff is repetative fluff
*/
extern crate test;
use test::Bencher;

use tokio::runtime::Runtime;

use super::{
    CronPool,
    SignalControl,
    CRON, meta::CronMeta
};
use crate::error::Error;

use tokio::stream::{Stream, StreamExt};
use std::time::Duration;

pub const JOB_CNT: usize = 100;
pub const POOLSIZE: usize = 16_384;

#[cfg(test)]
pub mod noop {
    use super::*;

    pub type NoOpPool<S, R> = CronPool<Worker<S, R>, R, S>;
    pub type Pool = NoOpPool<State, Response>;


    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct State;

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct Response;

    #[derive(Debug)]
    pub struct Worker<S, R> {
        _state : std::marker::PhantomData<S>,
        _response : std::marker::PhantomData<R>,
    }

    impl<S, R> Worker<S, R> {
        pub fn new() -> Self {
            Self {
                _state: std::marker::PhantomData,
                _response: std::marker::PhantomData,
            }
        }
    }

    impl<S, R> Default for Worker<S, R> {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait::async_trait]
    impl<S, R> CRON for Worker<S, R> 
    where
        S: Send + Sync + Default + std::fmt::Debug,
        R: Send + Sync + Default + std::fmt::Debug
    {
        type State = S;
        type Response = R;

        /// Run function, and then append to parent if more jobs are needed
        async fn exec(_state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {
            Ok((SignalControl::Success(false), Some(R::default())))
        }
    }

    // pub fn get_pool(timeout: f32, fire_in: f32, max_retries: usize) -> Pool {
    //     let mut pool: Pool = Pool::new(POOLSIZE);
    //     let buf = 
    //     for _ in 0..JOB_CNT {
    //         pool.insert(State, Duration::from_secs_f32(timeout), Duration::from_secs_f32(fire_in), max_retries);
    //     }

    //     pool
    // }
}


#[bench]
fn pool_poll_noop_bench(b: &mut Bencher) {

    let mut rt = Runtime::new().unwrap();
    let mut buf = vec![(
        CronMeta::new(Duration::from_secs_f32(100.0), Duration::from_secs_f32(0.0), 3),
        noop::State
    ); JOB_CNT];

    let mut pool = noop::Pool::new(POOLSIZE, 0);
    rt.block_on(async {
        pool.fire_jobs(&mut buf)
    });

    b.iter(|| rt.block_on(pool.next()));
}

#[bench]
fn pool_poll_addition_bench(b: &mut Bencher) {

    #[derive(Debug, Default, Clone)]
    pub struct State {
        a: u16,
        b: u16
    };

    #[derive(Debug, Default)]
    pub struct Worker {
        count: usize,
    }

    #[async_trait::async_trait]
    impl CRON for Worker 
    {
        type State = State;
        type Response = usize;

        /// Run function, and then append to parent if more jobs are needed
        async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {
            Ok((SignalControl::Success(false), Some((state.a as u32 + state.b as u32) as usize)))
        }
    }

    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();


    
    for _ in 0..JOB_CNT {
        let meta = CronMeta::new(Duration::from_secs_f32(100.0), Duration::from_secs_f32(0.0), 3);
        buf.push((meta, State {
            a: rand::random(),
            b: rand::random()
        }));
    }

    let mut pool: CronPool<Worker, usize, State> = CronPool::new(POOLSIZE, 0);
    
    rt.block_on(async {    
        pool.fire_jobs(&mut buf);
    });

    b.iter(|| rt.block_on(pool.next()));
}


#[test]
fn pool_single_in_single_out() {
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    buf.push((
        CronMeta::new(Duration::from_secs_f32(100.0), Duration::from_secs_f32(0.0), 3),
        noop::State
    ));

    rt.block_on(async move {
        let mut pool = noop::Pool::new(POOLSIZE, 0);
        pool.fire_jobs(&mut buf);
        
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
        let rbuf = pool.next().await.unwrap();
        assert_eq!(rbuf.len(), 1);
    });
}



#[test]
fn pool_job_count_accurate() {
    let mut rt = Runtime::new().unwrap();

    let mut buf = vec![(
        CronMeta::new(Duration::from_secs_f32(100.0), Duration::from_secs_f32(0.0), 3),
        noop::State
    ); 1];

    rt.block_on(async move {
        let mut pool = noop::Pool::new(POOLSIZE, 0);
        
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
    
    let mut buf = vec![(
        CronMeta::new(Duration::from_secs_f32(100.0), Duration::from_secs_f32(0.0), 3),
        noop::State
    ); JOB_CNT];
    
    rt.block_on(async move {
        let mut pool = noop::Pool::new(POOLSIZE, 0);

        assert_eq!(buf.len(), JOB_CNT);
        pool.fire_jobs(&mut buf);
        
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
        
        assert_eq!(JOB_CNT, pool.next().await.unwrap().len());
    });
}



//Run all once, retry, run again, succeed, all in; all out
#[test]
fn all_retry_now_once() {
    use noop as mock;
    
    #[derive(Debug)]
    struct Worker;

    #[async_trait::async_trait]
    impl CRON for Worker {
        type State = usize;
        type Response = mock::Response;

        async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {            
            *state += 1;

            if *state > 1 {
                Ok((SignalControl::Success(false), Some(mock::Response)))
            }
            else {
                Ok((SignalControl::Retry, Some(mock::Response)))
            }
            
        }
    }


    //create async runtime
    let mut rt = Runtime::new().unwrap();
    let mut buf = vec![(
        CronMeta::new(Duration::from_secs_f32(100.0), Duration::from_secs_f32(0.0), 3),
        0
    ); JOB_CNT];
    
    let mut pool: CronPool<Worker, mock::Response, usize> = CronPool::new(POOLSIZE, 0);

    rt.block_on(async move {
        
        // Fire and retrieve once
        pool.fire_jobs(&mut buf);
        assert_eq!(buf.len(), 0);
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;

        for (meta, ctrl, resp, state) in pool.next().await.unwrap() {
            assert_eq!(meta.ctr, 2);
            assert_eq!(state, 2);
        }
    });
}



//assert all tasks do eventually timeout
#[test]
fn all_timeout() {
    use noop as mock;

    #[derive(Debug)]
    struct Worker;

    #[async_trait::async_trait]
    impl CRON for Worker {
        type State = mock::State;
        type Response = mock::Response;

        async fn exec(_state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {            
            tokio::time::delay_for(Duration::from_secs(3)).await;
            Ok((SignalControl::Success(false), Some(mock::Response)))
        }
    }

    
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();
    
    let mut pool: CronPool<Worker, mock::Response, mock::State> = CronPool::new(POOLSIZE, 0);

    for _ in 0..JOB_CNT {
        buf.push((
            CronMeta::new(Duration::from_secs_f32(0.1), Duration::from_secs_f32(0.0), 1),
            noop::State
        ));
    }

    rt.block_on(async move {
        assert_eq!(buf.len(), JOB_CNT);
        // Fire and retrieve once
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), JOB_CNT+1);
        assert_eq!(buf.len(), 0);
        
        tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
        
        while let Some(data) = pool.next().await {
            for (meta, ..) in data {
                assert_eq!(meta.ctr, 2)
            }
        }
        
        assert_eq!(pool.job_count(), 1);
       

    });
}
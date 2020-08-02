/*
Mocked up pools for testing and benching.

Most of this stuff is repetative fluff
*/
use crate::{
    schedule::{
        pool::CronPool,
        meta::CronMeta,
        SignalControl,
        CRON, Subscriber, MetaSubscriber
    },
    error::Error
};

use std::time::Duration;

pub const JOB_CNT: usize = 100;
pub const POOLSIZE: usize = 16_384;

pub mod noop {
    use super::*;

    pub type Pool = CronPool<Worker<State, Response>, Response, State>;

    pub type NoOpPool<S, R> = CronPool<Worker<S, R>, R, S>;

    #[derive(Debug, Default)]
    pub struct State;

    #[derive(Debug, Default)]
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
        async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {
            Ok((SignalControl::Success(false), Some(R::default())))
        }

        fn name() -> String {
            "noopworker".to_string()
        }
    }

    pub fn get_pool(timeout: f32, fire_in: f32, max_retries: usize) -> Pool {
        let mut pool: Pool = Pool::new(POOLSIZE);

        for x in 0..JOB_CNT {
            pool.insert(State, Duration::from_secs_f32(timeout), Duration::from_secs_f32(fire_in), max_retries);
        }

        pool
    }

}

pub mod counter {
    use super::*;
    
    pub type Pool = CronPool<Worker, Response, State>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct State(usize);

    impl State {
        pub fn count(&self) -> usize {
            self.0
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct Response(usize);

    #[derive(Debug, Default)]
    pub struct Worker;

    #[async_trait::async_trait]
    impl CRON for Worker {
        type State = State;
        type Response = Response;

        /// Run function, and then append to parent if more jobs are needed
        async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {            
            let resp = Ok((SignalControl::Success(false), Some(Response(state.0))));
            state.0 += 1;
            resp
        }

        fn name() -> String {
            format!("{:?}", Worker)
        }
    }
    

    pub fn get_pool(timeout: f32, fire_in: f32, max_retries: usize) -> Pool {
        let mut pool: Pool = Pool::new(POOLSIZE);

        for _ in 0..JOB_CNT {
            pool.insert(State(0), Duration::from_secs_f32(timeout), Duration::from_secs_f32(fire_in), max_retries);
        }

        pool
    }
}
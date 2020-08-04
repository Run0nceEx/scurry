/*
Mocked up pools for testing and benching.

Most of this stuff is repetative fluff
*/
use super::benchmarks::rewrite_schedule::{
        CronPool,
};


use crate::schedule::{
    SignalControl,
    CRON
};
use crate::error::Error;

use std::time::Duration;

pub const JOB_CNT: usize = 100;
pub const POOLSIZE: usize = 16_384;

pub mod noop {
    use super::*;

    pub type NoOpPool<S, R> = CronPool<Worker<S, R>, R, S>;
    pub type Pool = NoOpPool<State, Response>;


    #[derive(Debug, Default, Clone)]
    pub struct State;

    #[derive(Debug, Default, Clone)]
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

        fn name() -> String {
            "noopworker".to_string()
        }
    }

    pub fn get_pool(timeout: f32, fire_in: f32, max_retries: usize) -> Pool {
        let mut pool: Pool = Pool::new(POOLSIZE);

        for _ in 0..JOB_CNT {
            pool.insert(State, Duration::from_secs_f32(timeout), Duration::from_secs_f32(fire_in), max_retries);
        }

        pool
    }

}

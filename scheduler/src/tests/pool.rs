use crate::{
    schedule::{CRON, MetaSubscriber, CronPool, Subscriber, meta::CronMeta, SignalControl},
    error::Error
};

use super::mock::{JOB_CNT, POOLSIZE};

use tokio::runtime::Runtime;

#[test]
fn single_in_single_out() {
    use super::mock::noop;
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    rt.block_on(async move {
        let mut pool = noop::Pool::new(POOLSIZE);

        pool.insert(
            noop::State,
            std::time::Duration::from_secs(100),
            std::time::Duration::from_secs(0),
            3
        );
        
        pool.release_ready(&mut buf).await.unwrap();
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), 1);

        let x = pool.process_reschedules().await;
        
        match  x {
            Some(_) => assert_eq!(1, 1),
            None => assert_eq!(1, 0),
        }
    });
}


#[test]
fn all_in_all_out() {
    use super::mock::noop::get_pool;
    let mut rt = Runtime::new().unwrap();
    
    let mut buf = Vec::new();
    
    rt.block_on(async move {
        let mut pool = get_pool(100.0, 0.0, 3);
        pool.release_ready(&mut buf).await.unwrap();

        assert_eq!(buf.len(), JOB_CNT);
        assert_eq!(pool.job_count(), 0);


        pool.fire_jobs(&mut buf);
        
        assert_eq!(pool.job_count(), JOB_CNT);
        
        let mut ctr = 0;
        for _ in 0..JOB_CNT {
            if let Some(_) = pool.process_reschedules().await {
                ctr += 1;
            }
        }


        assert_eq!(pool.job_count(), 0);
        assert_eq!(ctr, JOB_CNT);
    });
}



// Run all once, retry, run again, succeed, all in; all out
#[test]
fn all_retry_now_once() {
    use super::mock::noop as mock;
    
    #[derive(Debug)]
    struct RetryOnce;

    #[async_trait::async_trait]
    impl MetaSubscriber for RetryOnce {
        async fn handle(&mut self, meta: &mut CronMeta, signal: &SignalControl) -> Result<SignalControl, Error>
        {
            match meta.ctr {
                0 | 1 => return Ok(SignalControl::Reschedule(std::time::Duration::from_secs(0))), // set up retry
                _ => return Ok(SignalControl::Success(false)), // auto pass
            }
            
        }
    }

    //create async runtime
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    rt.block_on(async move {
        let mut pool = mock::get_pool(100.0, 0.0, 3);
        pool.subscribe_meta_handler(RetryOnce);

        // FIRST ITER
        pool.release_ready(&mut buf).await.unwrap();
        
        assert_eq!(buf.len(), JOB_CNT);
        assert_eq!(pool.job_count(), 0);

        // Fire and retrieve once
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), JOB_CNT);
        assert_eq!(buf.len(), 0);

        tokio::time::delay_for(std::time::Duration::from_secs(2)).await;

        // capture all the results
        let mut ctr = 0;
        for _ in 0..JOB_CNT {
            if let Some((meta, resp, state)) = pool.process_reschedules().await {
                ctr += 1;
                assert!(meta.ctr > 1);
            }
        }

        assert_eq!(ctr, 0);
        assert_eq!(pool.job_count(), 0);
        assert_eq!(buf.len(), 0);


        // SECOND ITER

        pool.release_ready(&mut buf).await.unwrap();
        
        assert_eq!(buf.len(), JOB_CNT);
        assert_eq!(pool.job_count(), 0);

        // Fire and retrieve once
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), JOB_CNT);
        assert_eq!(buf.len(), 0);

        tokio::time::delay_for(std::time::Duration::from_secs(2)).await;

        // capture all the results
        for _ in 0..JOB_CNT {
            if let Some((meta, resp, state)) = pool.process_reschedules().await {
                assert!(meta.ctr > 1);
            }
        }

        assert_eq!(pool.job_count(), 0);
        assert_eq!(buf.len(), 0);

    });
}



// assert all tasks do eventually timeout
#[test]
fn all_timeout() {
    use super::mock::noop as mock;
    use std::time::Duration;

    #[derive(Debug)]
    struct Worker;

    #[async_trait::async_trait]
    impl CRON for Worker {
        type State = mock::State;
        type Response = mock::Response;

        async fn exec(state: &mut Self::State) -> Result<(SignalControl, Option<Self::Response>), Error> {            
            tokio::time::delay_for(Duration::from_secs(3)).await;
            Ok((SignalControl::Success(false), Some(mock::Response)))
        }

        fn name() -> String {
            format!("{:?}", Worker)
        }
    }

    
    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    rt.block_on(async move {
        let mut pool: CronPool<Worker, mock::Response, mock::State> = CronPool::new(POOLSIZE);

        let live_for = Duration::from_secs(1);

        for _ in 0..JOB_CNT {
            pool.insert(mock::State, live_for, Duration::from_secs(0), 1);
        }

        pool.release_ready(&mut buf).await.unwrap();
        pool.fire_jobs(&mut buf);

        tokio::time::delay_for(Duration::from_secs(5)).await;

        for _ in 0..JOB_CNT {
            if let Some((meta, response, state)) = pool.process_reschedules().await {
                assert!(meta.ctr > meta.max_ctr);
                assert!(meta.durations.get(0).unwrap() > &live_for);
            }
        }
    });
}


// if the mspc::channel has nothing in its queue, it will block
// we have to make sure we bypass blocked execution

#[test]
fn pass_blocking_recv() {
    use super::mock::noop;

    let mut rt = Runtime::new().unwrap();

    rt.block_on(async move {
        let mut pool = noop::Pool::new(POOLSIZE);

        match tokio::time::timeout(std::time::Duration::from_secs(10), pool.process_reschedules()).await {
            Ok(_) => assert_eq!(0, 0),
            Err(_) => assert_eq!(1, 0)
        }
    });
}


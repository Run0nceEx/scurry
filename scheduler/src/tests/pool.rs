

use crate::{
    schedule::{CRON, MetaSubscriber, CronPool, Subscriber, meta::CronMeta, SignalControl},
    error::Error
};

use super::mock::{JOB_CNT, POOLSIZE};

use tokio::runtime::Runtime;


// All in - all out test of delay queue
#[test]
fn release_all() {
    use super::mock::noop::get_pool;

    let mut rt = Runtime::new().unwrap();
    
    let mut pool = get_pool(100.0, 0.0, 3);

    let mut buf = Vec::new();
    
    rt.block_on(async move {
        pool.release_ready(&mut buf).await.unwrap();
        assert_eq!(buf.len(), JOB_CNT);
    });
}


#[test]
fn job_count_accurate() {
    use super::mock::noop::get_pool;

    let mut rt = Runtime::new().unwrap();
    
    let mut pool = get_pool(100.0, 0.0, 3);
    let mut buf = Vec::new();
    
    rt.block_on(async move {
        pool.release_ready(&mut buf).await.unwrap();
        pool.fire_jobs(&mut buf);
        assert_eq!(pool.job_count(), JOB_CNT);
    });
}


// Run all once, retry, run again, succeed, all in; all out
#[test]
fn all_in_all_out() {
    use super::mock::noop::get_pool;
    
    let mut rt = Runtime::new().unwrap();
    let mut pool = get_pool(100.0, 0.0, 3);
    let mut buf = Vec::new();
    
    rt.block_on(async move {
        pool.release_ready(&mut buf).await.unwrap();
        pool.fire_jobs(&mut buf);
        
        assert_eq!(pool.job_count(), JOB_CNT);

        for _ in 0..JOB_CNT {
            pool.process_reschedules(&mut buf).await;
        }

        assert_eq!(buf.len(), 0);
        assert_eq!(pool.job_count(), 0);
    });

}



// Run all once, retry, run again, succeed, all in; all out
#[test]
fn all_retry_once() {
    use super::mock::noop as mock;
    
    #[derive(Default, Debug)]
    struct CountUpState(usize);


    #[derive(Debug)]
    struct RetryOnce(usize);

    #[async_trait::async_trait]
    impl Subscriber<mock::Response, CountUpState> for RetryOnce {
        async fn handle(&mut self, 
            meta: &mut CronMeta,
            signal: &mut SignalControl,
            data: &Option<mock::Response>,
            state: &mut CountUpState,
        ) -> Result<(), Error> {
            // Check state
            match state.0 {
                0 => *signal = SignalControl::RetryNow, // set up retry
                1 => assert_eq!(0, 0), // auto pass
                _ => assert_eq!(1, 0) // automatically fail
            }
            Ok(())
        }
    }

    let mut pool: mock::GenericPool<CountUpState, mock::Response> = CronPool::new(POOLSIZE);
    

    //pool.subscribe_meta_handler(RetryOnce(0));

    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    rt.block_on(async move {
        pool.release_ready(&mut buf).await.unwrap();
        pool.fire_jobs(&mut buf);

        for _ in 0..JOB_CNT*2 {
            pool.process_reschedules(&mut buf).await;
        }
    });

}


// Assert nothing can block our execution, and that all tasks do eventually timeout
#[test]
fn all_timeout() {



}


// Keep retrying until scheduler yells at us
// for maximum rescheduling
#[test]
fn all_retry_exhaust() {


}




// assert that its actually delaying as long as it says it is
#[test]
fn jitter_delay_test() {


}


// if the mspc::channel has nothing in its queue, it will block
// we have to make sure we bypass blocked execution

#[test]
fn pass_blocking_recv() {
    use super::mock::noop::get_pool;

    let mut rt = Runtime::new().unwrap();
    let mut pool = get_pool(1.0, 0.0, 3);
    let mut buf = Vec::new();


    rt.block_on(async move {
        pool.process_reschedules(&mut buf).await;
        assert_eq!(0, 0);
    });    

}


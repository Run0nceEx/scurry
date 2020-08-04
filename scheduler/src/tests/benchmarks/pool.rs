extern crate test;
use test::Bencher;


use crate::tests::mock::POOLSIZE;
use tokio::runtime::Runtime;



#[bench]
fn single_in_single_out(b: &mut Bencher) {
    use super::rewrite_schedule::CronPool;
    use crate::tests::mock::noop;

    type Pool = CronPool<noop::Worker<noop::State, noop::Response>, noop::Response, noop::State>;

    let mut rt = Runtime::new().unwrap();
    let mut buf = Vec::new();

    let mut pool = rt.block_on(async move {
        let mut pool = Pool::new(POOLSIZE);
        pool.release_ready(&mut buf).await.unwrap();
        pool.fire_jobs(&mut buf);
        pool
    });

    let mut rbuf = Vec::new();
    b.iter(move || rt.block_on(pool.process_reschedules(&mut rbuf)));
}
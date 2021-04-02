use std::net::SocketAddr;
use tokio_stream::StreamExt;

use tokio::{
    net::{TcpListener, TcpStream},
    io::AsyncWriteExt
};

use px_core::{
    pool::{CRON, Worker, JobCtrl},
    util::Boundary,
    model::State as NetState,
    error::Error
};

use criterion::{
    black_box,
    criterion_group,
    Criterion,
    BenchmarkId,
    Throughput
};

const JOB_CNT: usize = 100;

/// Test how fast tokio's connect is.
fn connect_bench(b: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(num_cpus::get())
		.enable_all()
        .build()
        .unwrap();
    
    let addr: SocketAddr = "127.0.0.1:20927".parse().unwrap();
    
    rt.spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            let (con, _addr) = listener.accept().await.unwrap();
            drop(con);
        }
    });
    
    b.bench_function("tokio connect", |b| b.iter(|| {
            rt.block_on(async move {
                let mut x = TcpStream::connect(addr).await.unwrap();
                x.shutdown().await.unwrap();
            });
        }
    ));
}

struct NoOpPool<S, R>(pub Worker<Handler<S, R>, R, S>)
where 
    S: Send + Sync + Clone + Default + std::fmt::Debug + 'static,
    R: Send + Sync + Clone + Default + std::fmt::Debug + 'static;


impl<S, R> NoOpPool<S, R>
where 
    S: Send + Sync + Clone + Default + std::fmt::Debug + 'static,
    R: Send + Sync + Clone + Default + std::fmt::Debug + 'static
{
    #[inline(always)]
    pub fn default_test() -> Self {
        Self(Worker::new(Boundary::Unlimited, std::time::Duration::from_secs(5)))
    }

}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct State(usize);

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Response {}

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

fn evpool_state_blackbox_next(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
		.worker_threads(num_cpus::get())
		.enable_all()
        .build()
        .unwrap();
    
    for i in [1, 10, 100, 1000, 10000, 25000].iter() {
        let mut buf = Vec::with_capacity(i+1);
        
        for _ in 0..*i {
            buf.push(black_box(State(rand::random())));
        }

        let len = buf.len();
        let mut pool = NoOpPool::<State, Response>::default_test();
        
        rt.block_on(async {
            pool.0.spawn(&mut buf)
        });

        let mut group = c.benchmark_group("evpool next");
            group.throughput(Throughput::Elements(len as u64));
            group.bench_function(BenchmarkId::new("evpool", len), |b| {
                b.iter(|| rt.block_on(pool.0.next()));
            });
            group.finish();
        }
}

criterion::criterion_group!(all, connect_bench, evpool_state_blackbox_next);
criterion::criterion_main!(all);

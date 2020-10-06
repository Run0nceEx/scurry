use std::{
	net::SocketAddr,
	time::Duration,
	
};

use crate::{
	netlib::probe::{
		tcp::TcpProbe,
		//socks5::{Socks5Scanner, ScanResult}
	},
	pool::{Worker, Pool, CRON, JobErr},
	util::{Boundary, get_max_fd},
	model::{State as NetState},
	cli::{input::combine, output::OutputType},
};

const TICK_NS: u64 = 500;

impl From<JobErr> for NetState {
	fn from(x: JobErr) -> NetState {
		NetState::Closed
	}
}

pub async fn connect_scan<'a>(generator: &mut combine::Feeder<'a>, results: &mut OutputType, timeout: Duration)
{
	let mut pool = tokio_tcp_pool::<TcpProbe, SocketAddr, SocketAddr>(timeout);
	let mut buffer = Vec::new();
	
	loop {
		if !generator.is_done() {
			pool.fire_from_feeder(&mut buffer, generator).await;
		}
		
		let jobs_done = pool.tick(&mut buffer).await;	
		
		results.handle(&jobs_done);
		
		if buffer.len() == 0 && generator.is_done() && pool.job_count() == 1 {
			break
		}
		
		tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
	}

	results.handle(&pool.flush_channel());
}


pub async fn socks_scan<'a>(generator: &mut combine::Feeder<'a>, results: &mut OutputType, timeout: Duration)
{
    // let mut pool = tokio_tcp_pool::<Socks5Scanner, ScanResult, SocketAddr>(timeout);
	// let mut buffer = Vec::new();
	
	// loop {
	// 	if !generator.is_done() {
	// 		pool.fire_from_feeder(&mut buffer, generator).await;
	// 	}
		
	// 	let jobs_done = pool.tick(&mut buffer).await;		
		
	// 	results.handle(&jobs_done);
		
	// 	if buffer.len() == 0 && generator.is_done() && pool.job_count() == 1 {
	// 		break
	// 	}
		
	// 	tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
	// }

	// results.handle(&pool.flush_channel());
}

fn tokio_tcp_pool<J, R, S>(timeout: Duration) -> Pool<J, R, S>
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,

{
	let limit = match get_max_fd().unwrap() {
		Boundary::Limited(i) => Boundary::Limited(i-100),
		x => x
	};

    Pool::new(
		Worker::new(limit, timeout)
	)
}

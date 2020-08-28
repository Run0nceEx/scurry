use std::{
	net::{SocketAddr, IpAddr},
	time::Duration,
	collections::HashMap,
};

use crate::{
	libcore::{
		task::{
			pool::CronPool,
			meta::CronMeta,
			Pool,
			CRON
		},
		
		discovery::service::{
			connect_scan::{OpenPortJob, Job, PortState},
			socks5::{ScanResult, Socks5Scanner}
		},

		util::{Boundary, get_max_fd},
	},
};

use smallvec::SmallVec;

const TICK_NS: u64 = 500;

pub async fn connect_scan<T>(generator: T, results: &mut HashMap<IpAddr, SmallVec<[Service; 32]>>) 
where 
	T: Iterator<Item=SocketAddr>,
{
    let mut pool = tokio_tcp_pool::<OpenPortJob, PortState, Job>();

	pool.mut_buffer().extend(
		generator
			.map(Job::from)
			.map(|x| (CronMeta::new(Duration::from_secs(100), 3), x))
	);
	
	tracing::event!(target: "Main", tracing::Level::INFO, "Queued up [{}] jobs", pool.buffer().len());
	
	pool.buffer();

	while pool.is_working() {
		pool.buffer();
		
        pool.tick().await;
        tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
	}
}


pub async fn socks_scan<T>(generator: T) 
where 
	T: Iterator<Item=SocketAddr>,
{
	
    let mut pool = tokio_tcp_pool::<Socks5Scanner, ScanResult, SocketAddr>();
		
	pool.mut_buffer().extend(
		generator
			.map(|x| (CronMeta::new(Duration::from_secs(100), 3), x))
	);
	
	tracing::event!(target: "Main", tracing::Level::INFO, "Queued up [{}] jobs", pool.buffer().len());
	
	while pool.is_working() {
        pool.tick().await;
        tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
	}
}

fn fd_throttle(leave_available: f32) -> usize {
	let boundary = get_max_fd().unwrap();
	//eprintln!("Setting throttle to {}", boundary);

	match boundary {
		Boundary::Limited(x) => (x as f32).powf(leave_available).round() as usize,
		Boundary::Unlimited => 0
	}
}



fn tokio_tcp_pool<J, R, S>() -> Pool<J, R, S>
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,

{
	const EVEC_SIZE: usize = 16384;
	const FD_AVAIL_PRECENT: f32 = 0.02;

	let throttle = fd_throttle(FD_AVAIL_PRECENT);
	
    Pool::new(CronPool::new(EVEC_SIZE, throttle))
}

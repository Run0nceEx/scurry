use std::net::SocketAddr;
use std::time::Duration;

use crate::libcore::{
	task::{
		pool::CronPool,
		meta::CronMeta,
        Pool,
        CRON
	},
	
	discovery::service::{
		connect_scan::{OpenPortJob, Job, PortState}
    },

    util::{Boundary, get_max_fd}
};


fn tokio_tcp_pool<J, R, S>() -> Pool<J, R, S>
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,

{
    let throttle = {
		let boundary = get_max_fd().unwrap();
		println!("Setting throttle to {}", boundary);

		match boundary {
			Boundary::Limited(x) => x,
			Boundary::Unlimited => 0
		}
	};
	
	let mut job_pool: CronPool<J, R, S> = CronPool::new(16*1024, throttle-100);
    let mut pool = Pool::new(job_pool);
    pool
}



async fn connect_scan<T>(generator: T) 
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
	
	while pool.is_working() {
		pool.tick().await;
	}
}


async fn socks_scan<T>(generator: T) 
where 
	T: Iterator<Item=SocketAddr>,
{
	
    let mut pool = tokio_tcp_pool::<OpenPortJob, PortState, SocketAddr>();
		
	pool.mut_buffer().extend(
		generator
			.map(|x| (CronMeta::new(Duration::from_secs(100), 3), x))
	);
	
	tracing::event!(target: "Main", tracing::Level::INFO, "Queued up [{}] jobs", pool.buffer().len());
	
	while pool.is_working() {
		pool.tick().await;
	}
}
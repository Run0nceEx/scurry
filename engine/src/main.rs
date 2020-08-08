#![feature(test)]
#[macro_use] extern crate tracing;

mod runtime;
use runtime::pool::CronPool;

mod handlers;
use handlers::{
	connect_scan::{OpenPortJob, Job, PortState, PrintSub},
	watchdog::WatchDog
};


mod error;
use error::Error;


use tokio::io::BufReader;
use tokio::prelude::*;


use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;


mod pnet_futures;

fn setup_subscribers() {
	let subscriber = FmtSubscriber::builder()
		// all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
		// will be written to stdout.
		.with_max_level(Level::DEBUG)
		// completes the builder.
		.finish();
	
	tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}



#[tokio::main]
async fn main() -> Result<(), Error> {
	setup_subscribers();
	
	let mut job_pool: CronPool<OpenPortJob, PortState, Job> = CronPool::new(16*1024);
	//job_pool.subscribe(PrintSub::new());
	//job_pool.subscribe_meta_handler(WatchDog::new().await);


	let file = tokio::fs::File::open("/home/ghost/projects/px-engine/proxbox-rs/scheduler/test.lst").await?;
	let mut reader = BufReader::new(file);
	let mut buf = String::new();
	
	let mut i: u32 = 0;

	let mut job_buf = Vec::new();

	while let Ok(n) = reader.read_line(&mut buf).await {
		if n > 0 {
			match buf.trim().parse::<std::net::SocketAddr>() {
				Ok(addr) =>  job_buf.push(addr),
				Err(e) => eprintln!("failed to parse {}", buf.trim())	
			}
			i += 1;
		}

		else {
			println!("lines read: {}", i);
			break
		}
		
		buf.clear();
	}


	// let mut ticker = std::time::Instant::now();


	// loop {
	// 	job_pool.release_ready(&mut job_buf).await?;

	// 	if job_buf.len() > 0 {
	// 		tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "Adding [{}] jobs", job_buf.len());
	// 		job_pool.fire_jobs(&mut job_buf);
	// 	}
		
	// 	job_pool.process_reschedules(&mut rbuf).await;

	// 	if ticker.elapsed() >= std::time::Duration::from_secs(5) {
	// 		tracing::event!(target: "Schedule Thread", tracing::Level::DEBUG, "Job count is [{}] jobs", job_pool.job_count());
	// 		ticker = std::time::Instant::now();
	// 	}

	// 	rbuf.clear();
	// 	job_buf.clear();
	// 	//tokio::time::delay_for(tokio::time::Duration::from_secs_f64(0.00001)).await;
	// }
	Ok(())
}
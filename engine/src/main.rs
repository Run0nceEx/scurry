#![feature(test)]
#[macro_use] extern crate tracing;

mod util;

mod task;

use task::{
	pool::CronPool,
	meta::CronMeta,
	stash::Stash,
	SignalControl
};

mod workers;
use workers::{
	connect_scan::{OpenPortJob, Job, PortState},
};

mod error;
use error::Error;

use tokio::io::BufReader;
use tokio::prelude::*;
use tokio::stream::{StreamExt, Stream};

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use std::time::Duration;

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


async fn read_ip_port_file(job_buf: &mut Vec<(CronMeta, Job)>) -> Result<(), Error> {
	let file = tokio::fs::File::open("/home/ghost/projects/px-engine/proxbox-rs/engine/sample.lst").await?;
	let mut reader = BufReader::new(file);
	let mut buf = String::new();

	while let Ok(n) = reader.read_line(&mut buf).await {
		match buf.trim().parse::<std::net::SocketAddr>() {
				Ok(addr) =>  {
					let meta = CronMeta::new(Duration::from_secs_f32(5.0), Duration::from_secs_f32(0.0), 3);
					let job = Job::new(addr);
					job_buf.push((meta, job));
				},
				
				Err(e) => {
					eprintln!("failed to parse {}", buf.trim())
				}	
		}
		
		if n == 0 {
			break
		}

		buf.clear();

	}
	Ok(())
}




#[tokio::main]
async fn main() -> Result<(), Error> {
	setup_subscribers();

	let boundary = util::get_max_fd().unwrap();
		
	let throttle = {
		tracing::event!(target: "Schedule Thread", tracing::Level::INFO, "throttle set at {}", boundary);
		match boundary {
			util::Boundary::Limited(x) => x,
			util::Boundary::Unlimited => 0
		}
	};

	let mut job_pool: CronPool<OpenPortJob, PortState, Job> = CronPool::new(16*1024, throttle-100);
	let mut stash: Stash<Job> = Stash::new();
	let mut job_buf = Vec::new();

	read_ip_port_file(&mut job_buf).await?;

	tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Queued [{}] jobs", job_buf.len());
	
	let mut ticker = std::time::Instant::now();
	let mut prev_count = job_buf.len();

	loop {
		stash.release(&mut job_buf).await;

		if job_buf.len() > 0 {
			tracing::event!(target: "Schedule Thread", tracing::Level::TRACE, "Adding [{}] jobs", job_buf.len());
			job_pool.fire_jobs(&mut job_buf);		
		}

		if ticker.elapsed() >= std::time::Duration::from_secs(5) {
			tracing::event!(
				target: "Schedule Thread", tracing::Level::DEBUG, "Job count is [{}/{} | +/- {}] jobs",
				job_pool.job_count(), job_buf.len(),
				prev_count - job_buf.len()
			);

			prev_count = job_buf.len();
			ticker = std::time::Instant::now();
		}

		while let Some(chunk) = job_pool.next().await {
			for (meta, ctrl, resp, state) in chunk {
				match ctrl {
					SignalControl::Stash(duration) => stash.insert(meta, state, &duration),
					SignalControl::Success(x) => {
						if let Some(PortState::Open(addr)) = resp {
							println!("open: {}", addr);
						}
					},
					_ => {}
				}
			}
		}

		if job_pool.job_count() == 1 {
			println!("Finished all jobs.");
			return Ok(())
		}
	}
}
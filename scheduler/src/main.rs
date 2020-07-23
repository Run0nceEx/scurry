#[macro_use] extern crate diesel;
#[macro_use] extern crate tracing;

// behavioral
mod schedule;
mod handlers;
mod error;

// persistence components
mod database;

// ephemeral persistence components maybe eventually?
use schedule::{
	sugar::{ScheduledJobPool},
};

use handlers::{
	connect_scan::{OpenPortJob, Job, PortState, PrintSub},
	fail_rate::WarnConstFailRate

};
use error::Error;
use std::io::{Read, BufReader, BufRead};

use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Error> {
	let subscriber = FmtSubscriber::builder()
		// all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
		// will be written to stdout.
		.with_max_level(Level::TRACE)
		// completes the builder.
		.finish();
	
		tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
	
	let mut job_pool: ScheduledJobPool<OpenPortJob, PortState, Job> = ScheduledJobPool::new(16*1024);
	job_pool.subscribe(PrintSub::new());
	job_pool.subscribe_meta_handler(WarnConstFailRate::new((0.65, 10000)));


	let file = std::fs::OpenOptions::new().read(true).open("/tmp/list")?;
	let mut reader = BufReader::new(file);
	let mut buf = String::new();
	
	let mut i: u32 = 0;

	while let Ok(n) = reader.read_line(&mut buf) {
		if n > 0 {
			match buf.trim().parse() {
				Ok(addr) =>  {
					job_pool.insert(
						Job::new(addr, 1),
						std::time::Duration::from_secs(2),
						std::time::Duration::from_millis(1),
						2
					);
				},
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
	

	let mut job_buf = Vec::new();
	loop {
		job_pool.release_ready(&mut job_buf).await?;
		job_pool.fire_jobs(&mut job_buf);
		job_pool.process_reschedules(&mut job_buf).await;
	}
}
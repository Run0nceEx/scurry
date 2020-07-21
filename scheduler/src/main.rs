#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
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

use handlers::connect_scan::{OpenPortJob, Job, PortState, PrintSub};
use error::Error;
use std::io::{Read, BufReader, BufRead};


#[tokio::main]
async fn main() -> Result<(), Error> {

	let mut job_pool: ScheduledJobPool<OpenPortJob, PortState, Job> = ScheduledJobPool::new(16*1024);
	job_pool.subscribe(PrintSub::new());

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
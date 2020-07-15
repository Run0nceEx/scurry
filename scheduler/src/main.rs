#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate tracing;

// behavioral
mod schedule;
mod handlers;
mod error;
mod state;

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

	let mut job_pool: ScheduledJobPool<OpenPortJob, PortState, Job> = ScheduledJobPool::new();
	job_pool.subscribe(PrintSub::new());

	let file = std::fs::OpenOptions::new().read(true).open("/tmp/list")?;
	let mut reader = BufReader::new(file);
	let mut buf = String::new();
	let mut i = 0;

	while let Ok(n) = reader.read_line(&mut buf) {
		i += 5;
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
		}
		else {
			break
		}
		buf.clear();
	}
	
	loop {
		job_pool.process_jobs().await?;
		job_pool.process_events().await;
		//std::thread::sleep(std::time::Duration::from_secs(5));
		//println!("{:?}", job_pool.handles());
	}
			

	//Ok(())
}
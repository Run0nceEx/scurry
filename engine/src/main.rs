#![feature(test)]

mod input;
mod cli;

mod libcore;
mod menu;
use crate::input::parser::ScanMethod;


mod error;
use error::Error;

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

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

use std::net::SocketAddr;



use structopt::StructOpt;

fn main() -> Result<(), Error> {
	use tokio::runtime::Runtime;

	let mut runtime: Runtime = Runtime::new()?;
	let opt = cli::Cli::from_args();

	return runtime.block_on(async move {
		setup_subscribers();
		
		match opt.method {
			ScanMethod::Open => {},
			ScanMethod::Socks5 => {}
		}

		Ok(())	
	});
}
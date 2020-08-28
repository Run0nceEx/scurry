#![feature(test)]

mod cli;
mod libcore;

use std::collections::HashMap;

use crate::cli::input::parser::ScanMethod;
use crate::cli::error::Error;


use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use structopt::StructOpt;
use tokio::runtime::Runtime;


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


fn main() -> Result<(), Error> {
	let opt = cli::args::Arguments::from_args();
	let mut results_buf = HashMap::new();

	let mut runtime: Runtime = Runtime::new()?;
	return runtime.block_on(async move {
		setup_subscribers();
		
		match opt.method {
			ScanMethod::Open => cli::menu::connect_scan(unimplemented!().drain(), &mut results_buf).await,
			ScanMethod::Socks5 => cli::menu::socks_scan(unimplemented!().drain(), ).await
		};

		Ok(())
	});
}
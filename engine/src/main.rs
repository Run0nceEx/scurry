#![feature(test)]

mod cli;

mod discovery;
mod pool;
mod error;
mod util;
mod model;

use crate::cli::error::Error;

use structopt::StructOpt;

use tokio::runtime::Runtime;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use cli::{
	menu::Output,
	input::{
		parser::{ScanMethod, Format},
		combine::Feeder,
	}
};

fn setup_subscribers() {
	let subscriber = FmtSubscriber::builder()
		// all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
		// will be written to stdout.
		.with_max_level(Level::TRACE)
		// completes the builder.
		.finish();
	
	tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}

fn main() -> Result<(), Error> {
	//cli::opt::Arguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
	let opt = cli::opt::Arguments::from_args();

	let mut runtime: Runtime = Runtime::new()?;

	let mut output_type: cli::menu::Output = opt.format.clone().into();	

	return runtime.block_on(async move {
		setup_subscribers();
		
		let mut generator = Feeder::new(&opt.ports, &opt.target);

		match opt.method {
		 	ScanMethod::Open => cli::menu::connect_scan(&mut generator, &mut output_type).await,
			ScanMethod::Socks5 => unimplemented!() // cli::menu::socks_scan(generators, &mut output_type).await
		};

		if let Output::Map(map) = output_type {
			match opt.format {
				Format::Stdout => map.into_iter().for_each(|(key, service)| {
					print!("{}", key);
					service.iter().for_each(|s| println!("\t\t\t{}\t{}", s.port, s.state));
				}),
				Format::Json => println!("{}", serde_json::to_string_pretty(&map).unwrap()),
				Format::Stream => unreachable!() 
			}
		}
		
		Ok(())
	});
}
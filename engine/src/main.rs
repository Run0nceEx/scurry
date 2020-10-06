#![feature(test)]

mod error;
mod model;
mod pool;
mod util;

mod netlib;
mod cli;

use crate::cli::error::Error;
use structopt::StructOpt;
use tokio::runtime::Builder;

use std::time::Duration;

use cli::{
	output::OutputType,
	input::{
		parser::{ScanMethod, Format},
		combine::Feeder,
	}
};


fn main() -> Result<(), Error> {
	//cli::opt::Arguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
	let opt = cli::opt::Arguments::from_args();

	let mut runtime = Builder::default()
		.core_threads(opt.threads.unwrap_or(num_cpus::get()))
		.enable_all()
		.build()?;

	let mut output_type: OutputType = opt.format.clone().into();	

	return runtime.block_on(async move {
		let mut generator = Feeder::new(&opt.ports, &opt.target, &opt.exclude);

		match opt.method {
		 	ScanMethod::Open => cli::menu::connect_scan(
				&mut generator,
				&mut output_type,
				Duration::from_secs_f32(opt.timeout)
			).await,
			
			ScanMethod::Socks5 => cli::menu::socks_scan(
				&mut generator,
				&mut output_type,
				Duration::from_secs_f32(opt.timeout)
			).await
		};

		if let OutputType::Map(map) = output_type {
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
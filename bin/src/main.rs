#![feature(test)]

mod cli;
mod handlers;

use crate::cli::error::Error;
use structopt::StructOpt;
use tokio::runtime::Builder;

use std::net::SocketAddr;
use std::time::Duration;
use handlers::{
	socks5::{ScanResult, Socks5Scanner},
	tcp::TcpProbe
};
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

	let mut runtime = Builder::new_multi_thread()
		.worker_threads(opt.threads.unwrap_or(num_cpus::get()))
		.enable_all()
		.build()?;
	
	let mut output_type: OutputType = opt.format.clone().into();	

	return runtime.block_on(async move {
		let mut generator = Feeder::new(&opt.ports, &opt.target, &opt.exclude);
		match opt.method {
			 ScanMethod::Complete { wait_flag } => cli::menu::run_handle::<TcpProbe, SocketAddr, SocketAddr>
			(
				&mut generator,
				&mut output_type,
				Duration::from_secs_f32(opt.timeout)
			).await,
			
			ScanMethod::Socks => cli::menu::run_handle::<Socks5Scanner, ScanResult, SocketAddr>
			(
				&mut generator,
				&mut output_type,
				Duration::from_secs_f32(opt.timeout)
			).await,
			
			_ => unimplemented!()

		};

		if let OutputType::Map(map) = output_type {
			match opt.format {
				Format::Stdout => map.into_iter().for_each(|(key, services)| {
					print!("{}", key);
					services.iter().for_each(|(port, netstate)| println!("\t\t\t{}\t{}", port, netstate));
				}),
				Format::Json => unimplemented!(), //println!("{}", serde_json::to_string_pretty(&map).unwrap()),
				Format::Stream => unreachable!() 
			}
		}
		
		Ok(())
	});
}
#![feature(test)]

mod cli;
mod libcore;

use crate::cli::error::Error;

use structopt::clap::Shell;
use structopt::StructOpt;

use tokio::runtime::Runtime;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use std::net::SocketAddr;

use cli::{
	menu::Output,
	input::{
		parser::{ScanMethod, AddressInput, Format},
		combine::{IpAddrPortCombinator, CidrPortCombinator},
		file::InputFile
	}
};
use libcore::model::PortInput;

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



fn make_generator<'a>(targets: &'a [AddressInput], ports: &'a [PortInput]) -> Box<dyn Iterator<Item=SocketAddr> + 'a> {
	//this hack did not work
	
	let mut generators: Box<dyn Iterator<Item=SocketAddr>> = Box::new(Vec::new().into_iter());
	
	let mut singles = Vec::new();
	
	for entry in targets {
		match entry {
			AddressInput::CIDR(rng) => 
				generators = Box::new(generators.chain(CidrPortCombinator::new(&rng, &ports))),	
			AddressInput::Singleton(singleton) =>
				generators = Box::new(generators.chain(IpAddrPortCombinator::new(singleton.clone(), &ports))),
			
			AddressInput::Pair(socket) => singles.push(socket), 
			AddressInput::File(_name) => unimplemented!()
		}
	}
	generators = Box::new(generators.chain(singles.into_iter().map(|x| *x)));
	generators
}


fn main() -> Result<(), Error> {
	cli::opt::Arguments::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
	let opt = cli::opt::Arguments::from_args();

	let mut runtime: Runtime = Runtime::new()?;

	let mut output_type: cli::menu::Output = opt.format.clone().into();	

	return runtime.block_on(async move {
		setup_subscribers();
		
		let generator = make_generator(&opt.target, &opt.ports);
		
		println!("targets: {:?}", &opt.target);


		match opt.method {
		 	ScanMethod::Open => cli::menu::connect_scan(generator, &mut output_type).await,
			ScanMethod::Socks5 => unimplemented!() // cli::menu::socks_scan(generators, &mut output_type).await
		};
		println!("{:?}", output_type);

		if let Output::Map(map) = output_type {
			match opt.format {
				Format::Stdout => map.into_iter().for_each(|(key, service)| {
					println!("{}", key);
					service.iter().for_each(|s| println!("\t\t{}\t{}", s.port, s.state));
				}),
				Format::Json => println!("{}", serde_json::to_string_pretty(&map).unwrap()),
				
				Format::Stream => unreachable!() 
			}
		}
		
		Ok(())
	});
}
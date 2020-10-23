use std::{
	net::{SocketAddr, IpAddr},
	collections::HashMap,
	fmt::Debug
};

use netcore::{
	model::{State as NetState, Service},
	pool::JobCtrl,
};

use crate::cli::input::parser;

#[derive(Debug)]
pub enum OutputType {
	Stream,
	Map(HashMap<IpAddr, Vec<Service>>)
}


impl From<parser::Format> for OutputType {
	fn from(x: parser::Format) -> Self {
		if let parser::Format::Stream = x {
			OutputType::Stream
		}
		else {
			OutputType::Map(HashMap::new())
		}
	}
}

trait WriteOutput {
    fn handle_output(&self, handle: &mut OutputType);
}

pub trait CastAs<T> {
	fn cast<'a>(&'a self) -> &'a T;
}

impl<T> CastAs<T> for T {
	fn cast(&self) -> &T {
		self
	}
}

impl OutputType {
	pub fn handle<R, S>(&mut self, buf: &Vec<(JobCtrl<R>, S)>)
	where
		R: Debug,
		S: CastAs<SocketAddr> + Debug
	{
		match self {
			OutputType::Stream => {
				for (sig, state) in buf {
					let sock = state.cast();
					match sig {
						JobCtrl::Return(netstate, _resp) => println!("{}\t{}\t{}", sock.ip(), sock.port(), netstate),	
						JobCtrl::Error(err) => eprintln!("unable to run [{}] {:?}:[{:?}]", sock, err, sig)
					}
				}
			},

			OutputType::Map(map) => {
				for (sig, state) in buf {
					let sock = state.cast();
					
					let service = match sig {
						JobCtrl::Return(netstate, _resp) => 
							Service {port: sock.port(), state: *netstate},
						JobCtrl::Error(_err) =>
							Service { port: sock.port(), state: NetState::Closed }				
					};

					match map.get_mut(&sock.ip()) {
						Some(buf) => { buf.push(service); },
						None => { 
							let mut buf = Vec::new();
							buf.push(service);
							map.insert(sock.ip(), buf);
						}
					}
				}
			}


			
		}
	}
}

use std::{
	net::{SocketAddr, IpAddr},
	time::Duration,
	collections::HashMap,
};
use crate::{
	libcore::{
		pool::{
			Worker,
			Pool,
			CRON,
			JobCtrl,
			JobErr
		},
		
		discovery::service::{
			connect_scan::{OpenPortJob},
		},

		util::{Boundary, get_max_fd},
		model::{Service, State as NetState},
	},
	cli::input::parser
};

use smallvec::SmallVec;
const TICK_NS: u64 = 500;

#[derive(Debug)]
pub enum Output {
	Stream,
	Map(HashMap<IpAddr, SmallVec<[Service; 32]>>)
}


impl From<parser::Format> for Output {
	fn from(x: parser::Format) -> Self {
		if let parser::Format::Stream = x {
			Output::Stream
		}
		else {
			Output::Map(HashMap::new())
		}
	}
}

impl From<JobErr> for NetState {
	fn from(x: JobErr) -> NetState {
		NetState::Closed
	}
}

use std::fmt::Debug;

trait CastAs<T> {
	fn cast<'a>(&'a self) -> &'a T;
}

impl<T> CastAs<T> for T {
	fn cast(&self) -> &T {
		self
	}
}

impl Output {
	fn handle<R, S>(&mut self, buf: &Vec<(JobCtrl<R>, S)>)
	where R: Debug,
	S: CastAs<SocketAddr> + Debug
	{
		match self {
			Output::Stream => {
				for (sig, state) in buf {
					let sock = state.cast();

					match sig {
						JobCtrl::Return(netstate, resp) => println!("{}\t{}\t{}", sock.ip(), sock.port(), netstate),	
						JobCtrl::Error(err) => eprintln!("unable to run {:?}:[{:?}]", state, sig)
					}
				}
			},

			Output::Map(map) => {
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
							let mut buf = SmallVec::new();
							buf.push(service);
							map.insert(sock.ip(), buf);
						}
					}

				}
			}
		}
	}
}

pub async fn connect_scan<T>(generator: T, results: &mut Output) 
where 
	T: Iterator<Item=SocketAddr>,
{
    let mut pool = tokio_tcp_pool::<OpenPortJob, SocketAddr, SocketAddr>();


	//println!("BUF {:?}", generator.collect::<Vec<_>>());
	// len = 0

	pool.mut_buffer().extend(
		generator
	);

	loop {
		println!("HALLO");
		let x = pool.tick().await;
		println!("+++ {:?}", x);
		results.handle(&x);

		if !pool.is_working() {
			break
		}  

		tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
	}
	

}


// pub async fn socks_scan<T>(generator: T, results: &mut Output) 
// where 
// 	T: Iterator<Item=SocketAddr>,
// {
	
//     let mut pool = tokio_tcp_pool::<Socks5Scanner, ScanResult, SocketAddr>();
		
// 	pool.mut_buffer().extend(
// 		generator
// 			.map(|x| (CronMeta::new(Duration::from_secs(100), 3), x))
// 	);
	
// 	while pool.is_working() {
//         let x = pool.tick().await;
// 		//results.handle(&x);
//         tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
// 	}
// }

fn fd_throttle(leave_available: FdAvail) -> usize {
	let boundary = get_max_fd().unwrap();
	eprintln!("Setting throttle to {}", boundary);

	match boundary {
		Boundary::Limited(limit) => match leave_available {
			FdAvail::Percentage(x) => (limit as f32).powf(x).round().floor() as usize,
			FdAvail::Number(x) => limit-x
		},
		Boundary::Unlimited => 0
	}
}

enum FdAvail {
	Number(usize),
	Percentage(f32)
}

fn tokio_tcp_pool<J, R, S>() -> Pool<J, R, S>
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,

{
	const EVEC_SIZE: usize = 16384;
	const FD_AVAIL_PRECENT: f32 = 0.02;
	const TIMEOUT: u64 = 15;
	
    Pool::new(
		Worker::new(EVEC_SIZE, fd_throttle(FdAvail::Number(100)), std::time::Duration::from_secs(TIMEOUT))
	)
}

use std::{
	net::{SocketAddr, IpAddr},
	time::Duration,
	collections::HashMap,
	fmt::Debug
};

use crate::{
	libcore::{
		discovery::service::connect_scan::OpenPortJob,
		pool::{Worker, Pool, CRON, JobCtrl, JobErr},
		util::{Boundary, get_max_fd},
		model::{Service, State as NetState},
	},
	cli::input::{parser, combine}
};

use smallvec::SmallVec;

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
	where
		R: Debug,
		S: CastAs<SocketAddr> + Debug
	{
		match self {
			Output::Stream => {
				for (sig, state) in buf {
					let sock = state.cast();

					match sig {
						JobCtrl::Return(netstate, _resp) => println!("{}\t{}\t{}", sock.ip(), sock.port(), netstate),	
						JobCtrl::Error(err) => eprintln!("unable to run {:?}:[{:?}]", err, sig)
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

pub async fn connect_scan<'a>(generator: &mut combine::Feeder<'a>, results: &mut Output) {
	const TICK_NS: u64 = 500;
	const CHUNK_SIZE: usize = 4000;

	let mut pool = tokio_tcp_pool::<OpenPortJob, SocketAddr, SocketAddr>();
	let mut buffer = Vec::new();

	generator.generate_chunk(&mut buffer, CHUNK_SIZE);

	while pool.is_working() {
		let x = pool.tick(&mut buffer).await;		
		results.handle(&x);

		tokio::time::delay_for(Duration::from_nanos(TICK_NS)).await;
	}

	results.handle(&pool.flush_channel());
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
	const TIMEOUT: u64 = 15;

	let limit = match get_max_fd().unwrap() {
		Boundary::Limited(i) => Boundary::Limited(i-100),
		x => x
	};

    Pool::new(
		Worker::new(limit, std::time::Duration::from_secs(TIMEOUT))
	)
}

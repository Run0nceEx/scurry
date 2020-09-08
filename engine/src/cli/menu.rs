use std::{
	net::{SocketAddr, IpAddr},
	time::Duration,
	collections::HashMap,
};
use crate::{
	libcore::{
		task::{
			pool::CronPool,
			meta::CronMeta,
			Pool,
			CRON,
			WorkBuf,
			SignalControl
		},
		
		discovery::service::{
			connect_scan::{OpenPortJob},
			//socks5::{ScanResult, Socks5Scanner}
		},

		util::{Boundary, get_max_fd},
		model::{Service, State as NetState},
	},
	cli::input::parser
};

use smallvec::SmallVec;
const TICK_NS: u64 = 500;

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

use std::fmt::Display;


impl Output {
	fn handle<T>(&mut self, buf: &WorkBuf<SocketAddr, T>) {
		match self {
			Output::Stream => {
				for (meta, sig, state) in buf {
					match sig {
						SignalControl::Success(netstate, resp) => println!("{}\t{}\t{}", resp.ip(), resp.port(), netstate),	
						_ => eprintln!("unable to run [{}]", meta.id)
					}
					
				}
			},

			Output::Map(map) => {
				for (meta, sig, state) in buf {
					match sig {
						SignalControl::Success(netstate, resp) => {
							match map.get_mut(&resp.ip()) {
								Some(buf) => buf.push(Service {port: resp.port(), state: *netstate}),
								None => {}
							}
						},
						
						_ => eprintln!("unable to run [{}]", meta.id)
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

	pool.mut_buffer().extend(
		generator
			.map(|x| (CronMeta::new(Duration::from_secs(100), 3), x))
	);
	
	pool.buffer();

	while pool.is_working() {
		let x = pool.tick().await;
		//results.handle(&x);

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

fn fd_throttle(leave_available: f32) -> usize {
	let boundary = get_max_fd().unwrap();
	eprintln!("Setting throttle to {}", boundary);

	match boundary {
		Boundary::Limited(x) => (x as f32).powf(leave_available).round() as usize,
		Boundary::Unlimited => 0
	}
}



fn tokio_tcp_pool<J, R, S>() -> Pool<J, R, S>
where
	J: CRON<Response = R, State = S> + std::marker::Unpin,
	R: Send + Sync + Clone + std::fmt::Debug + 'static,
    S: Send + Sync + Clone + std::fmt::Debug + 'static,

{
	const EVEC_SIZE: usize = 16384;
	const FD_AVAIL_PRECENT: f32 = 0.02;
	
    Pool::new(
		CronPool::new(EVEC_SIZE, fd_throttle(FD_AVAIL_PRECENT))
	)
}

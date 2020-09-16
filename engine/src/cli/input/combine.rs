use cidr_utils::cidr::{IpCidrIpAddrIterator, IpCidr};
use std::net::{IpAddr, SocketAddr};
use crate::libcore::model::PortInput;
use super::parser::AddressInput;


enum FeedItem<'a> {
	IpAddr(IpAddrPortCombinator<'a>),
	Cidr(CidrPortCombinator<'a>)
}

pub struct Feeder<'a> {
	ports: &'a [PortInput],
	items: Vec<FeedItem<'a>>,
	working_on: Option<FeedItem<'a>>,
}


impl<'a> Feeder<'a> {
	pub fn new(ports: &'a [PortInput], address_input: &Vec<AddressInput> ) -> Self {
		let mut items = Vec::new();

		for addr in address_input {
			match addr {
				AddressInput::CIDR(cidr) => items.push(
					FeedItem::Cidr(CidrPortCombinator::new(&cidr, ports))
				),

				AddressInput::Singleton(ip) => items.push(
					FeedItem::IpAddr(IpAddrPortCombinator::new(*ip, ports))
				),
				
				_ => unimplemented!()
			}
		}
		
		Self {
			ports,
			items,
			working_on: None
		}
	}


	pub fn generate_chunk(&mut self, buffer: &mut Vec<SocketAddr>, amount: usize) -> usize {
		let original = buffer.len();

		while original-buffer.len() >= amount {
			if let Some(iterator) = &mut self.working_on {
				let item = match iterator {
					FeedItem::Cidr(cidr_iterator) => cidr_iterator.next(),
					FeedItem::IpAddr(ip_iterator) => ip_iterator.next()
				};
				
				match item {
					Some(item) => {
						buffer.push(item);
						continue
					}

					None => self.working_on = None
				}
			}
			
			if let Some(x) = self.items.pop() {
				self.working_on = Some(x);
			}

			else {
				return original-buffer.len()
			}
		}

		original-buffer.len()
	}
}


pub struct IpAddrPortCombinator<'a> {
	ip: IpAddr,
	ports: &'a [PortInput],
	current_port_range: Option<std::ops::Range<u16>>,
	i: usize
}

impl<'a> IpAddrPortCombinator<'a> {
	pub fn new(ip: IpAddr, ports: &'a [PortInput]) -> Self {
		Self {
			ip,
			ports,
			current_port_range: None,
			i: 0
		}
	} 
}

impl<'a> Iterator for IpAddrPortCombinator<'a> {
	type Item = SocketAddr;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(rng) = &mut self.current_port_range {
			if let Some(port) = rng.next() {
				return Some(SocketAddr::new(self.ip, port))
			}
			else {
				self.current_port_range = None;
			}
		}
		
		if self.i > self.ports.len()-1 {
			return None
		}

        let port = &self.ports[self.i];
        
		let addr = match port {
			PortInput::Singleton(port) => SocketAddr::new(self.ip, *port),
			PortInput::Range(rng) => {
				let mut rng = rng.clone();
				
				let port = rng.next().unwrap();
				self.current_port_range = Some(rng);
				
				SocketAddr::new(self.ip, port)
			}
		};
		
		self.i += 1;
		return Some(addr);
	}
}

pub struct CidrPortCombinator<'a> {
	cidr: IpCidrIpAddrIterator,
	inner: IpAddrPortCombinator<'a>
}

impl<'a> CidrPortCombinator<'a> {
    pub fn new(cidr: &IpCidr, ports: &'a [PortInput]) ->  Self {
        let mut rng = cidr.iter_as_ip_addr();
        let seed = rng.next().unwrap();
        
        Self {
            cidr: rng,
            inner: IpAddrPortCombinator::new(seed, ports)
        }
    }
}

impl<'a> Iterator for CidrPortCombinator<'a> {
	type Item = SocketAddr;

	fn next(&mut self) -> Option<Self::Item> {
		match self.inner.next() {
			Some(addr) => return Some(addr),
			None => {
				if let Some(ip) = self.cidr.next() {
					self.inner = IpAddrPortCombinator::new(ip, self.inner.ports);
					return Some(self.inner.next().unwrap())
				}
				return None
			}
		}
	}
}


#[cfg(test)]
mod test {
	use super::*;
	use std::net::Ipv4Addr;
    use std::str::FromStr;

	#[test]
	fn ip_generates_one() {
        let ports = &[PortInput::from_str("1").unwrap()];

		let data = IpAddrPortCombinator::new(
			"127.0.0.1".parse().unwrap(), 
			ports
		).next();

		assert_eq!(data, Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 1)));

	}

    #[test]
	fn ip_generates_many() {
        let ports = &[PortInput::from_str("1-5").unwrap()];

		println!("PORTS: {:?}", ports);
        let data: Vec<SocketAddr> = IpAddrPortCombinator::new(
			"127.0.0.1".parse().unwrap(), 
			ports
		).collect();
        
		println!("SOCKETS: {:?}", data);
		for i in 1..5 {
			assert!(data.contains(
				&SocketAddr::new(
					IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
					i
				)
			))
		}
	}
	
	#[test]
	fn cidr_generates_many() {
		let ports = &[PortInput::from_str("1").unwrap()];

		println!("PORTS: {:?}", ports);
        let data: Vec<SocketAddr> = CidrPortCombinator::new(
			&IpCidr::from_str("127.0.0.1/30").unwrap(),
			ports
		).collect();
        
		println!("SOCKETS: {:?}", data);
		for ip_end in 1..4 {
			assert!(data.contains(
				&SocketAddr::new(
					IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip_end)),
					1
				)
			))
		}
	}

	#[test]
	fn cidr_generates_one() {
		let ports = &[PortInput::from_str("1").unwrap()];

		let data = CidrPortCombinator::new(
			&IpCidr::from_str("127.0.0.1/32").unwrap(),
			ports
		).next();

		assert_eq!(data, Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 1)));
	}
}
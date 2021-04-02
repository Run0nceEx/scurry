use cidr_utils::cidr::{IpCidrIpAddrIterator, IpCidr};
use std::net::{IpAddr, SocketAddr};
use px_core::model::PortInput;
use super::parser::AddressInput;

#[derive(Debug)]
enum FeedItem<'a> {
	IpAddr(IpSpan<'a>),
	Cidr(CidrSpan<'a>)
}

enum ExclusionItem {
	Range(IpCidr),
	Addr(IpAddr)
}

pub struct Feeder<'a> {
	exclusions: Vec<ExclusionItem>,
	items: Vec<FeedItem<'a>>,
	working_on: Option<FeedItem<'a>>,
}

impl<'a> Feeder<'a> {
	pub fn new(ports: &'a [PortInput], address_input: &[AddressInput], exclusions: &'a [AddressInput]) -> Self {
		let mut items = Vec::new();

		for addr in address_input {
			items.push(match addr {
				AddressInput::CIDR(cidr) =>
					FeedItem::Cidr(CidrSpan::new(&cidr, ports)),
				AddressInput::Singleton(ip) =>
					FeedItem::IpAddr(IpSpan::new(*ip, ports)),
				_ => unimplemented!()
			});
		}
		
		let mut exclude = Vec::new();
		for addr in exclusions {
			exclude.push(match addr { 
				AddressInput::CIDR(cidr) => ExclusionItem::Range(cidr.clone()),
				AddressInput::Singleton(ip) => ExclusionItem::Addr(ip.clone()),
				_ => unimplemented!()
			});
		}

		Self {
			items,
			exclusions: exclude,
			working_on: None
		}
	}

	pub fn is_done(&self) -> bool {
		match &self.working_on {
			None => self.items.len() == 0,
			Some(_) => return false
		}
	}

	pub fn generate_chunk(&mut self, buffer: &mut Vec<SocketAddr>, amount: usize) -> usize {
		let original = buffer.len();

		while amount > buffer.len() - original {
			if let Some(iterator) = &mut self.working_on {
				let item = match iterator {
					FeedItem::Cidr(cidr_iterator) => cidr_iterator.next(),
					FeedItem::IpAddr(ip_iterator) => ip_iterator.next()
				};
				
				match item {
					Some(item) => {
						let push_buf = !self.exclusions.iter().any(|x|
							match x {
								ExclusionItem::Range(rng) => rng.contains(item.ip()),
								ExclusionItem::Addr(ip) => *ip == item.ip()
							}
						);

						if push_buf { buffer.push(item) }
						continue
					}
					None => self.working_on = None
				}
			}

			else {
				self.working_on = self.items.pop();
				match self.working_on {
					Some(_) => continue,
					None => return buffer.len()-original 
				}
			}
		}

		return buffer.len()-original;
	}
}


// TODO(adam) : Update using Combinators provided in library
#[derive(Debug)]
pub struct IpSpan<'a> {
	ip: IpAddr,
	ports: &'a [PortInput],
	current_port_range: Option<std::ops::Range<u16>>,
	i: usize
}

impl<'a> IpSpan<'a> {
	pub fn new(ip: IpAddr, ports: &'a [PortInput]) -> Self {
		IpSpan {
			ip,
			ports,
			current_port_range: None,
			i: 0
		}
	} 
}

impl<'a> Iterator for IpSpan<'a> {
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

#[derive(Debug)]
pub struct CidrSpan<'a> {
	cidr: IpCidrIpAddrIterator,
	inner: IpSpan<'a>
}

impl<'a> CidrSpan<'a> {
    pub fn new(cidr: &IpCidr, ports: &'a [PortInput]) ->  Self {
        let mut rng = cidr.iter_as_ip_addr();
        let seed = rng.next().unwrap();
        
        CidrSpan {
            cidr: rng,
            inner: IpSpan::new(seed, ports)
        }
    }
}

impl<'a> Iterator for CidrSpan<'a> {
	type Item = SocketAddr;

	fn next(&mut self) -> Option<Self::Item> {
		match self.inner.next() {
			Some(addr) => return Some(addr),
			None => {
				if let Some(ip) = self.cidr.next() {
					self.inner = IpSpan::new(ip, self.inner.ports);
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

		let data = IpSpan::new(
			"127.0.0.1".parse().unwrap(), 
			ports
		).next();

		assert_eq!(data, Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 1)));

	}

    #[test]
	fn ip_generates_many() {
        let ports = &[PortInput::from_str("1-5").unwrap()];

        let data: Vec<SocketAddr> = IpSpan::new(
			"127.0.0.1".parse().unwrap(), 
			ports
		).collect();
        
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
		let ports = &[PortInput::from_str("1-5").unwrap()];

		println!("PORTS: {:?}", ports);
        let data: Vec<SocketAddr> = CidrSpan::new(
			&IpCidr::from_str("127.0.0.1/24").unwrap(),
			ports
		).collect();
        
		println!("SOCKETS: {:?}", data);
		// check all results showed up
		for ip_end in 1..255 {
			for port in 1..5 {
				assert!(data.contains(
					&SocketAddr::new(
						IpAddr::V4(Ipv4Addr::new(127, 0, 0, ip_end)),
						port
					)
				))
			}
		}

		assert!(
			!data.contains(
				&SocketAddr::new(
					IpAddr::V4(Ipv4Addr::new(127, 0, 1, 1)),
					80
				)
			)
		)

	}

	#[test]
	fn cidr_generates_one() {
		let ports = &[PortInput::from_str("1").unwrap()];

		let data = CidrSpan::new(
			&IpCidr::from_str("127.0.0.1/32").unwrap(),
			ports
		).next();

		assert_eq!(data, Some(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), 1)));
	}

	#[test]
	fn generator_from_cli() {
		let ports = &[PortInput::from_str("1-5").unwrap()];
		
		let x = &[
			AddressInput::CIDR(IpCidr::from_str("10.0.0.1/24").unwrap())
		];
		
		let mut feed = Feeder::new(ports, x, &[]);
		
		let mut buf = Vec::new();
		feed.generate_chunk(&mut buf, 8);

		println!("{:?}", buf);
		assert_eq!(buf.len(), 8)
	}
}
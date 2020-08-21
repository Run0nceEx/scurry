use cidr_utils::cidr::{IpCidrIpAddrIterator, IpCidr};
use std::net::{IpAddr, SocketAddr};
use super::parser::PortInput;

pub struct IpAddrPortCombinator<'a> {
	inner: IpAddr,
	ports: &'a [PortInput],
	current_port_range: Option<std::ops::Range<u16>>,
	i: usize
}

impl<'a> IpAddrPortCombinator<'a> {
	pub fn new(ip: IpAddr, ports: &'a [PortInput]) -> Self {
		Self {
			ports: ports,
			inner: ip,
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
				return Some(SocketAddr::new(self.inner, port))
			}
			else {
				self.current_port_range = None;
			}
		}

		if self.i >= self.ports.len()-1 {
			return None
		}

        let port = &self.ports[self.i];
        
		let addr = match port {
			PortInput::Singleton(port) => SocketAddr::new(self.inner, *port),
			PortInput::Range(rng) => {
				let mut rng = rng.clone();
				let port = rng.next().unwrap();
				self.current_port_range = Some(rng);

				SocketAddr::new(self.inner, port)
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
    use std::collections::HashSet;
    use std::convert::TryFrom;

    #[test]
	fn does_not_duplicate() {
        let ports = &[
            PortInput::try_from("1").unwrap(),
            PortInput::try_from("2").unwrap(),
            PortInput::try_from("3").unwrap(),
            PortInput::try_from("5-80").unwrap(),
        ];
        
        let data: Vec<SocketAddr> = CidrPortCombinator::new(
            &IpCidr::from_str("127.0.0.1/24").unwrap(),
            ports
        ).collect();
        
        let mut copy_data: HashSet<SocketAddr> = HashSet::new();
        copy_data.extend(data.iter().map(|x| *x));

        let check: Vec<SocketAddr> = copy_data.drain().collect();

        assert_eq!(check, data);
    }
}
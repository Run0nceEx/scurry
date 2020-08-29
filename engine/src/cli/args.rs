use structopt::StructOpt;
use super::input::parser::*;

use crate::libcore::model::PortInput;

#[derive(Debug, StructOpt)]
#[structopt(about = "Port scanner")]
pub struct Arguments {
    #[structopt(long = "debug-trace", env = "PXENGINE_DEBUG")]
    pub debug_trace: bool,
    
    // Target IP addresses, supports IPv4 and IPv6. Accepts Accepts a sequence of IPs "10.0.0.1" and CIDR "10.0.0.1/24"
    #[structopt(parse(try_from_str = address_parser), short)]
    pub target: Vec<AddressInput>,

    /// Ranges of ports you'd like to scan on every IP, Accepts a sequence of numbers "80" and ranges "8000-10000"
    #[structopt(parse(try_from_str = port_parser), short)]
    pub port: Vec<PortInput>,

    #[structopt(parse(try_from_str = method_parser), short)]
    pub method: ScanMethod,

    #[structopt(long, required_if("method", "syn"), env = "PXENGINE_IFACE")]
    pub iface: String,
}
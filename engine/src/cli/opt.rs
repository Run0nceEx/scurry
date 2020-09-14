use structopt::StructOpt;
use super::input::parser::*;

use crate::libcore::model::PortInput;

#[derive(Debug, StructOpt)]
#[structopt(about = "Port scanner")]
pub struct Arguments {
    
    #[structopt(long, short = "-fmt", default_value = "stdout")]
    /// Specify output format
    pub format: Format,

    #[structopt(long = "--debug-trace", short, env = "SCURRY_DEBUG")]
    /// Enable debugging all targets by using "ALL", or specify with a list of named targets.
    pub debug_target: Vec<String>,
    
    #[structopt(parse(try_from_str = address_parser), short, long)]
    /// Target IP addresses, supports IPv4 and IPv6. Accepts Accepts a sequence of IPs "10.0.0.1" and CIDR "10.0.0.1/24"
    pub target: Vec<AddressInput>,

    /// Exclude by IP/cidr address
    #[structopt(parse(try_from_str = address_parser), short = "-x", long)]
    pub exclude: Vec<AddressInput>,

    #[structopt(long, short)]
    /// Ranges of ports you'd like to scan on every IP, Accepts a sequence of numbers "80" and ranges "8000-10000"
    pub ports: Vec<PortInput>,

    #[structopt(short, long, default_value = "open")]
    /// choice of handler used
    pub method: ScanMethod,

}
use structopt::StructOpt;
use super::input::parser::*;

use px_core::common::port::{PortInput, port_parser};
use std::str::FromStr;

#[derive(Debug, StructOpt)]
#[structopt(about = "Port scanner")]
pub struct Arguments {

    #[structopt(long, short = "-fmt", default_value = "stdout")]
    /// Specify output format
    pub format: Format,
    
    #[structopt(parse(try_from_str = address_parser), short = "-t", long)]
    /// Target IP addresses, supports IPv4 and IPv6. Accepts Accepts a sequence of IPs "10.0.0.1" and CIDR "10.0.0.1/24"
    pub target: Vec<AddressInput>,

    #[structopt(parse(try_from_str = address_parser), short = "-x", long)]
    /// Exclude by IP/cidr address
    pub exclude: Vec<AddressInput>,

    #[structopt(parse(try_from_str = port_parser), long, short)]
    /// Ranges of ports you'd like to scan on every IP, Accepts a sequence of numbers "80" and ranges "8000-10000"
    pub ports: Vec<PortInput>,

    #[structopt(short, long, default_value = "open")]
    /// choice of handler used
    pub method: ScanMethod,

    #[structopt(long)]
    // amount of threads (defaults to core count)
    pub threads: Option<usize>,

    // #[structopt(parse(from_occurrences)), short]
    // // unused currently
    // pub verbose: u8,

    #[structopt(long, default_value = "5", env = "SCURRY_TIMEOUT")]
    /// Specify output format
    pub timeout: f32,

    //#[structopt(long = "--verify-tls", env = "SCURRY_VERIFY_TLS")]
    // Specify output format
    // pub verify_tls: bool,



}
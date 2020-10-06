use structopt::StructOpt;
use super::input::parser::*;

use crate::model::PortInput;

#[derive(Debug, StructOpt)]
#[structopt(about = "Port scanner")]
pub struct Arguments {

    #[structopt(long, short = "-fmt", default_value = "stdout")]
    /// Specify output format
    pub format: Format,
    
    #[structopt(parse(try_from_str = address_parser), short = "-t", long)]
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

    #[structopt(long)]
    pub threads: Option<usize>,

    #[structopt(parse(from_occurrences))]
    pub verbose: u8,

    #[structopt(long, default_value = "5", env = "SCURRY_TIMEOUT")]
    /// Specify output format
    pub timeout: f32,

    //#[structopt(long = "--verify-tls", env = "SCURRY_VERIFY_TLS")]
    // Specify output format
    // pub verify_tls: bool,



}
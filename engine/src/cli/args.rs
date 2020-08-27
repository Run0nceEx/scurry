use structopt::StructOpt;

use super::input::parser::*;


#[derive(Debug, StructOpt)]
#[structopt(about = "Port scanner")]
pub struct Arguments {
    #[structopt(long, env = "PXENGINE_DEBUG")]
    pub debug: bool,
    
    #[structopt(parse(try_from_str = address_parser), short)]
    pub target: Vec<AddressInput>,

    #[structopt(parse(try_from_str = port_parser), short)]
    pub port: Vec<PortInput>,

    #[structopt(parse(try_from_str = method_parser), short)]
    pub method: ScanMethod,

    #[structopt(long, required_if("method", "syn"), env = "PXENGINE_IFACE")]
    pub iface: String,
}
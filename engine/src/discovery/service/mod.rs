// contains code for discovering what service is running behind a port

mod nmap;

pub mod connect_scan;
//pub mod socks5;

use crate::pool::JobErr;

fn handle_io_error(err: std::io::Error) -> JobErr {
    match err.kind() {
        std::io::ErrorKind::Other => match err.raw_os_error() {
            Some(i) => JobErr::Errno(i),
            None => JobErr::Other
        },
        x => JobErr::IO(x),
    }
}

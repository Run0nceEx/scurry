//pub mod vscan;
//pub mod parsers;
pub mod tcp;
pub mod socks5;


use px_core::pool::JobErr;
fn handle_io_error(err: std::io::Error) -> JobErr {
    match err.kind() {
        std::io::ErrorKind::Other => match err.raw_os_error() {
            Some(i) => JobErr::Errno(i),
            None => JobErr::Other
        },
        x => JobErr::IO(x),
    }
}
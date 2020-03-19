use async_std::io;
use async_std::net::{TcpStream};
mod net;

async fn noop(_stream: io::Result<TcpStream>) -> io::Result<()> { Ok(()) }

#[async_std::main]
async fn main() -> io::Result<()> {

    net::negotiate(vec!["", ""].into_iter(), noop).await;

    Ok(())
}
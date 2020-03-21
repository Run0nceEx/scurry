use async_std::io;
use async_std::net::{TcpStream};
mod net;

async fn noop(_stream: io::Result<TcpStream>) -> io::Result<()> { Ok(()) }

#[async_std::main]
async fn main() -> io::Result<()> {
    // let (s, r) = async_std::sync::channel(20);

    // net::negotiate(vec!["", ""].into_iter(), noop, s).await;

    Ok(())
}
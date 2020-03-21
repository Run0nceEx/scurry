
use super::negotiations::spawner;
use async_std::sync::{Sender, Receiver, channel};
use async_std::net::TcpStream;
use async_std::io;

async fn noop(_: TcpStream) -> io::Result<()> {
    Ok(())
}


async fn accepts_string_vec() {
    
    let (s, r) = channel(1024);
    let x: Vec<String> = Vec::new();

    spawner(x.into_iter(), noop, s).await;


}
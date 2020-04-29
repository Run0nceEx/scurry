mod proto;

use std::net::SocketAddr;
pub use proto::Scannable;


pub async fn is_protocol<T, S, E>(x: T, addr: SocketAddr) -> Result<(bool, T), E>
where T: Scannable<S, E> {
    let mut y = x.connect(addr).await?;
    Ok((x.scan(&mut y).await?, x))
} 
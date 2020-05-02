mod proto;

use std::net::SocketAddr;
pub use proto::{Scannable};

pub enum Error {}

pub async fn is_protocol<T, S, E>(mut x: T, addr: SocketAddr) -> Result<(bool, T), Error>
where T: Scannable  {
    Ok((x.scan().await, x))
}
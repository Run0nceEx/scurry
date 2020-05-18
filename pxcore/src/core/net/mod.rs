mod comm;

pub use comm::{Identifier, Connector};

// pub async fn is_protocol<T, S, E>(mut x: T, addr: SocketAddr) -> Result<(bool, T), Box<std::error::Error>>
// where T: Scannable  {
//     Ok((x.scan().await?, x))
// }
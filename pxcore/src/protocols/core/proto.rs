
use tokio::io::{AsyncRead, AsyncWrite};
use std::pin::Pin;

use std::net::SocketAddr;


#[async_trait::async_trait]
pub trait Scannable<T, E> {
    async fn connect(&self, addr: SocketAddr) -> Result<T, E>;
    async fn scan(&self, stream: &mut T) -> Result<bool, E>;
}

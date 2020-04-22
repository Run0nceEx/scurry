mod http;
mod error;
mod socks;
mod chains;

pub use error::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use std::pin::Pin;

use std::net::SocketAddr;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;


#[async_trait::async_trait]
pub trait Scannable<T> {
    async fn connect(&self, addr: SocketAddr ) -> Result<T, Error>;
    async fn scan(&self, addr: SocketAddr) -> Result<bool, Error>;
}

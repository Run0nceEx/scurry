// use async_std::net::{TcpStream, SocketAddr, ToSocketAddrs};
// use super::codec::ProxyProtocol;
// use async_std::io;
// use async_std::pin::Pin;
// use async_std::task::{Poll, Context};

// // Used to implement proxy chaining functionality
// trait ChainLink: Read + Write {
//     fn build_forward(&self, buf: &mut [u8]) {
//         unimplemented!()
//     }
// }

// /// A `Future` which resolves to a socket to the target server through proxy.
// pub struct ChainConnector {
//     entry_proxy: TcpStream,
//     chain: Vec<SocketAddr>,
// }

// impl<'a, 't, S> for ChainConnector<'a, 't, S>
// where
//     S: Stream<Item = Result<SocketAddr>> + Unpin
// {
//     fn foo(&self);
// }

// impl io::Read for ProxyStream {
//     fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<io::Result<usize>> {
//         Pin::new(&mut self.inner).poll_read(cx, buf)
//     }
// }


// impl io::Write for ProxyStream {
//     fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<io::Result<usize>> {
//         let nbuf = self.codec.write_package(buf);
//         Pin::new(&mut self.inner).poll_write(cx, &nbuf[..])
//     }
    
//     fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
//         Pin::new(&mut self.inner).poll_flush(cx)
//     }
    
//     fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<()>> {
//         Pin::new(&mut self.inner).poll_close(cx)
//     }
// }


// struct ProxyChain {
//     proxies: Vec<(Box<dyn Protocol>, SocketAddr)>
// }


// impl ProxyChain {
//     fn new(sock: SocketAddr) -> ProxyChain {
//         ProxyChain {
//             proxies: Vec::new(),
//         }
//     }

//     fn push(&mut self, item: (Box<dyn Protocol>, SocketAddr)) {
//         self.proxies.push(item)
//     }

//     fn pop(&mut self, index: usize) {
//         self.proxies.remove(index);
//     }

//     fn create_connection_package(&self, target: SocketAddr) {

//     }   
// }


// impl From<Vec<(Box<dyn Protocol>, SocketAddr)>> for ProxyChain {
//     fn from(proxies: Vec<(Box<dyn Protocol>, SocketAddr)>) -> ProxyChain {
//         ProxyChain { 
//             proxies: Vec::from(proxies)
//         }
//     }
// }
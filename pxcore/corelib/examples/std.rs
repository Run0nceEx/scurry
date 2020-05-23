

// use corelib::*;
// use tokio::net::TcpStream;
// use std::net::SocketAddr;
// use std::error::Error;

// impl Negotiate for TcpStream {}

// #[async_trait::async_trait]
// impl Connector<TcpStream> for TcpStream {
//     async fn init_connect(addr: SocketAddr) -> Result<TcpStream, Box<dyn Error>> {
//         Ok(TcpStream::connect(addr).await?)
//     }
// }

// struct Test;

// #[async_trait::async_trait]
// impl Identifier<TcpStream> for Test {
//   async fn detect(&self, con: &mut TcpStream) -> Result<bool, Box<dyn Error>> {
//       Ok(true)
//   }
// }

// async fn detect<S, C, I>(addr: SocketAddr, scanners: &[I]) -> Option<I>
// where
//     S: Negotiate,
//     C: Connector<S>,
//     I: Identifier<S> + Clone
// {    
//     for scanner in scanners {
//         if let Ok(mut stream) = C::init_connect(addr).await {
//             if scanner.detect(&mut stream).await.is_ok() {
//                 return Some(scanner.clone())
//             }
//         }            
//     }
//     None
// }


// struct A(TcpStream);
// impl A {
//     async fn new(addr: SocketAddr) {
//         let mut stream = TcpStream::connect(addr).await.unwrap();
//         Test{}.detect(&mut stream).await;
//     }
// }

fn main() {}
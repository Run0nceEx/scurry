/// originally from https://github.com/jgall/pnet-futures/blob/master/src/lib.rs
/// we've taken their source and just updated to modern tokio

use pnet::packet::ipv4::Ipv4Packet;
use pnet::transport;
use tokio::stream::Stream;

use std::task::{Context, Poll};
use std::pin::Pin;

pub struct TransportStream<'a> {
    //tr: pnet::transport::TransportReceiver,
    inner: transport::Ipv4TransportChannelIterator<'a>,
}


impl<'a> TransportStream<'a> {
    pub fn new(receiver: &'a mut pnet::transport::TransportReceiver) -> Self {
        Self {
            inner: transport::ipv4_packet_iter(receiver),
        }
    }
}

// You might ask yourself why this struct is necessary:
// the libpnet packet struct contains slice references that cannot be safely passed across threads.
#[derive(Debug)]
pub struct ToPacket<T> {
    content: Vec<u8>,
    pd: std::marker::PhantomData<T>,
}

impl<'a> ToPacket<Ipv4Packet<'a>> {
    pub fn to_packet(&'a mut self) -> Ipv4Packet<'a> {
        Ipv4Packet::new(self.content.as_slice()).unwrap()
    }
}

impl<'a> Stream for TransportStream<'a> {
    type Item = Result<(ToPacket<Ipv4Packet<'a>>, std::net::IpAddr), std::io::Error>;
    //type Error = std::io::Error;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>, ) -> Poll<Option<Self::Item>>{
        const TIMEOUT: u64 = 2500;
        let next = self.inner.next_with_timeout(std::time::Duration::from_nanos(TIMEOUT));

        match next {
            Ok(Some(p)) => {
                use pnet::packet::*;

                let (packet, addr) = p;
                let packet_content = packet.packet();
                
                Poll::Ready(Some(Ok((
                    ToPacket::<Ipv4Packet<'a>> {
                        content: packet_content.to_vec(),
                        pd: std::marker::PhantomData,
                    },
                    addr,
                ))))
            },

            Ok(None) => Poll::Pending,
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pnet::packet::ip::IpNextHeaderProtocols;
    use pnet::transport::transport_channel;
    use pnet::transport::TransportChannelType::Layer4;
    use pnet::transport::TransportProtocol::Ipv4;
    use tokio::runtime::Runtime;
    use tokio::stream::StreamExt;

    mod must_run_with_sudo {
        use super::*;
        // #[test]
        // fn raw_packet_listen() {
        //     let mut rt = Runtime::new().unwrap();

            
        //     let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Test1));

        //     // Create a new transport channel, dealing with layer 4 packets on a test protocol
        //     // It has a receive buffer of 4096 bytes.
        //     let (mut tx, mut rx) = match transport_channel(4096, protocol) {
        //         Ok((tx, rx)) => (tx, rx),
        //         Err(e) => panic!(
        //             "An error occurred when creating the transport channel: {}",
        //             e
        //         ),
        //     };
            
        //     let transport_stream = TransportStream::new(&mut rx);
            
        //     // rt.block_on(
        //     //     transport_stream
        //     //         .filter_map(|x| x.ok())
        //     //         .map(|(p, a)| println!("packet [{}]: {:#?}", a, p))
        //     //         .next()
        //     // );
            
        //     assert_eq!(2 + 2, 4);
        // }
    }
}
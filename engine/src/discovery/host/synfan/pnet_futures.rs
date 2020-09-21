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

            Ok(None) => Poll::Ready(None),
            Err(e) => Poll::Ready(Some(Err(e))),
        }
    }
}
// ties schedules together so that they stream into eachother

/// originally from https://github.com/jgall/pnet-futures/blob/master/src/lib.rs
/// we've taken their source and just updated to modern tokio

use pnet::packet::ipv4::Ipv4Packet;
use pnet::transport;
use tokio::stream::Stream;

use std::task::{Context, Poll};
use std::pin::Pin;

pub struct PoolLink<J1, R1, S1, J2, S2> {
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

impl<'a> Stream for TransportStream<'a> {
    type Item = Result<(ToPacket<Ipv4Packet<'a>>, std::net::IpAddr), std::io::Error>;
    //type Error = std::io::Error;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>, ) -> Poll<Option<Self::Item>>{
        const TIMEOUT: u64 = 5000;
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
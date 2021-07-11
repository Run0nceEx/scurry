use anyhow::Result;
use capsule::batch::{Batch, Pipeline, Poll};
use capsule::config::load_config;
use capsule::packets::ip::v4::Ipv4;
use capsule::packets::ip::v6::Ipv6;
use capsule::packets::ip::IpPacket;
use capsule::packets::{EtherTypes, Ethernet, Packet, Tcp, Tcp4, Tcp6};
use capsule::{compose, Mbuf, PortQueue, Runtime};
use colored::*;
use tracing::{debug, Level};
use tracing_subscriber::fmt;

#[inline]
fn dump_eth(packet: Mbuf) -> Result<Ethernet> {
    let ethernet = packet.parse::<Ethernet>()?;

    let info_fmt = format!("{:?}", ethernet).magenta().bold();
    println!("{}", info_fmt);

    Ok(ethernet)
}

#[inline]
fn dump_v4(ethernet: &Ethernet) -> Result<()> {
    let v4 = ethernet.peek::<Ipv4>()?;
    let info_fmt = format!("{:?}", v4).yellow();
    println!("{}", info_fmt);

    let tcp = v4.peek::<Tcp4>()?;
    dump_tcp(&tcp);

    Ok(())
}

#[inline]
fn dump_v6(ethernet: &Ethernet) -> Result<()> {
    let v6 = ethernet.peek::<Ipv6>()?;
    let info_fmt = format!("{:?}", v6).cyan();
    println!("{}", info_fmt);

    let tcp = v6.peek::<Tcp6>()?;
    dump_tcp(&tcp);

    Ok(())
}

#[inline]
fn dump_tcp<T: IpPacket>(tcp: &Tcp<T>) {
    let tcp_fmt = format!("{:?}", tcp).green();
    println!("{}", tcp_fmt);

    let flow_fmt = format!("{:?}", tcp.flow()).bright_blue();
    println!("{}", flow_fmt);
}

fn install(q: PortQueue) -> impl Pipeline {
    Poll::new(q.clone())
        .map(dump_eth)
        .group_by(
            |ethernet| ethernet.ether_type(),
            |groups| {
                compose!( groups {
                    EtherTypes::Ipv4 => |group| {
                        group.for_each(dump_v4)
                    }
                    EtherTypes::Ipv6 => |group| {
                        group.for_each(dump_v6)
                    }
                });
            },
        )
        .send(q)
}

// fn main() -> Result<()> {
//     let subscriber = fmt::Subscriber::builder()
//         .with_max_level(Level::DEBUG)
//         .finish();
//     tracing::subscriber::set_global_default(subscriber)?;

//     let config = load_config()?;
//     debug!(?config);

//     Runtime::build(config)?
//         .add_pipeline_to_port("eth1", install)?
//         .add_pipeline_to_port("eth2", install)?
//         .execute()
// }
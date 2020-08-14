// This is a scanner thats concepts
// and resources are adopted from the nmap project
// theres probably 100 things im doing wrong, and that im missing (syn packets for ex)
// i do accept contributions, and they'll probably be on this file.
use crate::{
    runtime::{
        {SignalControl, CRON},
        //pool::Subscriber,
        CronMeta,
    },
    error::Error,
};

use tokio::net::TcpStream;
use std::net::SocketAddr;


#[derive(Debug, Clone)]
pub struct Job {
    addr: SocketAddr,
}


#[derive(Debug)]
pub struct Peer {
    addr: TcpStream,

}


#[derive(Debug)]
pub struct Worker;

#[async_trait::async_trait]
impl CRON for Worker
{
    type State = Job;
    type Response = ();

    async fn exec(state: &mut Job) -> Result<(SignalControl, Option<Self::Response>), Error>
    {

        Ok((SignalControl::Success(false), None))
    }

    fn name() -> String {
        let x = format!("{:?}", Worker);
        x
    }
}

use corelib::{ConnectionHandler, Connector};

use crate::{
    schedule::{
        {CronControls, CRON},
        core::CronMeta,
        //listener::EventListener
    },
    database::models::latency::LatencyModel
};

use tokio::{
    time::{Instant, Duration},
};

use std::net::SocketAddr;

const MEASUREMENTS: usize = 5;


#[derive(Clone)]
pub struct Latency {
    results: [Duration; MEASUREMENTS],
    ctr: usize,
}

#[derive(Clone)]
pub struct LatencyJob {
    addr: SocketAddr,
    res: Latency<T>,
    tts: Duration
}


pub struct OpenPortJob;

#[async_trait::async_trait]
impl CRON for OpenPortJob
{
    type State = Job;
    type Response = ScheduleControls<PortState>;

    async fn exec(state: Job) -> (Self::Response, Self::State)
    {
        match scan(state.addr).await {
            Ok(_)   => return (ScheduleControls::Success(PortState::Open(state.addr)), state), 
            Err(_e) => return (ScheduleControls::Success(PortState::Closed(state.addr)), state)
        }
    }
}


pub struct PrintSub;

#[async_trait::async_trait]
impl Subscriber<PortState> for PrintSub {
    async fn handle(&self, data: &PortState) -> Result<(), Error> {
        println!("{:?}", data)
    }
}

impl std::fmt::Display for PrintSub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrintSub")
    }
}





// #[async_trait::async_trait]
// impl<C> EventListener<Latency<C>> for LatencyModel where C: Connector + Send + 'static {
//     fn filter<T>(&self, evt_name: &str, meta: CronMeta<T, Latency<C>>) -> bool {
//         if let Some(meta) = meta.last_ctrl {   
//             match meta {
//                 CronControls::Success(resp) => {
//                     return evt_name == "Latency_test";
//                 }
//                 _ => return false
//             }            
//         }
//         false
//     }

//     async fn handle(&mut self, data: Latency<C>) {

//     }
// }
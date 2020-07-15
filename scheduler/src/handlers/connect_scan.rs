use crate::{
    schedule::{
        {ScheduleControls, CRON, CronMeta},
        sugar::Subscriber,
    },
    database::models::latency::LatencyModel,
    error::Error,
};
use tokio::{
    net::TcpStream    
};
use std::net::SocketAddr;


#[derive(Debug, Clone)]
pub struct Job {
    addr: SocketAddr,
    ctr: u8,
    ctr_max: u8
}

impl Job {
    pub fn new(addr: SocketAddr, max: u8) -> Self {
        Self {
            addr: addr,
            ctr: 0,
            ctr_max: max
        }
    }
}

#[derive(Debug, Clone)]
pub enum PortState {
    Open(SocketAddr),
    Closed(SocketAddr)
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
            Err(_e) => {
                return (ScheduleControls::Success(PortState::Closed(state.addr)), state)    
            }
        }
    }
}

async fn scan(addr: SocketAddr) -> Result<(), crate::error::Error> {
    //TODO Add timeout
    let mut connection = TcpStream::connect(addr).await?;
    
    Ok(())
}

pub struct PrintSub {
    instant: std::time::Instant,
    ctr: u64,
}

impl PrintSub {
    pub fn new() -> Self {
        Self {
            instant: std::time::Instant::now(),
            ctr: 0
        }
    }
}

#[async_trait::async_trait]
impl Subscriber<(ScheduleControls<PortState>, CronMeta, Job)> for PrintSub {
    async fn handle(&mut self, data: &(ScheduleControls<PortState>, CronMeta, Job)) -> Result<(), Error> {
        self.ctr += 1;
        //println!("[{:?}] {:?}", self.ctr, data);
        if self.ctr == 20000 {
          println!("Done");
          panic!("");
        }
        Ok(())
    }
}

impl std::fmt::Display for PrintSub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrintSub")
    }
}


// #[async_trait::async_trait]
// impl EventListener<Latency> for LatencyModel {
//     fn filter<T>(&self, evt_name: &str, meta: CronMeta<T, Latency>) -> bool {
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

//     async fn handle(&mut self, data: Latency) {
   
//     }
// }